use crate::{
    constants::{EFF_MASK, SFF_MASK},
    error::Error,
    CanResult,
};
use bitflags::bitflags;
use serde::{Deserialize, Serialize};

bitflags! {
    /// Identifier flags for indicating various frame types.
    ///
    /// These flags are applied logically in `can`, but flag values themselves correspond to the
    /// format used by the Linux [SocketCAN][socketcan] library.  This lets flags be applied
    /// logically to identifiers such that callers can construct their calls to the underlying CAN
    /// transceivers/controllers in whatever way is required, but also provides a happy path for
    /// SocketCAN users by allowing generation of the all-in-one 32-bit identifier value.
    ///
    /// [socketcan]: https://www.kernel.org/doc/Documentation/networking/can.txt
    #[repr(transparent)]
    pub struct IdentifierFlags: u32 {
        /// The frame is using the extended format i.e. 29-bit extended identifiers.
        const EXTENDED = 0x8000_0000;
        /// The frame is a remote transmission request.
        const REMOTE = 0x4000_0000;
        /// The frame is an error frame.
        const ERROR = 0x2000_0000;
    }
}

bitflags! {
    #[repr(transparent)]
    pub struct CanFdFlags: u8 {
        /// bit rate switch (second bitrate for payload data)
        const BRS = 0x01;
        /// error state indicator of the transmitting node
        const ESI = 0x02;
        /// if set, the frame is a CAN FD frame;
        /// if not set, the frame may be a CAN CC frame or a CAN FD frame.
        const FDF = 0x04;
    }
}

bitflags! {
    #[repr(transparent)]
    pub struct CanXlFlags: u8 {
        /// Simple Extended Content.
        const SEC = 0x01;
        /// Remote Request Substitution.
        const RRS = 0x02;
        /// if set, the frame is a CAN XL frame;
        /// if not set, the frame is a CAN CC frame or a CAN FD frame.
        const XLF = 0x80;
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Deserialize, Serialize, Hash)]
pub enum Filter {
    Standard { id: StandardId, mask: u16 },
    Extended { id: ExtendedId, mask: u32 },
}

impl Default for Filter {
    fn default() -> Self {
        Self::Standard {
            id: StandardId::default(),
            mask: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Hash)]
pub enum Id {
    Standard(StandardId),
    Extended(ExtendedId),
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Hash)]
pub struct StandardId(u16);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Hash)]
pub struct ExtendedId(u32);

impl StandardId {
    pub const MAX: u16 = 0x7FF;

    pub fn new(id: u16) -> CanResult<Self> {
        if id > Self::MAX {
            Err(Error::InvalidIdentifier(id as u32))
        } else {
            Ok(Self(id))
        }
    }

    #[inline]
    pub fn as_raw(self) -> u16 {
        self.0
    }
}

impl ExtendedId {
    pub const MAX: u32 = 0x1FFF_FFFF;

    pub fn new(id: u32) -> CanResult<Self> {
        if id > Self::MAX {
            Err(Error::InvalidIdentifier(id))
        } else {
            Ok(Self(id))
        }
    }

    #[inline]
    pub fn as_raw(self) -> u32 {
        self.0
    }
}

impl Into<u32> for Id {
    /// 32-bit integer with extended flag
    fn into(self) -> u32 {
        self.into_socketcan_bits()
    }
}

impl TryFrom<u32> for Id {
    type Error = Error;

    fn try_from(v: u32) -> Result<Self, Self::Error> {
        let flag_mask =
            (IdentifierFlags::EXTENDED | IdentifierFlags::REMOTE | IdentifierFlags::ERROR).bits();
        let raw = v & !flag_mask;

        if v & IdentifierFlags::EXTENDED.bits() != 0 {
            ExtendedId::new(raw & EFF_MASK).map(Id::Extended)
        } else {
            StandardId::new((raw & SFF_MASK) as u16).map(Id::Standard)
        }
    }
}

impl Id {
    #[inline]
    pub fn from_bits(raw: u32, extended: Option<bool>) -> CanResult<Self> {
        match extended {
            Some(true) => ExtendedId::new(raw & EFF_MASK).map(Self::Extended),
            Some(false) => StandardId::new((raw & SFF_MASK) as u16).map(Self::Standard),
            None => Self::try_from(raw),
        }
    }

    #[inline]
    pub fn as_raw(self) -> u32 {
        match self {
            Self::Standard(id) => id.as_raw() as u32,
            Self::Extended(id) => id.as_raw(),
        }
    }

    #[inline]
    pub fn is_extended(&self) -> bool {
        matches!(self, Self::Extended(_))
    }

    #[inline]
    pub fn into_socketcan_bits(self) -> u32 {
        match self {
            Self::Standard(id) => id.as_raw() as u32,
            Self::Extended(id) => id.as_raw() | IdentifierFlags::EXTENDED.bits(),
        }
    }

    /// Display [`Id`] as hex string.
    #[inline]
    pub fn into_hex(self) -> String {
        format!("{:08X}", self.into_socketcan_bits())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_into_u32_uses_socketcan_bits() {
        let extended = Id::Extended(ExtendedId::new(0x123456).unwrap());
        let raw: u32 = extended.into();

        assert_eq!(raw, 0x0012_3456 | IdentifierFlags::EXTENDED.bits());
    }

    #[test]
    fn try_from_socketcan_bits_masks_non_identifier_flags() {
        let bits = 0x0012_3456
            | IdentifierFlags::EXTENDED.bits()
            | IdentifierFlags::REMOTE.bits()
            | IdentifierFlags::ERROR.bits();

        let id = Id::try_from(bits).unwrap();
        assert_eq!(id, Id::Extended(ExtendedId::new(0x0012_3456).unwrap()));
    }
}

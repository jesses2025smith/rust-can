use crate::native::can::ZCanFrame;
use rs_can::{
    can_utils, CanDirection, CanError, CanFdFlags, CanKind, IdentifierFlags, DEFAULT_PADDING,
    EFF_MASK, MAX_FRAME_SIZE,
};
use std::{
    ffi::{c_uchar, c_uint},
    fmt::{Display, Formatter},
};

/// Then CAN frame type used in crate.
#[repr(C)]
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone)]
pub enum ZCanFrameType {
    CAN = 0,
    CANFD = 1,
    ALL = 2,
}

impl TryFrom<u8> for ZCanFrameType {
    type Error = CanError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::CAN),
            1 => Ok(Self::CANFD),
            2 => Ok(Self::ALL),
            _ => Err(CanError::other_error("parameter not supported")),
        }
    }
}

impl Display for ZCanFrameType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CAN => write!(f, "CAN"),
            Self::CANFD => write!(f, "CANFD"),
            Self::ALL => write!(f, "CAN|CANFD"),
        }
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub enum ZCanTxMode {
    #[default]
    Normal = 0, //**< normal transmission */
    Once = 1,              //**< single-shot transmission */
    SelfReception = 2,     //**< self reception */
    SelfReceptionOnce = 3, //**< single-shot transmission & self reception */
}

impl TryFrom<u8> for ZCanTxMode {
    type Error = CanError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Normal),
            1 => Ok(Self::Once),
            2 => Ok(Self::SelfReception),
            3 => Ok(Self::SelfReceptionOnce),
            _ => Err(CanError::other_error("parameter not supported")),
        }
    }
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub(crate) struct ZCanChlErrorInner {
    pub(crate) code: c_uint,
    pub(crate) passive: [c_uchar; 3],
    pub(crate) arb_lost: c_uchar,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct ZCanMsg20<const S: usize> {
    pub(crate) can_id: c_uint,
    pub(crate) can_len: c_uchar,
    pub(crate) flags: c_uchar, /* padding when using can else additional flags for CAN FD,i.e error code */
    pub(crate) __res0: c_uchar, /* reserved / padding (used for channel) */
    #[allow(dead_code)]
    pub(crate) __res1: c_uchar, /* reserved / padding */
    pub(crate) data: [c_uchar; S],
}

impl<const S: usize> ZCanMsg20<S> {
    pub fn new(can_id: c_uint, can_len: c_uchar, flags: c_uchar, data: [c_uchar; S]) -> Self {
        Self {
            can_id,
            can_len,
            flags,
            __res0: Default::default(),
            __res1: Default::default(),
            data,
        }
    }

    #[inline(always)]
    pub fn set_channel(&mut self, channel: u8) -> &Self {
        self.__res0 = channel;
        self
    }
    #[allow(unused)]
    #[inline(always)]
    pub fn get_channel(&self) -> u8 {
        self.__res0
    }
}

impl<const S: usize> Default for ZCanMsg20<S> {
    fn default() -> Self {
        Self {
            can_id: Default::default(),
            can_len: Default::default(),
            flags: Default::default(),
            __res0: Default::default(),
            __res1: Default::default(),
            data: [Default::default(); S],
        }
    }
}

impl<const S: usize> Into<ZCanFrame> for ZCanMsg20<S> {
    fn into(self) -> ZCanFrame {
        #[allow(deprecated)]
        let kind = can_utils::can_kind_by_len(S).unwrap();

        let can_id = self.can_id;
        let length = self.can_len as usize;
        let mut data = self.data.to_vec();
        data.resize(length, Default::default());
        ZCanFrame {
            timestamp: None,
            arbitration_id: can_id & EFF_MASK,
            is_extended_id: (can_id & IdentifierFlags::EXTENDED.bits()) > 0,
            is_remote_frame: (can_id & IdentifierFlags::REMOTE.bits()) > 0,
            is_error_frame: (can_id & IdentifierFlags::ERROR.bits()) > 0,
            channel: self.__res0,
            length,
            data,
            kind,
            direction: CanDirection::Receive,
            bitrate_switch: match kind {
                CanKind::Classical => false,
                CanKind::FD => self.flags & CanFdFlags::BRS.bits() > 0,
                CanKind::XL => todo!("XL is not supported!"),
            },
            error_state_indicator: match kind {
                CanKind::Classical => false,
                CanKind::FD => self.flags & CanFdFlags::ESI.bits() > 0,
                CanKind::XL => todo!("XL is not supported!"),
            },
            tx_mode: None,
        }
    }
}

impl<const S: usize> From<ZCanFrame> for ZCanMsg20<S> {
    fn from(msg: ZCanFrame) -> Self {
        let is_fd = S > MAX_FRAME_SIZE;

        let can_id = can_id_add_flags(&msg);
        let length = msg.length as u8;
        let flags = if is_fd {
            (if msg.bitrate_switch {
                CanFdFlags::BRS.bits()
            } else {
                Default::default()
            }) | (if msg.error_state_indicator {
                CanFdFlags::ESI.bits()
            } else {
                Default::default()
            })
        } else {
            Default::default()
        };
        let mut data = msg.data;
        data.resize(S, DEFAULT_PADDING);

        Self::new(can_id, length, flags, data.try_into().unwrap())
    }
}

// pub(crate) type ZCanChlError = ZCanChlErrorInner;
fn can_id_add_flags(msg: &ZCanFrame) -> u32 {
    msg.arbitration_id
        | if msg.is_extended_id {
            IdentifierFlags::EXTENDED.bits()
        } else {
            Default::default()
        }
        | if msg.is_remote_frame {
            IdentifierFlags::REMOTE.bits()
        } else {
            Default::default()
        }
        | if msg.is_error_frame {
            IdentifierFlags::ERROR.bits()
        } else {
            Default::default()
        }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rs_can::{CanFrame, CanId, StandardId};

    #[test]
    fn remote_frame_round_trip_preserves_dlc() {
        let id = CanId::Standard(StandardId::new(0x123).unwrap());
        let msg = ZCanFrame::new_remote(id, 8).unwrap();

        let native: ZCanMsg20<8> = msg.into();
        assert_eq!(native.can_len, 8);

        let round_trip: ZCanFrame = native.into();
        assert!(round_trip.is_remote_frame);
        assert_eq!(round_trip.length, 8);
    }
}

use crate::can::ZCanTxMode;
use rs_can::{
    CanDirection, CanError, CanFdFlags, CanFrame, CanId, CanKind, CanResult, FrameFormat,
    Timestamp, MAX_FD_FRAME_SIZE, MAX_FRAME_SIZE,
};
use std::fmt::{Display, Formatter};

#[repr(C)]
#[derive(Debug, Clone)]
pub struct ZCanFrame {
    pub(crate) timestamp: Option<Timestamp>,
    pub(crate) arbitration_id: u32,
    pub(crate) is_extended_id: bool,
    pub(crate) is_remote_frame: bool,
    pub(crate) is_error_frame: bool,
    pub(crate) channel: u8,
    pub(crate) length: usize,
    pub(crate) data: Vec<u8>,
    pub(crate) kind: CanKind,
    pub(crate) direction: CanDirection,
    pub(crate) bitrate_switch: bool,
    pub(crate) error_state_indicator: bool,
    pub(crate) tx_mode: Option<u8>,
}

impl CanFrame for ZCanFrame {
    type Channel = u8;

    #[inline]
    fn new_can(id: CanId, data: &[u8]) -> CanResult<Self> {
        let length = data.len();
        if length > MAX_FRAME_SIZE {
            return Err(CanError::InvalidDLC(length));
        }
        Ok(Self {
            timestamp: None,
            arbitration_id: id.as_raw(),
            is_extended_id: id.is_extended(),
            is_remote_frame: false,
            is_error_frame: false,
            channel: Default::default(),
            length,
            data: data.to_vec(),
            kind: CanKind::Classical,
            direction: CanDirection::Transmit,
            bitrate_switch: false,
            error_state_indicator: false,
            tx_mode: Default::default(),
        })
    }

    #[inline]
    fn new_remote(id: CanId, dlc: u8) -> CanResult<Self> {
        if dlc as usize > MAX_FRAME_SIZE {
            return Err(CanError::InvalidDLC(dlc as usize));
        }
        Ok(Self {
            timestamp: None,
            arbitration_id: id.as_raw(),
            is_extended_id: id.is_extended(),
            is_remote_frame: true,
            is_error_frame: false,
            channel: Default::default(),
            length: dlc as usize,
            data: Default::default(),
            kind: CanKind::Classical,
            direction: CanDirection::Transmit,
            bitrate_switch: false,
            error_state_indicator: false,
            tx_mode: Default::default(),
        })
    }

    #[inline]
    fn new_can_fd(id: CanId, data: &[u8], flags: CanFdFlags) -> CanResult<Self> {
        let length = data.len();
        if length > MAX_FD_FRAME_SIZE {
            return Err(CanError::InvalidDLC(length));
        }
        Ok(Self {
            timestamp: None,
            arbitration_id: id.as_raw(),
            is_extended_id: id.is_extended(),
            is_remote_frame: false,
            is_error_frame: false,
            channel: Default::default(),
            length,
            data: data.to_vec(),
            kind: CanKind::FD,
            direction: CanDirection::Transmit,
            bitrate_switch: flags.contains(CanFdFlags::BRS),
            error_state_indicator: flags.contains(CanFdFlags::ESI),
            tx_mode: Default::default(),
        })
    }

    #[inline]
    fn id(&self) -> CanId {
        CanId::from_bits(self.arbitration_id, Some(self.is_extended_id)).unwrap()
    }

    #[inline]
    fn channel(&self) -> Self::Channel {
        self.channel
    }

    #[inline]
    fn set_channel(&mut self, v: Self::Channel) -> &mut Self {
        self.channel = v;
        self
    }

    #[inline]
    fn kind(&self) -> CanKind {
        self.kind
    }

    #[inline]
    fn format(&self) -> FrameFormat {
        if self.is_remote_frame {
            FrameFormat::Remote
        } else if self.is_error_frame {
            FrameFormat::Error
        } else {
            FrameFormat::Data
        }
    }

    #[inline]
    fn data(&self) -> &[u8] {
        self.data.as_slice()
    }

    #[inline]
    fn len(&self) -> usize {
        self.length
    }

    #[inline]
    fn direction(&self) -> CanDirection {
        self.direction
    }

    #[inline]
    fn set_direction(&mut self, d: CanDirection) -> &mut Self {
        self.direction = d;
        self
    }

    #[inline]
    fn timestamp(&self) -> Option<Timestamp> {
        self.timestamp
    }

    #[inline]
    fn set_timestamp(&mut self, ts: Option<Timestamp>) -> &mut Self {
        self.timestamp = ts;
        self
    }

    #[inline]
    fn is_bitrate_switch(&self) -> bool {
        self.bitrate_switch
    }

    #[inline]
    fn set_bitrate_switch(&mut self, v: bool) -> &mut Self {
        self.bitrate_switch = v;
        self
    }

    #[inline]
    fn is_esi(&self) -> bool {
        self.error_state_indicator
    }

    #[inline]
    fn set_esi(&mut self, v: bool) -> &mut Self {
        self.error_state_indicator = v;
        self
    }
}

impl PartialEq for ZCanFrame {
    fn eq(&self, other: &Self) -> bool {
        if self.length != other.length {
            return false;
        }

        if self.is_remote_frame {
            other.is_remote_frame && (self.arbitration_id == other.arbitration_id)
        } else {
            (self.arbitration_id == other.arbitration_id)
                && (self.is_extended_id == other.is_extended_id)
                && (self.is_error_frame == other.is_error_frame)
                && (self.error_state_indicator == other.error_state_indicator)
                && (self.data == other.data)
        }
    }
}

impl ZCanFrame {
    #[inline(always)]
    pub fn tx_mode(&self) -> u8 {
        self.tx_mode.unwrap_or_else(|| ZCanTxMode::default() as u8)
    }
    #[inline(always)]
    pub fn set_tx_mode(&mut self, tx_mode: ZCanTxMode) -> &mut Self {
        self.tx_mode = Some(tx_mode as u8);
        self
    }
}

impl Display for ZCanFrame {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        <dyn CanFrame<Channel = u8> as Display>::fmt(self, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rs_can::{CanId, StandardId};

    #[test]
    fn invalid_standard_id_is_clamped_to_fallback_identifier() {
        let msg = ZCanFrame {
            timestamp: None,
            arbitration_id: 0x800,
            is_extended_id: false,
            is_remote_frame: false,
            is_error_frame: false,
            channel: 0,
            length: 0,
            data: Vec::new(),
            kind: CanKind::Classical,
            direction: CanDirection::Receive,
            bitrate_switch: false,
            error_state_indicator: false,
            tx_mode: None,
        };

        assert_eq!(msg.id(), CanId::Standard(StandardId::new(0).unwrap()));
    }
}

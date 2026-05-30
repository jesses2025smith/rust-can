use crate::{socket, FD_FRAME_SIZE, FRAME_SIZE, XL_FRAME_SIZE};
use libc::{can_frame, canfd_frame, canxl_frame};
use rs_can::{
    can_utils, CanDirection, CanError, CanFdFlags, CanFrame, CanId, CanKind, CanResult,
    FrameFormat, IdentifierFlags, Timestamp, EFF_MASK, MAX_FD_FRAME_SIZE, MAX_FRAME_SIZE,
};
use std::fmt::{Display, Formatter};

pub enum CanAnyFrame {
    Normal(can_frame),
    Remote(can_frame),
    Error(can_frame),
    FD(canfd_frame),
    XL(canxl_frame),
}

impl CanAnyFrame {
    pub fn size(&self) -> usize {
        match self {
            Self::Normal(_) => FRAME_SIZE,
            Self::Remote(_) => FRAME_SIZE,
            Self::Error(_) => FRAME_SIZE,
            Self::FD(_) => FD_FRAME_SIZE,
            Self::XL(_) => XL_FRAME_SIZE,
        }
    }
}

impl From<can_frame> for CanAnyFrame {
    #[inline(always)]
    fn from(frame: can_frame) -> CanAnyFrame {
        let can_id = frame.can_id;
        if can_id & IdentifierFlags::REMOTE.bits() != 0 {
            Self::Remote(frame)
        } else if can_id & IdentifierFlags::ERROR.bits() != 0 {
            Self::Error(frame)
        } else {
            Self::Normal(frame)
        }
    }
}

impl From<canfd_frame> for CanAnyFrame {
    #[inline(always)]
    fn from(frame: canfd_frame) -> Self {
        Self::FD(frame)
    }
}

impl From<canxl_frame> for CanAnyFrame {
    fn from(frame: canxl_frame) -> Self {
        Self::XL(frame)
    }
}

#[derive(Debug, Clone)]
pub struct SocketCanFrame {
    pub(crate) timestamp: Option<Timestamp>,
    pub(crate) arbitration_id: u32,
    pub(crate) is_extended_id: bool,
    pub(crate) is_remote_frame: bool,
    pub(crate) is_error_frame: bool,
    pub(crate) channel: String,
    pub(crate) length: usize,
    pub(crate) data: Vec<u8>,
    pub(crate) kind: CanKind,
    pub(crate) direction: CanDirection,
    pub(crate) bitrate_switch: bool,
    pub(crate) error_state_indicator: bool,
}

impl SocketCanFrame {
    fn socketcan_id_bits(&self) -> u32 {
        let id = self.id();
        let mut can_id = id.into_socketcan_bits();

        if self.is_error_frame {
            can_id |= IdentifierFlags::ERROR.bits();
        }

        if self.is_remote_frame {
            can_id |= IdentifierFlags::REMOTE.bits();
        }

        can_id
    }
}

impl TryFrom<CanAnyFrame> for SocketCanFrame {
    type Error = CanError;

    fn try_from(frame: CanAnyFrame) -> Result<Self, Self::Error> {
        match frame {
            CanAnyFrame::Normal(f) => Ok(Self {
                timestamp: None,
                arbitration_id: f.can_id & EFF_MASK,
                is_extended_id: f.can_id & IdentifierFlags::EXTENDED.bits() != 0,
                is_remote_frame: false,
                is_error_frame: false,
                channel: Default::default(),
                length: f.can_dlc as usize,
                data: f.data[..f.can_dlc as usize].to_vec(),
                kind: CanKind::Classical,
                direction: Default::default(),
                bitrate_switch: false,
                error_state_indicator: false,
            }),
            CanAnyFrame::Remote(f) => Ok(Self {
                timestamp: None,
                arbitration_id: f.can_id & EFF_MASK,
                is_extended_id: f.can_id & IdentifierFlags::EXTENDED.bits() != 0,
                is_remote_frame: true,
                is_error_frame: false,
                channel: Default::default(),
                length: f.can_dlc as usize,
                data: f.data[..f.can_dlc as usize].to_vec(),
                kind: CanKind::Classical,
                direction: Default::default(),
                bitrate_switch: false,
                error_state_indicator: false,
            }),
            CanAnyFrame::Error(f) => Ok(Self {
                timestamp: None,
                arbitration_id: f.can_id & EFF_MASK,
                is_extended_id: f.can_id & IdentifierFlags::EXTENDED.bits() != 0,
                is_remote_frame: false,
                is_error_frame: true,
                channel: Default::default(),
                length: f.can_dlc as usize,
                data: f.data[..f.can_dlc as usize].to_vec(),
                kind: CanKind::Classical,
                direction: Default::default(),
                bitrate_switch: false,
                error_state_indicator: false,
            }),
            CanAnyFrame::FD(f) => Ok(Self {
                timestamp: None,
                arbitration_id: f.can_id & EFF_MASK,
                is_extended_id: f.can_id & IdentifierFlags::EXTENDED.bits() != 0,
                is_remote_frame: false,
                is_error_frame: false,
                channel: Default::default(),
                length: f.len as usize,
                data: f.data[..f.len as usize].to_vec(),
                kind: CanKind::FD,
                direction: Default::default(),
                bitrate_switch: f.flags & 0x01 != 0,
                error_state_indicator: f.flags & 0x02 != 0,
            }),
            CanAnyFrame::XL(_) => Err(CanError::NotImplementedError),
        }
    }
}

impl Into<CanAnyFrame> for SocketCanFrame {
    fn into(self) -> CanAnyFrame {
        match self.kind {
            CanKind::Classical => {
                let mut frame = socket::can_frame_default();
                let length = self.data.len();
                frame.data[..length].copy_from_slice(&self.data);
                frame.can_dlc = length as u8;
                let can_id = self.socketcan_id_bits();

                if self.is_error_frame {
                    frame.can_id = can_id;
                    return CanAnyFrame::Error(frame);
                }

                if self.is_remote_frame {
                    frame.can_id = can_id;
                    return CanAnyFrame::Remote(frame);
                }

                frame.can_id = can_id;
                CanAnyFrame::Normal(frame)
            }
            CanKind::FD => {
                let mut frame = socket::canfd_frame_default();
                let can_id = self.socketcan_id_bits();

                let length = self.data.len();
                frame.can_id = can_id;
                frame.data[..length].copy_from_slice(&self.data);
                frame.len = length as u8;
                frame.flags |= CanFdFlags::FDF.bits();
                if self.bitrate_switch {
                    frame.flags |= CanFdFlags::BRS.bits();
                }

                if self.error_state_indicator {
                    frame.flags |= CanFdFlags::ESI.bits();
                }

                CanAnyFrame::FD(frame)
            }
            CanKind::XL => todo!("XL is not supported now!"),
        }
    }
}

impl CanFrame for SocketCanFrame {
    type Channel = String;

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
            direction: Default::default(),
            bitrate_switch: false,
            error_state_indicator: false,
        })
    }

    fn new_remote(id: CanId, dlc: u8) -> CanResult<Self> {
        let dlc = dlc as usize;
        if dlc > MAX_FRAME_SIZE {
            return Err(CanError::InvalidDLC(dlc));
        }
        let mut data = Vec::new();
        can_utils::data_resize(&mut data, dlc);

        Ok(Self {
            timestamp: None,
            arbitration_id: id.as_raw(),
            is_extended_id: id.is_extended(),
            is_remote_frame: true,
            is_error_frame: false,
            channel: Default::default(),
            length: dlc,
            data,
            kind: CanKind::Classical,
            direction: Default::default(),
            bitrate_switch: false,
            error_state_indicator: false,
        })
    }

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
            direction: Default::default(),
            bitrate_switch: flags.contains(CanFdFlags::BRS),
            error_state_indicator: flags.contains(CanFdFlags::ESI),
        })
    }

    fn id(&self) -> CanId {
        CanId::from_bits(self.arbitration_id, Some(self.is_extended_id)).unwrap()
    }

    fn channel(&self) -> Self::Channel {
        self.channel.clone()
    }

    fn set_channel(&mut self, v: Self::Channel) -> &mut Self {
        self.channel = v;
        self
    }

    fn kind(&self) -> CanKind {
        self.kind
    }

    fn format(&self) -> FrameFormat {
        if self.is_error_frame {
            FrameFormat::Error
        } else if self.is_remote_frame {
            FrameFormat::Remote
        } else {
            FrameFormat::Data
        }
    }

    fn data(&self) -> &[u8] {
        &self.data
    }

    fn len(&self) -> usize {
        self.length
    }

    fn direction(&self) -> CanDirection {
        self.direction
    }

    fn set_direction(&mut self, d: CanDirection) -> &mut Self {
        self.direction = d;
        self
    }

    fn timestamp(&self) -> Option<Timestamp> {
        self.timestamp
    }

    fn set_timestamp(&mut self, ts: Option<Timestamp>) -> &mut Self {
        self.timestamp = ts;
        self
    }

    fn is_bitrate_switch(&self) -> bool {
        self.bitrate_switch
    }

    fn set_bitrate_switch(&mut self, v: bool) -> &mut Self {
        self.bitrate_switch = v;
        self
    }

    fn is_esi(&self) -> bool {
        self.error_state_indicator
    }

    fn set_esi(&mut self, v: bool) -> &mut Self {
        self.error_state_indicator = v;
        self
    }
}

impl PartialEq for SocketCanFrame {
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

impl Display for SocketCanFrame {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        <dyn CanFrame<Channel = String> as Display>::fmt(self, f)
    }
}

use rs_can::{
    CanDirection, CanError, CanFdFlags, CanFrame, CanId, CanKind, CanResult, ExtendedId,
    FrameFormat, StandardId, Timestamp, TimestampSource, MAX_FD_FRAME_SIZE, MAX_FRAME_SIZE,
};
use std::fmt::{Display, Formatter};

#[repr(C)]
#[derive(Debug, Clone)]
pub struct NiCanFrame {
    timestamp: Option<Timestamp>,
    arbitration_id: u32,
    is_extended_id: bool,
    is_remote_frame: bool,
    is_error_frame: bool,
    channel: String,
    length: usize,
    data: Vec<u8>,
    direction: CanDirection,
    bitrate_switch: bool,
    error_state_indicator: bool,
}

impl CanFrame for NiCanFrame {
    type Channel = String;

    fn new_can(id: CanId, data: &[u8]) -> CanResult<Self> {
        if data.len() > MAX_FRAME_SIZE {
            return Err(CanError::InvalidDLC(data.len()));
        }

        Ok(Self {
            timestamp: None,
            arbitration_id: id.as_raw(),
            is_extended_id: id.is_extended(),
            is_remote_frame: false,
            is_error_frame: false,
            channel: Default::default(),
            length: data.len(),
            data: data.to_vec(),
            direction: CanDirection::Transmit,
            bitrate_switch: false,
            error_state_indicator: false,
        })
    }

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
            data: vec![0; dlc as usize],
            direction: CanDirection::Transmit,
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
            direction: CanDirection::Transmit,
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
        CanKind::Classical
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
        self.timestamp = td;
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

impl Display for NiCanFrame {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        <dyn CanFrame<Channel = String> as Display>::fmt(self, f)
    }
}

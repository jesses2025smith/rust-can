use crate::can::ZCanTxMode;
use rs_can::{
    CanDirection, CanError, CanFdFlags, CanFrame, CanId, CanKind, CanResult, FrameFormat,
    Timestamp, MAX_FD_FRAME_SIZE, MAX_FRAME_SIZE,
};
use std::fmt::{Display, Formatter};

#[cfg_attr(feature = "pyo3", pyo3::pyclass(from_py_object))]
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

#[cfg(feature = "pyo3")]
impl ZCanFrame {
    const TIMESTAMP: &str = "timestamp";
    const ARBITRATION_ID: &str = "arbitration_id";
    const IS_EFF: &str = "is_extended_id"; /* extended frame format (EFF) */
    const IS_RTR: &str = "is_remote_frame"; /* remote transmission request */
    const IS_ERR: &str = "is_error_frame"; /* error message frame */
    const CHANNEL: &str = "channel";
    const DLC: &str = "dlc";
    const DATA: &str = "data";
    const IS_CAN_FD: &str = "is_fd";
    const IS_RX: &str = "is_rx";
    const IS_BRS: &str = "bitrate_switch";
    const IS_ESI: &str = "error_state_indicator";
    const PYTHON_CAN: &str = "can";
    const PYTHON_CAN_MESSAGE: &str = "Message";

    pub fn to_python<'py>(
        &self,
        py: pyo3::Python<'py>,
    ) -> pyo3::PyResult<pyo3::Bound<'py, pyo3::PyAny>> {
        use pyo3::types::{PyAnyMethods, PyDict};

        let can_mod = py.import(Self::PYTHON_CAN)?;
        let message_class = can_mod.getattr(Self::PYTHON_CAN_MESSAGE)?;

        let kwargs = PyDict::new(py);
        kwargs.set_item(
            Self::TIMESTAMP,
            self.timestamp.unwrap_or_default().nanos as f64 / 1000.,
        )?;
        kwargs.set_item(Self::ARBITRATION_ID, self.arbitration_id)?;
        kwargs.set_item(Self::IS_EFF, self.is_extended_id)?;
        kwargs.set_item(Self::IS_RTR, self.is_remote_frame)?;
        kwargs.set_item(Self::IS_ERR, self.is_error_frame)?;
        kwargs.set_item(Self::CHANNEL, self.channel)?;
        kwargs.set_item(Self::DLC, self.data.len())?;
        kwargs.set_item(Self::DATA, self.data.clone())?;
        kwargs.set_item(Self::IS_CAN_FD, matches!(self.kind(), CanKind::FD))?;
        kwargs.set_item(
            Self::IS_RX,
            matches!(self.direction(), CanDirection::Receive),
        )?;
        kwargs.set_item(Self::IS_BRS, self.bitrate_switch)?;
        kwargs.set_item(Self::IS_ESI, self.error_state_indicator)?;

        message_class.call((), Some(&kwargs))
    }

    pub fn from_python<'py>(
        _py: pyo3::Python<'py>,
        py_message: &pyo3::Bound<'py, pyo3::PyAny>,
    ) -> pyo3::PyResult<Self> {
        use pyo3::types::PyAnyMethods;
        use rs_can::TimestampSource;

        let timestamp: f64 = py_message.getattr(Self::TIMESTAMP)?.extract()?;
        let timestamp = Timestamp {
            nanos: (timestamp * 1000.) as u128,
            source: TimestampSource::System,
        };
        let arbitration_id: u32 = py_message.getattr(Self::ARBITRATION_ID)?.extract()?;
        let is_extended_id: bool = py_message.getattr(Self::IS_EFF)?.extract()?;
        let is_remote_frame: bool = py_message.getattr(Self::IS_RTR)?.extract()?;
        let is_error_frame: bool = py_message.getattr(Self::IS_ERR)?.extract()?;
        let channel: Option<u8> = py_message.getattr(Self::CHANNEL)?.extract()?;
        let data: Vec<u8> = py_message.getattr(Self::DATA)?.extract()?;
        let is_fd: bool = py_message.getattr(Self::IS_CAN_FD)?.extract()?;
        let is_rx: bool = py_message.getattr(Self::IS_RX)?.extract()?;
        let bitrate_switch: bool = py_message.getattr(Self::IS_BRS)?.extract()?;
        let error_state_indicator: bool = py_message.getattr(Self::IS_ESI)?.extract()?;

        Ok(Self {
            timestamp: Some(timestamp),
            arbitration_id,
            is_extended_id,
            is_remote_frame,
            is_error_frame,
            channel: channel.unwrap_or(0),
            length: data.len(),
            data,
            kind: if is_fd {
                CanKind::FD
            } else {
                CanKind::Classical
            },
            direction: if is_rx {
                CanDirection::Receive
            } else {
                CanDirection::Transmit
            },
            bitrate_switch,
            error_state_indicator,
            tx_mode: Default::default(),
        })
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

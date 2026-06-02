use crate::native::can::{
    constants::{CANERR_FRAME_LENGTH, TIME_FLAG_VALID},
    ZCanFrame,
};
use rs_can::{
    can_utils, CanDirection, CanKind, Timestamp, TimestampSource, DEFAULT_PADDING,
    MAX_FD_FRAME_SIZE, MAX_FRAME_SIZE,
};
use std::ffi::{c_uchar, c_uint, c_ushort};

/// only used usbcan on linux
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub(crate) struct ZCanFrameVCI {
    pub(crate) can_id: c_uint,
    pub(crate) timestamp: c_uint,
    pub(crate) time_flag: c_uchar,
    pub(crate) tx_mode: c_uchar,
    pub(crate) rem_flag: c_uchar,
    pub(crate) ext_flag: c_uchar,
    pub(crate) can_len: c_uchar,
    pub(crate) data: [c_uchar; MAX_FRAME_SIZE],
    pub(crate) channel: c_uchar,
    #[allow(dead_code)]
    pub(crate) reserved: [c_uchar; 3 - 1], // use 1byte to channel
}

impl Into<ZCanFrame> for ZCanFrameVCI {
    fn into(self) -> ZCanFrame {
        if self.time_flag != TIME_FLAG_VALID {
            rsutil::warn!("ZCanFrameVCI - time flag is invalid")
        }
        let timestamp = Timestamp {
            nanos: self.timestamp as u128 * 1_000,
            source: TimestampSource::Hardware,
        };
        let arbitration_id = self.can_id;
        let is_extended_id = self.ext_flag > 0;
        let is_remote_frame = self.rem_flag > 0;
        let channel = self.channel;
        let length = self.can_len as usize;
        let mut data = self.data.to_vec();
        data.resize(length, Default::default());
        ZCanFrame {
            timestamp: Some(timestamp),
            arbitration_id,
            is_extended_id,
            is_remote_frame,
            is_error_frame: false,
            channel,
            length,
            data,
            kind: CanKind::Classical,
            direction: CanDirection::Receive,
            bitrate_switch: false,
            error_state_indicator: false,
            tx_mode: None,
        }
    }
}

impl From<ZCanFrame> for ZCanFrameVCI {
    fn from(msg: ZCanFrame) -> Self {
        let can_id = msg.arbitration_id;
        let timestamp = msg
            .timestamp
            .map(|t| (t.nanos / 1_000) as u32)
            .unwrap_or_default();
        let time_flag = TIME_FLAG_VALID;
        let tx_mode = msg.tx_mode();
        let rem_flag = if msg.is_remote_frame { 1 } else { 0 };
        let ext_flag = if msg.is_extended_id { 1 } else { 0 };
        let can_len = msg.length as u8;
        let channel = msg.channel;
        let mut data = msg.data;
        data.resize(MAX_FRAME_SIZE, DEFAULT_PADDING);
        Self {
            can_id,
            timestamp,
            time_flag,
            tx_mode,
            rem_flag,
            ext_flag,
            can_len,
            data: data.try_into().unwrap(),
            channel,
            reserved: Default::default(),
        }
    }
}

/// only used usbcanfd on linux
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct ZCanMsg20<const S: usize> {
    pub(crate) timestamp: c_uint,
    pub(crate) can_id: c_uint,
    /// bit31~13: reserved
    /// bit12 : /**< error state */
    /// bit11 : /**< bit-rate switch */
    /// bit10 : /**< error flag */
    /// bit9  : /**< 0-std_frame, 1-ext_frame */
    /// bit8  : /**< 0-data_frame, 1-remote_frame */
    /// bit7~4: /**< 0-CAN2.0, 1-CANFD */
    /// bit3~0: /**< TX-mode, @see ZCAN_TX_MODE */
    pub(crate) flags: c_uint,
    pub(crate) __pad: c_ushort,
    pub(crate) channel: c_uchar,
    pub(crate) can_len: c_uchar,
    pub(crate) data: [c_uchar; S],
}

impl<const S: usize> Default for ZCanMsg20<S> {
    fn default() -> Self {
        Self {
            timestamp: Default::default(),
            can_id: Default::default(),
            flags: Default::default(),
            __pad: Default::default(),
            channel: Default::default(),
            can_len: Default::default(),
            data: [Default::default(); S],
        }
    }
}

impl<const S: usize> Into<ZCanFrame> for ZCanMsg20<S> {
    fn into(self) -> ZCanFrame {
        let length = self.can_len as usize;
        let mut data = self.data.to_vec();
        data.resize(length, Default::default());
        #[allow(deprecated)]
        let kind = can_utils::can_kind_by_len(S).unwrap();
        let timestamp = Timestamp {
            nanos: self.timestamp as u128 * 1_000,
            source: TimestampSource::Hardware,
        };

        ZCanFrame {
            timestamp: Some(timestamp),
            arbitration_id: self.can_id,
            is_extended_id: (self.flags & (0x01 << 9)) > 0,
            is_remote_frame: (self.flags & (0x01 << 8)) > 0,
            is_error_frame: (self.flags & (0x01 << 10)) > 0,
            channel: self.channel,
            length,
            data,
            kind,
            direction: CanDirection::Receive,
            bitrate_switch: match kind {
                CanKind::Classical => false,
                CanKind::FD => (self.flags & (0x01 << 11)) > 0,
                CanKind::XL => todo!("XL is not supported!"),
            },
            error_state_indicator: match kind {
                CanKind::Classical => false,
                CanKind::FD => (self.flags & (0x01 << 12)) > 0,
                CanKind::XL => todo!("XL is not supported!"),
            },
            tx_mode: Some((self.flags & 0x3) as u8),
        }
    }
}

impl<const S: usize> From<ZCanFrame> for ZCanMsg20<S> {
    fn from(msg: ZCanFrame) -> Self {
        let flags = (msg.tx_mode() as u32)
            | match msg.kind {
                CanKind::Classical => 0,
                CanKind::FD => 0x01 << 4,
                CanKind::XL => todo!("XL is not supported!"),
            }
            | if msg.is_remote_frame { 0x01 << 8 } else { 0 }
            | if msg.is_extended_id { 0x01 << 9 } else { 0 }
            | if msg.is_error_frame { 0x01 << 10 } else { 0 }
            | if msg.bitrate_switch { 0x01 << 11 } else { 0 }
            | if msg.error_state_indicator {
                0x01 << 12
            } else {
                0
            };
        let timestamp = msg
            .timestamp
            .map(|t| (t.nanos / 1_000) as u32)
            .unwrap_or_default();
        let can_id = msg.arbitration_id;
        let channel = msg.channel;
        let can_len = msg.length as u8;
        let mut data = msg.data;
        data.resize(S, DEFAULT_PADDING);
        Self {
            timestamp,
            can_id,
            flags,
            __pad: Default::default(),
            channel,
            can_len,
            data: data.try_into().unwrap(),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub(crate) union ZCanFrameInner {
    pub(crate) libusbcan: ZCanFrameVCI,                // libusbcan.so
    pub(crate) libusbcanfd: ZCanMsg20<MAX_FRAME_SIZE>, // libusbcanfd.so
    pub(crate) libother: super::common::ZCanMsg20<MAX_FRAME_SIZE>,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub(crate) union ZCanFdFrameInner {
    pub(crate) libusbcanfd: ZCanMsg20<MAX_FD_FRAME_SIZE>, // libusbcanfd.so
    pub(crate) libother: super::common::ZCanMsg20<MAX_FD_FRAME_SIZE>,
}

/// only used usbcanfd on linux
pub(crate) type ZCanChlErrInfo = ZCanMsg20<CANERR_FRAME_LENGTH>;

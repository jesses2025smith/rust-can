use crate::native::can::{frame::common::ZCanMsg20, ZCanFrame};
use rs_can::{Timestamp, TimestampSource, MAX_FD_FRAME_SIZE, MAX_FRAME_SIZE};
use std::ffi::{c_uint, c_ulonglong};

#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub(crate) struct ZCanFrameTx<const S: usize> {
    pub(crate) frame: ZCanMsg20<S>,
    pub(crate) tx_mode: c_uint, // ZCanTxMode
}

impl<const S: usize> From<ZCanFrame> for ZCanFrameTx<S> {
    fn from(msg: ZCanFrame) -> Self {
        let tx_mode = msg.tx_mode() as u32;
        let frame = msg.into();
        Self { frame, tx_mode }
    }
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub(crate) struct ZCanFrameRx<const S: usize> {
    pub(crate) frame: ZCanMsg20<S>,
    pub(crate) timestamp: c_ulonglong,
}

impl<const S: usize> Into<ZCanFrame> for ZCanFrameRx<S> {
    fn into(self) -> ZCanFrame {
        let timestamp = Timestamp {
            nanos: self.timestamp as u128 * 1_000,
            source: TimestampSource::Hardware,
        };
        let mut msg: ZCanFrame = self.frame.into();
        msg.timestamp = Some(timestamp);
        msg
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub(crate) union ZCanFrameInner {
    pub(crate) tx: ZCanFrameTx<MAX_FRAME_SIZE>,
    pub(crate) rx: ZCanFrameRx<MAX_FRAME_SIZE>,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub(crate) union ZCanFdFrameInner {
    pub(crate) tx: ZCanFrameTx<MAX_FD_FRAME_SIZE>,
    pub(crate) rx: ZCanFrameRx<MAX_FD_FRAME_SIZE>,
}

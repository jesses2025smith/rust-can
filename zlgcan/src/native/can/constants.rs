#![allow(unused)]

pub(crate) const BITRATE_CFG_FILENAME: &str = "bitrate.cfg.yaml";
pub(crate) const TIMING0: &str = "timing0";
pub(crate) const TIMING1: &str = "timing1";
pub(crate) const TSEG1: &str = "tseg1"; // Time Segment 1
pub(crate) const TSEG2: &str = "tseg2"; // Time Segment 2
pub(crate) const SJW: &str = "sjw"; // Synchronization Jump Width
pub(crate) const SMP: &str = "smp"; // Sampling specifies
pub(crate) const BRP: &str = "brp"; // BaudRate Pre-scale

// pub const CAN_FRAME_LENGTH: usize = 8;
pub(crate) const CANERR_FRAME_LENGTH: usize = 8;
// pub const CANFD_FRAME_LENGTH: usize = 64;
pub(crate) const TIME_FLAG_VALID: u8 = 1;

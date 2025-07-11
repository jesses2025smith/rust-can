use crate::native::can::{ZCanChlMode, ZCanChlType};
use rs_can::CanError;
use std::ffi::c_uint;

/// Linux USBCANFD
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct ZCanFdChlCfgInner {
    #[doc = "< clock(Hz)"]
    clk: c_uint,
    #[doc = "< bit0-normal/listen_only, bit1-ISO/BOSCH"]
    mode: c_uint,
    aset: super::ZCanFdChlCfgSet,
    dset: super::ZCanFdChlCfgSet,
}

impl ZCanFdChlCfgInner {
    #[inline(always)]
    pub fn new(
        can_type: ZCanChlType,
        mode: ZCanChlMode,
        clock: u32,
        aset: super::ZCanFdChlCfgSet,
        dset: super::ZCanFdChlCfgSet,
    ) -> Self {
        let mut mode = mode as u32;
        if let ZCanChlType::CANFD_NON_ISO = can_type {
            mode |= 2;
        }
        Self {
            clk: clock,
            mode,
            aset,
            dset,
        }
    }
}
/// end of Linux USBCANFD

#[repr(C)]
#[derive(Copy, Clone)]
pub union ZCanChlCfgUnion {
    pub(crate) can: super::common::ZCanChlCfgInner,
    pub(crate) canfd: super::common::ZCanFdChlCfgInner,
}

pub(crate) fn get_fd_cfg(
    can_type: ZCanChlType,
    mode: ZCanChlMode,
    bitrate: u32,
    dbitrate: Option<u32>,
    ctx: &super::BitrateCtx,
) -> Result<self::ZCanFdChlCfgInner, CanError> {
    let (aset, dset) = super::get_fd_set(bitrate, dbitrate, ctx)?;
    let clock = ctx
        .clock
        .ok_or(CanError::other_error("`clock` is not configured in file!"))?;

    Ok(self::ZCanFdChlCfgInner::new(
        can_type, mode, clock, aset, dset,
    ))
}

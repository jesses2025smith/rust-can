use std::ffi::c_uint;
use rs_can::CanError;
use crate::can::{common::BitrateCtx, ZCanChlMode, ZCanChlType};

use super::ZCanFdChlCfgSet;

/// Linux USBCANFD
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct ZCanFdChlCfgInner {
    #[doc = "< clock(Hz)"]
    clk: c_uint,
    #[doc = "< bit0-normal/listen_only, bit1-ISO/BOSCH"]
    mode: c_uint,
    aset: ZCanFdChlCfgSet,
    dset: ZCanFdChlCfgSet,
}
impl ZCanFdChlCfgInner {
    #[inline(always)]
    pub fn new(
        can_type: ZCanChlType,
        mode: ZCanChlMode,
        clock: u32,
        aset: ZCanFdChlCfgSet,
        dset: ZCanFdChlCfgSet
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
    can_type: u8,
    mode: u8,
    bitrate: u32,
    dbitrate: Option<u32>,
    ctx: &BitrateCtx,
) -> Result<self::ZCanFdChlCfgInner, CanError> {
    let (aset, dset) = super::get_fd_set(bitrate, dbitrate, ctx)?;
    let clock = ctx.clock
        .ok_or(CanError::other_error("`clock` is not configured in file!"))?;
    let can_type = ZCanChlType::try_from(can_type)?;

    Ok(self::ZCanFdChlCfgInner::new(
        can_type,
        ZCanChlMode::try_from(mode)?,
        clock,
        aset,
        dset,
    ))
}

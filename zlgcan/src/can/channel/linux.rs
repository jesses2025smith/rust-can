use std::collections::HashMap;
use std::ffi::{c_uchar, c_uint, c_ushort};
use rs_can::CanError;
use crate::can::{common::BitrateCtx, ZCanChlMode, ZCanChlType, constant::{BRP, SJW, SMP, TSEG1, TSEG2}};

/// Linux USBCANFD
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ZCanFdChlCfgSet {
    tseg1: c_uchar,
    tseg2: c_uchar,
    sjw: c_uchar,
    smp: c_uchar,
    brp: c_ushort,
}

impl TryFrom<&HashMap<String, u32>> for ZCanFdChlCfgSet {
    type Error = CanError;
    fn try_from(value: &HashMap<String, u32>) -> Result<Self, Self::Error> {
        let &tseg1 = value.get(TSEG1)
            .ok_or(CanError::OtherError(format!("`{}` is not configured in file!", TSEG1)))?;
        let &tseg2 = value.get(TSEG2)
            .ok_or(CanError::OtherError(format!("ZLGCAN - `{}` is not configured in file!", TSEG2)))?;
        let &sjw = value.get(SJW)
            .ok_or(CanError::OtherError(format!("ZLGCAN - `{}` is not configured in file!", SJW)))?;
        let &smp = value.get(SMP)
            .ok_or(CanError::OtherError(format!("ZLGCAN - `{}` is not configured in file!", SMP)))?;
        let &brp = value.get(BRP)
            .ok_or(CanError::OtherError(format!("ZLGCAN - `{}` is not configured in file!", BRP)))?;

        Ok(Self::new(tseg1, tseg2, sjw, smp, brp))
    }
}

impl ZCanFdChlCfgSet {
    #[inline(always)]
    pub fn new(tseg1: u32, tseg2: u32, sjw: u32, smp: u32, brp: u32) -> Self {
        Self {
            tseg1: tseg1 as u8,
            tseg2: tseg2 as u8,
            sjw: sjw as u8,
            smp: smp as u8,
            brp: brp as u16,
        }
    }
    /// Only used for USBCANFD-800U
    #[inline(always)]
    pub fn get_timing(&self) -> u32 {
        (self.brp as u32) << 22
            | (self.sjw as u32 & 0x7f) << 15
            | (self.tseg2 as u32 & 0x7f) << 8
            | (self.tseg1 as u32)
    }
}

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

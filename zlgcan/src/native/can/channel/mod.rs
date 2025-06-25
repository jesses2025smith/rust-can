pub(crate) mod common;
pub use common::{ZCanChlStatus, ZCanChlType, ZCanChlMode};

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub(crate) use linux::*;
#[cfg(target_os = "windows")]
mod win;
#[cfg(target_os = "windows")]
pub(crate) use win::*;

use std::{collections::HashMap, ffi::{c_uchar, c_uint, c_ushort}};
use rs_can::{CanError, ChannelConfig};
use crate::{
    constants,
    native::{
        device::ZCanDeviceType,
        can::{
            common::BitrateCtx, ZCanFilterType,
            constants::{BRP, SJW, SMP, TSEG1, TSEG2}
        }
    }
};

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct ZCanFdChlCfgSet {
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

#[repr(C)]
pub(crate) struct ZCanChlCfg {
    can_type: c_uint,
    cfg: ZCanChlCfgUnion,
}

impl ZCanChlCfg {
    #[inline(always)]
    pub fn new(
        dev_type: ZCanDeviceType,
        can_type: ZCanChlType,
        ctx: &BitrateCtx,
        cfg: &ChannelConfig,
    ) -> Result<Self, CanError> {
        if dev_type.canfd_support() {
            let (aset, dset) = get_fd_set(cfg.bitrate(), cfg.dbitrate(), ctx)?;
            Ok(Self {
                can_type: can_type as u32,
                cfg: ZCanChlCfgUnion {
                    canfd: common::ZCanFdChlCfgInner::new(
                        cfg.get_other::<ZCanChlMode>(constants::CHANNEL_MODE)?
                            .unwrap_or(ZCanChlMode::Normal),
                        aset.get_timing(),  // timing0 and timing1 ignored expect USBCANFD_800U
                        dset.get_timing(),
                        cfg.get_other::<ZCanFilterType>(constants::FILTER_TYPE)?
                            .unwrap_or(ZCanFilterType::default()),
                        cfg.get_other::<u32>(constants::ACC_CODE)?,
                        cfg.get_other::<u32>(constants::ACC_MASK)?,
                        cfg.get_other::<u32>(constants::BRP)?,
                    )?
                }
            })
        }
        else {
            Ok(Self {
                can_type: ZCanChlType::CAN as u32,
                cfg: ZCanChlCfgUnion {
                    can: common::ZCanChlCfgInner::try_from_with(ctx, cfg)?
                }
            })
        }
    }
}

pub(crate) fn get_fd_set(
    bitrate: u32,
    dbitrate: Option<u32>,
    ctx: &BitrateCtx,
) -> Result<(ZCanFdChlCfgSet, ZCanFdChlCfgSet), CanError> {
    let bitrate_ctx = &ctx.bitrate;
    let dbitrate_ctx = &ctx.data_bitrate;
    let aset = bitrate_ctx
        .get(&bitrate.to_string())
        .ok_or(CanError::OtherError(format!("bitrate `{}` is not configured in file!", bitrate)))?;
    let dset=
        match dbitrate {
            Some(v) => {    // dbitrate is not None
                match dbitrate_ctx {
                    Some(ctx) => {  // dbitrate context is not None
                        match ctx.get(&v.to_string()) {
                            Some(value) => Ok(value),
                            None => Err(CanError::OtherError(format!("data bitrate `{}` is not configured in file!", v))),
                        }
                    },
                    None => {   // dbitrate context is None
                        match bitrate_ctx.get(&v.to_string()) {
                            Some(value) => Ok(value),
                            None => Err(CanError::OtherError(format!("data bitrate `{}` is not configured in file!", v))),
                        }
                    }
                }
            },
            None => {   // dbitrate is None
                match dbitrate_ctx {
                    Some(ctx) => {
                        match ctx.get(&bitrate.to_string()) {
                            Some(value) => Ok(value),
                            None => Ok(aset),
                        }
                    },
                    None => Ok(aset),
                }
            }
        }?;

    Ok((ZCanFdChlCfgSet::try_from(aset)?, ZCanFdChlCfgSet::try_from(dset)?))
}

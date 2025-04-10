pub(crate) mod common;
pub use common::{ZCanChlStatus, ZCanChlType, ZCanChlMode};

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub(crate) use linux::*;
#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub(crate) use windows::*;

use std::ffi::c_uint;
use rs_can::{CanError, ChannelConfig};
use crate::can::{common::BitrateCtx, ZCanFilterType};
use crate::{ACC_CODE, ACC_MASK, BRP, CHANNEL_MODE, FILTER_TYPE};
use crate::device::ZCanDeviceType;

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
                        cfg.get_other::<u8>(CHANNEL_MODE)?
                            .unwrap_or(ZCanChlMode::Normal as u8),
                        aset.get_timing(),  // timing0 and timing1 ignored expect USBCANFD_800U
                        dset.get_timing(),
                        cfg.get_other::<u8>(FILTER_TYPE)?
                            .unwrap_or(ZCanFilterType::default() as u8),
                        cfg.get_other::<u32>(ACC_CODE)?,
                        cfg.get_other::<u32>(ACC_MASK)?,
                        cfg.get_other::<u32>(BRP)?,
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


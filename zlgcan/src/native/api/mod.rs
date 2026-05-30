#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub(crate) use linux::*;
#[cfg(target_os = "windows")]
mod win;
#[cfg(target_os = "windows")]
pub(crate) use win::*;

use crate::native::{
    can::*,
    cloud::*,
    device::{CmdPath, IProperty, ZCanDeviceType, ZDeviceInfo},
    lin::*,
};
use rs_can::{CanError, CanResult, ChannelConfig};
use std::ffi::{c_char, c_void};

#[derive(Debug, Copy, Clone)]
pub(crate) struct ZDeviceContext {
    pub(crate) dev_type: ZCanDeviceType,
    pub(crate) dev_idx: u32,
    pub(crate) dev_hdl: Option<u32>,
    #[allow(unused)]
    pub(crate) is_derive: bool,
}

impl ZDeviceContext {
    pub fn new(dev_type: ZCanDeviceType, dev_idx: u32, is_derive: bool) -> Self {
        Self {
            dev_type,
            dev_idx,
            dev_hdl: Default::default(),
            is_derive,
        }
    }
    #[inline(always)]
    pub fn device_handler(&self) -> CanResult<u32> {
        self.dev_hdl
            .ok_or(CanError::other_error("device is not initialized!"))
    }
}

#[derive(Debug, Copy, Clone)]
pub(crate) struct ZChannelContext {
    pub(crate) device: ZDeviceContext,
    pub(crate) channel: u8,
    pub(crate) chl_hdl: Option<u32>,
    pub(crate) timestamp: u64,
}

impl ZChannelContext {
    pub fn new(device: ZDeviceContext, channel: u8) -> Self {
        Self {
            device,
            channel,
            chl_hdl: Default::default(),
            timestamp: Default::default(),
        }
    }

    #[inline(always)]
    pub fn device_handler(&self) -> CanResult<u32> {
        self.device.device_handler()
    }

    #[inline(always)]
    pub fn channel_handler(&self) -> CanResult<u32> {
        self.chl_hdl
            .ok_or(CanError::other_error("channel is not initialized!"))
    }
}

#[allow(unused_variables, dead_code)]
pub trait ZDeviceApi {
    fn open(&self, context: &mut ZDeviceContext) -> CanResult<()>;
    fn close(&self, context: &ZDeviceContext) -> CanResult<()>;
    fn read_device_info(&self, context: &ZDeviceContext) -> CanResult<ZDeviceInfo>;
    fn is_online(&self, context: &ZDeviceContext) -> CanResult<bool> {
        Err(CanError::NotSupportedError)
    }
    fn get_property(&self, context: &ZChannelContext) -> CanResult<IProperty> {
        Err(CanError::NotSupportedError)
    }
    fn release_property(&self, p: &IProperty) -> CanResult<()> {
        Err(CanError::NotSupportedError)
    }
    fn set_reference(
        &self,
        context: &ZChannelContext,
        cmd_path: &CmdPath,
        value: *const c_void,
    ) -> CanResult<()> {
        Err(CanError::NotSupportedError)
    }
    fn get_reference(
        &self,
        context: &ZChannelContext,
        cmd_path: &CmdPath,
        value: *mut c_void,
    ) -> CanResult<()> {
        Err(CanError::NotSupportedError)
    }
    fn set_value(
        &self,
        context: &ZChannelContext,
        cmd_path: &CmdPath,
        value: *const c_void,
    ) -> CanResult<()> {
        Err(CanError::NotSupportedError)
    }
    fn get_value(&self, context: &ZChannelContext, cmd_path: &CmdPath) -> CanResult<*const c_void> {
        Err(CanError::NotSupportedError)
    }
    fn set_values(
        &self,
        context: &ZChannelContext,
        values: Vec<(CmdPath, *const c_char)>,
    ) -> CanResult<()> {
        Err(CanError::NotSupportedError)
    }
    fn get_values(&self, context: &ZChannelContext, paths: Vec<CmdPath>) -> CanResult<Vec<String>> {
        Err(CanError::NotSupportedError)
    }
    fn debug(&self, level: u32) -> CanResult<()> {
        Err(CanError::NotSupportedError)
    }
}

#[allow(unused_variables)]
pub trait ZCanApi {
    fn init_can_chl(
        &self,
        libpath: &str,
        context: &mut ZChannelContext,
        cfg: &ChannelConfig,
    ) -> CanResult<()>;
    fn reset_can_chl(&self, context: &ZChannelContext) -> CanResult<()>;
    fn read_can_chl_status(&self, context: &ZChannelContext) -> CanResult<ZCanChlStatus>;
    fn read_can_chl_error(&self, context: &ZChannelContext) -> CanResult<ZCanChlError>;
    fn clear_can_buffer(&self, context: &ZChannelContext) -> CanResult<()>;
    fn get_can_num(&self, context: &ZChannelContext, can_type: ZCanFrameType) -> CanResult<u32>;
    fn receive_can(
        &self,
        context: &ZChannelContext,
        size: u32,
        timeout: u32,
    ) -> CanResult<Vec<ZCanFrame>>;
    fn transmit_can(&self, context: &ZChannelContext, frames: Vec<ZCanFrame>) -> CanResult<u32>;
    fn receive_canfd(
        &self,
        context: &ZChannelContext,
        size: u32,
        timeout: u32,
    ) -> CanResult<Vec<ZCanFrame>> {
        Err(CanError::NotSupportedError)
    }
    fn transmit_canfd(&self, context: &ZChannelContext, frames: Vec<ZCanFrame>) -> CanResult<u32> {
        Err(CanError::NotSupportedError)
    }
}

#[allow(unused_variables, dead_code)]
pub trait ZLinApi {
    fn init_lin_chl(&self, context: &mut ZChannelContext, cfg: &ZLinChlCfg) -> CanResult<()> {
        Err(CanError::NotSupportedError)
    }
    fn reset_lin_chl(&self, context: &ZChannelContext) -> CanResult<()> {
        Err(CanError::NotSupportedError)
    }
    fn clear_lin_buffer(&self, context: &ZChannelContext) -> CanResult<()> {
        Err(CanError::NotSupportedError)
    }
    fn get_lin_num(&self, context: &ZChannelContext) -> CanResult<u32> {
        Err(CanError::NotSupportedError)
    }
    fn receive_lin(
        &self,
        context: &ZChannelContext,
        size: u32,
        timeout: u32,
    ) -> CanResult<Vec<ZLinFrame>> {
        Err(CanError::NotSupportedError)
    }
    fn transmit_lin(&self, context: &ZChannelContext, frames: Vec<ZLinFrame>) -> CanResult<u32> {
        Err(CanError::NotSupportedError)
    }
    fn set_lin_subscribe(
        &self,
        context: &ZChannelContext,
        cfg: Vec<ZLinSubscribe>,
    ) -> CanResult<()> {
        Err(CanError::NotSupportedError)
    }
    fn set_lin_publish(&self, context: &ZChannelContext, cfg: Vec<ZLinPublish>) -> CanResult<()> {
        Err(CanError::NotSupportedError)
    }
    fn wakeup_lin(&self, context: &ZChannelContext) -> CanResult<()> {
        Err(CanError::NotSupportedError)
    }
    fn set_lin_publish_ex(
        &self,
        context: &ZChannelContext,
        cfg: Vec<ZLinPublishEx>,
    ) -> CanResult<()> {
        Err(CanError::NotSupportedError)
    }
    #[deprecated(since = "0.1.0", note = "This method is deprecated!")]
    fn set_lin_slave_msg(&self, context: &ZChannelContext, msg: Vec<ZLinFrame>) -> CanResult<()> {
        Err(CanError::NotSupportedError)
    }
    #[deprecated(since = "0.1.0", note = "This method is deprecated!")]
    fn clear_lin_slave_msg(&self, context: &ZChannelContext, pids: Vec<u8>) -> CanResult<()> {
        Err(CanError::NotSupportedError)
    }
}

#[allow(unused_variables, dead_code)]
pub trait ZCloudApi {
    fn set_server(&self, server: ZCloudServerInfo) -> CanResult<()> {
        Err(CanError::NotSupportedError)
    }
    fn connect_server(&self, username: &str, password: &str) -> CanResult<()> {
        Err(CanError::NotSupportedError)
    }
    fn is_connected_server(&self) -> CanResult<bool> {
        Err(CanError::NotSupportedError)
    }
    fn disconnect_server(&self) -> CanResult<()> {
        Err(CanError::NotSupportedError)
    }
    fn get_userdata(&self, update: i32) -> CanResult<ZCloudUserData> {
        Err(CanError::NotSupportedError)
    }
    fn receive_gps(
        &self,
        context: &ZDeviceContext,
        size: u32,
        timeout: u32,
    ) -> CanResult<Vec<ZCloudGpsFrame>> {
        Err(CanError::NotSupportedError)
    }
}

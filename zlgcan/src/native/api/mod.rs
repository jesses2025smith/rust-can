
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub(crate) use linux::*;
#[cfg(target_os = "windows")]
mod win;
#[cfg(target_os = "windows")]
pub(crate) use win::*;

use rs_can::{CanError, ChannelConfig};
use std::ffi::{c_char, c_void};
use crate::native::{can::*, cloud::*, device::{CmdPath, IProperty, ZDeviceInfo, ZCanDeviceType}, lin::*};

#[derive(Debug, Copy, Clone)]
pub(crate) struct ZDeviceContext {
    pub(crate) dev_type: ZCanDeviceType,
    pub(crate) dev_idx: u32,
    pub(crate) dev_hdl: Option<u32>,
    #[allow(unused)]
    pub(crate) is_derive: bool,
}

impl ZDeviceContext {
    pub fn new(
        dev_type: ZCanDeviceType,
        dev_idx: u32,
        is_derive: bool,
    ) -> Self {
        Self {
            dev_type,
            dev_idx,
            dev_hdl: Default::default(),
            is_derive
        }
    }
    #[inline(always)]
    pub fn device_handler(&self) -> Result<u32, CanError> {
        self.dev_hdl.ok_or(CanError::other_error("device is not initialized!"))
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
    pub fn new(
        device: ZDeviceContext,
        channel: u8,
    ) -> Self {
        Self {
            device,
            channel,
            chl_hdl: Default::default(),
            timestamp: Default::default(),
        }
    }

    #[inline(always)]
    pub fn device_handler(&self) -> Result<u32, CanError> {
        self.device.device_handler()
    }

    #[inline(always)]
    pub fn channel_handler(&self) -> Result<u32, CanError> {
        self.chl_hdl.ok_or(CanError::other_error("channel is not initialized!"))
    }
}

#[allow(unused_variables, dead_code)]
pub trait ZDeviceApi {
    fn open(&self, context: &mut ZDeviceContext) -> Result<(), CanError>;
    fn close(&self, context: &ZDeviceContext) -> Result<(), CanError>;
    fn read_device_info(&self, context: &ZDeviceContext) -> Result<ZDeviceInfo, CanError>;
    fn is_online(&self, context: &ZDeviceContext) -> Result<bool, CanError> {
        Err(CanError::NotSupportedError)
    }
    fn get_property(&self, context: &ZChannelContext) -> Result<IProperty, CanError> {
        Err(CanError::NotSupportedError)
    }
    fn release_property(&self, p: &IProperty) -> Result<(), CanError> {
        Err(CanError::NotSupportedError)
    }
    fn set_reference(&self, context: &ZChannelContext, cmd_path: &CmdPath, value: *const c_void) -> Result<(), CanError> {
        Err(CanError::NotSupportedError)
    }
    fn get_reference(&self, context: &ZChannelContext, cmd_path: &CmdPath, value: *mut c_void) -> Result<(), CanError> {
        Err(CanError::NotSupportedError)
    }
    fn set_value(&self, context: &ZChannelContext, cmd_path: &CmdPath, value: *const c_void) -> Result<(), CanError> {
        Err(CanError::NotSupportedError)
    }
    fn get_value(&self, context: &ZChannelContext, cmd_path: &CmdPath) -> Result<*const c_void, CanError> {
        Err(CanError::NotSupportedError)
    }
    fn set_values(&self, context: &ZChannelContext, values: Vec<(CmdPath, *const c_char)>) -> Result<(), CanError> {
        Err(CanError::NotSupportedError)
    }
    fn get_values(&self, context: &ZChannelContext, paths: Vec<CmdPath>) -> Result<Vec<String>, CanError> {
        Err(CanError::NotSupportedError)
    }
    fn debug(&self, level: u32) -> Result<(), CanError> {
        Err(CanError::NotSupportedError)
    }
}

#[allow(unused_variables)]
pub trait ZCanApi {
    fn init_can_chl(&self, libpath: &str, context: &mut ZChannelContext, cfg: &ChannelConfig) -> Result<(), CanError>;
    fn reset_can_chl(&self, context: &ZChannelContext) -> Result<(), CanError>;
    fn read_can_chl_status(&self, context: &ZChannelContext) -> Result<ZCanChlStatus, CanError>;
    fn read_can_chl_error(&self, context: &ZChannelContext) -> Result<ZCanChlError, CanError>;
    fn clear_can_buffer(&self, context: &ZChannelContext) -> Result<(), CanError>;
    fn get_can_num(&self, context: &ZChannelContext, can_type: ZCanFrameType) -> Result<u32, CanError>;
    fn receive_can(&self, context: &ZChannelContext, size: u32, timeout: u32) -> Result<Vec<CanMessage>, CanError>;
    fn transmit_can(&self, context: &ZChannelContext, frames: Vec<CanMessage>) -> Result<u32, CanError>;
    fn receive_canfd(&self, context: &ZChannelContext, size: u32, timeout: u32) -> Result<Vec<CanMessage>, CanError> {
        Err(CanError::NotSupportedError)
    }
    fn transmit_canfd(&self, context: &ZChannelContext, frames: Vec<CanMessage>) -> Result<u32, CanError> {
        Err(CanError::NotSupportedError)
    }
}

#[allow(unused_variables, dead_code)]
pub trait ZLinApi {
    fn init_lin_chl(&self, context: &mut ZChannelContext, cfg: &ZLinChlCfg) -> Result<(), CanError> {
        Err(CanError::NotSupportedError)
    }
    fn reset_lin_chl(&self, context: &ZChannelContext) -> Result<(), CanError> {
        Err(CanError::NotSupportedError)
    }
    fn clear_lin_buffer(&self, context: &ZChannelContext) -> Result<(), CanError> {
        Err(CanError::NotSupportedError)
    }
    fn get_lin_num(&self, context: &ZChannelContext) -> Result<u32, CanError> {
        Err(CanError::NotSupportedError)
    }
    fn receive_lin(
        &self,
        context: &ZChannelContext,
        size: u32,
        timeout: u32,
    ) -> Result<Vec<ZLinFrame>, CanError> {
        Err(CanError::NotSupportedError)
    }
    fn transmit_lin(&self, context: &ZChannelContext, frames: Vec<ZLinFrame>) -> Result<u32, CanError> {
        Err(CanError::NotSupportedError)
    }
    fn set_lin_subscribe(&self, context: &ZChannelContext, cfg: Vec<ZLinSubscribe>)-> Result<(), CanError> {
        Err(CanError::NotSupportedError)
    }
    fn set_lin_publish(&self, context: &ZChannelContext, cfg: Vec<ZLinPublish>) -> Result<(), CanError> {
        Err(CanError::NotSupportedError)
    }
    fn wakeup_lin(&self, context: &ZChannelContext) -> Result<(), CanError> {
        Err(CanError::NotSupportedError)
    }
    fn set_lin_publish_ex(&self, context: &ZChannelContext, cfg: Vec<ZLinPublishEx>) -> Result<(), CanError> {
        Err(CanError::NotSupportedError)
    }
    #[deprecated(since="0.1.0", note="This method is deprecated!")]
    fn set_lin_slave_msg(&self, context: &ZChannelContext, msg: Vec<ZLinFrame>) -> Result<(), CanError> {
        Err(CanError::NotSupportedError)
    }
    #[deprecated(since="0.1.0", note="This method is deprecated!")]
    fn clear_lin_slave_msg(&self, context: &ZChannelContext, pids: Vec<u8>) -> Result<(), CanError> {
        Err(CanError::NotSupportedError)
    }
}

#[allow(unused_variables, dead_code)]
pub trait ZCloudApi {
    fn set_server(&self, server: ZCloudServerInfo) -> Result<(), CanError> {
        Err(CanError::NotSupportedError)
    }
    fn connect_server(&self, username: &str, password: &str) -> Result<(), CanError> {
        Err(CanError::NotSupportedError)
    }
    fn is_connected_server(&self) -> Result<bool, CanError> {
        Err(CanError::NotSupportedError)
    }
    fn disconnect_server(&self) -> Result<(), CanError> {
        Err(CanError::NotSupportedError)
    }
    fn get_userdata(&self, update: i32) -> Result<ZCloudUserData, CanError> {
        Err(CanError::NotSupportedError)
    }
    fn receive_gps(
        &self,
        context: &ZDeviceContext,
        size: u32,
        timeout: u32
    ) -> Result<Vec<ZCloudGpsFrame>, CanError> {
        Err(CanError::NotSupportedError)
    }
}

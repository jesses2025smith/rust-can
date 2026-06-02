#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::ZDriver;
#[cfg(target_os = "windows")]
mod win;
#[cfg(target_os = "windows")]
pub use win::ZDriver;

use crate::native::{
    api::{ZChannelContext, ZDeviceContext},
    can::{ZCanChlError, ZCanChlStatus, ZCanFrame, ZCanFrameType},
    cloud::{ZCloudGpsFrame, ZCloudServerInfo, ZCloudUserData},
    device::{DeriveInfo, ZCanDeviceType, ZDeviceInfo},
    lin::{ZLinChlCfg, ZLinFrame, ZLinPublish, ZLinPublishEx, ZLinSubscribe},
};
use rs_can::{CanError, CanResult, ChannelConfig};
use std::collections::HashMap;

#[derive(Debug, Clone)]
#[repr(C)]
pub(crate) struct Handler {
    pub(crate) device: ZDeviceContext,
    pub(crate) info: ZDeviceInfo,
    pub(crate) cans: HashMap<u8, ZChannelContext>,
    pub(crate) lins: HashMap<u8, ZChannelContext>,
}

impl Handler {
    pub fn new(device: ZDeviceContext, info: ZDeviceInfo) -> Self {
        Self {
            device,
            info,
            cans: Default::default(),
            lins: Default::default(),
        }
    }
    #[inline(always)]
    pub fn device_context(&self) -> &ZDeviceContext {
        &self.device
    }
    #[inline(always)]
    pub fn device_info(&self) -> &ZDeviceInfo {
        &self.info
    }
    #[inline(always)]
    pub fn can_channels(&self) -> &HashMap<u8, ZChannelContext> {
        &self.cans
    }
    #[inline(always)]
    pub fn lin_channels(&self) -> &HashMap<u8, ZChannelContext> {
        &self.lins
    }
    #[inline(always)]
    pub fn add_can(&mut self, channel: u8, context: ZChannelContext) {
        self.cans.insert(channel, context);
    }
    #[inline(always)]
    pub fn find_can(&self, channel: u8) -> Option<&ZChannelContext> {
        self.cans.get(&channel)
    }
    #[inline(always)]
    pub fn remove_can(&mut self, channel: u8) {
        self.cans.remove(&channel);
    }
    #[inline(always)]
    pub fn add_lin(&mut self, channel: u8, handler: ZChannelContext) {
        self.lins.insert(channel, handler);
    }
    #[inline(always)]
    pub fn find_lin(&self, channel: u8) -> Option<&ZChannelContext> {
        self.lins.get(&channel)
    }
    #[inline(always)]
    pub fn remove_lin(&mut self, channel: u8) {
        self.lins.remove(&channel);
    }
}

impl ZDriver {
    pub(crate) fn device_handler<C, T>(&self, callback: C) -> CanResult<T>
    where
        C: FnOnce(&Handler) -> CanResult<T>,
    {
        match &self.handler {
            Some(v) => callback(v),
            None => Err(CanError::device_not_opened()),
        }
    }

    #[inline(always)]
    pub(crate) fn can_handler<C, T>(&self, channel: u8, callback: C) -> CanResult<T>
    where
        C: FnOnce(&ZChannelContext) -> CanResult<T>,
    {
        self.device_handler(|hdl| -> CanResult<T> {
            match hdl.find_can(channel) {
                Some(context) => callback(context),
                None => Err(CanError::channel_not_opened(channel)),
            }
        })
    }

    #[inline(always)]
    pub(crate) fn lin_handler<C, T>(&self, channel: u8, callback: C) -> CanResult<T>
    where
        C: FnOnce(&ZChannelContext) -> CanResult<T>,
    {
        self.device_handler(|hdl| -> CanResult<T> {
            match hdl.lin_channels().get(&channel) {
                Some(chl) => callback(chl),
                None => Err(CanError::channel_not_opened(channel)),
            }
        })
    }
}

#[allow(unused_variables)]
pub trait ZDevice {
    fn native(
        libpath: String,
        dev_type: ZCanDeviceType,
        dev_idx: u32,
        derive: Option<DeriveInfo>,
    ) -> CanResult<Self>
    where
        Self: Sized;
    fn device_type(&self) -> ZCanDeviceType;
    fn device_index(&self) -> u32;
    fn open(&mut self) -> CanResult<()>;
    fn close(&mut self);
    fn device_info(&self) -> CanResult<&ZDeviceInfo>;
    fn is_derive_device(&self) -> bool;
    fn is_online(&self) -> CanResult<bool> {
        Err(CanError::NotSupportedError)
    }
    fn timestamp(&self, channel: u8) -> CanResult<u64>;
}

#[allow(unused_variables)]
pub trait ZCan {
    fn init_can_chl(&mut self, channel: u8, cfg: &ChannelConfig) -> CanResult<()>;
    fn reset_can_chl(&mut self, channel: u8) -> CanResult<()>;
    // fn resistance_state(&self, dev_idx: u32, channel: u8) -> CanResult<()>;
    fn read_can_chl_status(&self, channel: u8) -> CanResult<ZCanChlStatus>;
    fn read_can_chl_error(&self, channel: u8) -> CanResult<ZCanChlError>;
    fn clear_can_buffer(&self, channel: u8) -> CanResult<()>;
    fn get_can_num(&self, channel: u8, can_type: ZCanFrameType) -> CanResult<u32>;
    fn receive_can(
        &self,
        channel: u8,
        size: u32,
        timeout: Option<u32>,
    ) -> CanResult<Vec<ZCanFrame>>;
    fn transmit_can(&self, channel: u8, frames: Vec<ZCanFrame>) -> CanResult<u32>;
    fn receive_canfd(
        &self,
        channel: u8,
        size: u32,
        timeout: Option<u32>,
    ) -> CanResult<Vec<ZCanFrame>> {
        Err(CanError::NotSupportedError)
    }
    fn transmit_canfd(&self, channel: u8, frames: Vec<ZCanFrame>) -> CanResult<u32> {
        Err(CanError::NotSupportedError)
    }
}

#[allow(unused_variables)]
pub trait ZLin {
    fn init_lin_chl(&mut self, channel: u8, cfg: ZLinChlCfg) -> CanResult<()> {
        Err(CanError::NotSupportedError)
    }
    fn reset_lin_chl(&mut self, channel: u8) -> CanResult<()> {
        Err(CanError::NotSupportedError)
    }
    fn clear_lin_buffer(&self, channel: u8) -> CanResult<()> {
        Err(CanError::NotSupportedError)
    }
    fn get_lin_num(&self, channel: u8) -> CanResult<u32> {
        Err(CanError::NotSupportedError)
    }
    fn receive_lin(
        &self,
        channel: u8,
        size: u32,
        timeout: Option<u32>,
    ) -> CanResult<Vec<ZLinFrame>> {
        Err(CanError::NotSupportedError)
    }
    fn transmit_lin(&self, channel: u8, frames: Vec<ZLinFrame>) -> CanResult<u32> {
        Err(CanError::NotSupportedError)
    }
    fn set_lin_subscribe(&self, channel: u8, cfg: Vec<ZLinSubscribe>) -> CanResult<()> {
        Err(CanError::NotSupportedError)
    }
    fn set_lin_publish(&self, channel: u8, cfg: Vec<ZLinPublish>) -> CanResult<()> {
        Err(CanError::NotSupportedError)
    }
    fn set_lin_publish_ext(&self, channel: u8, cfg: Vec<ZLinPublishEx>) -> CanResult<()> {
        Err(CanError::NotSupportedError)
    }
    fn wakeup_lin(&self, channel: u8) -> CanResult<()> {
        Err(CanError::NotSupportedError)
    }
    #[deprecated(since = "0.1.0", note = "This method is deprecated!")]
    fn set_lin_slave_msg(&self, channel: u8, msg: Vec<ZLinFrame>) -> CanResult<()> {
        Err(CanError::NotSupportedError)
    }
    #[deprecated(since = "0.1.0", note = "This method is deprecated!")]
    fn clear_lin_slave_msg(&self, channel: u8, pids: Vec<u8>) -> CanResult<()> {
        Err(CanError::NotSupportedError)
    }
}

#[allow(unused_variables)]
pub trait ZCloud {
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
    fn get_userdata(&self, update: Option<i32>) -> CanResult<ZCloudUserData> {
        Err(CanError::NotSupportedError)
    }
    fn receive_gps(&self, size: u32, timeout: Option<u32>) -> CanResult<Vec<ZCloudGpsFrame>> {
        Err(CanError::NotSupportedError)
    }
}

/// device is supported LIN
#[allow(dead_code)]
#[inline(always)]
pub(crate) fn lin_support(dev_type: ZCanDeviceType) -> CanResult<()> {
    if !dev_type.lin_support() {
        return Err(CanError::NotSupportedError);
    }
    Ok(())
}

/// device is supported CLOUD
#[allow(dead_code)]
#[inline(always)]
pub(crate) fn cloud_support(dev_type: ZCanDeviceType) -> CanResult<()> {
    if !dev_type.cloud_support() {
        return Err(CanError::NotSupportedError);
    }
    Ok(())
}

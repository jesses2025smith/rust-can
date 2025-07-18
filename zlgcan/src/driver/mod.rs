#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::ZDriver;
#[cfg(target_os = "windows")]
mod win;
#[cfg(target_os = "windows")]
pub use win::ZDriver;

use crate::{
    constants,
    native::{
        api::{ZChannelContext, ZDeviceContext},
        can::{CanMessage, ZCanChlError, ZCanChlStatus, ZCanFrameType},
        cloud::{ZCloudGpsFrame, ZCloudServerInfo, ZCloudUserData},
        device::{DeriveInfo, ZCanDeviceType, ZDeviceInfo},
        lin::{ZLinChlCfg, ZLinFrame, ZLinPublish, ZLinPublishEx, ZLinSubscribe},
    },
};
use rs_can::{CanDevice, CanError, CanFrame, CanResult, CanType, ChannelConfig, DeviceBuilder};
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
    pub(crate) fn device_handler<C, T>(&self, callback: C) -> Result<T, CanError>
    where
        C: FnOnce(&Handler) -> Result<T, CanError>,
    {
        match &self.handler {
            Some(v) => callback(v),
            None => Err(CanError::device_not_opened()),
        }
    }

    #[inline(always)]
    pub(crate) fn can_handler<C, T>(&self, channel: u8, callback: C) -> Result<T, CanError>
    where
        C: FnOnce(&ZChannelContext) -> Result<T, CanError>,
    {
        self.device_handler(|hdl| -> Result<T, CanError> {
            match hdl.find_can(channel) {
                Some(context) => callback(context),
                None => Err(CanError::channel_not_opened(channel)),
            }
        })
    }

    #[inline(always)]
    pub(crate) fn lin_handler<C, T>(&self, channel: u8, callback: C) -> Result<T, CanError>
    where
        C: FnOnce(&ZChannelContext) -> Result<T, CanError>,
    {
        self.device_handler(|hdl| -> Result<T, CanError> {
            match hdl.lin_channels().get(&channel) {
                Some(chl) => callback(chl),
                None => Err(CanError::channel_not_opened(channel)),
            }
        })
    }
}

#[async_trait::async_trait]
impl CanDevice for ZDriver {
    type Channel = u8;
    type Frame = CanMessage;

    #[inline]
    fn opened_channels(&self) -> Vec<Self::Channel> {
        match &self.handler {
            Some(v) => v.can_channels().keys().map(|v| v.clone()).collect(),
            None => vec![],
        }
    }

    async fn transmit(&self, msg: Self::Frame, _: Option<u32>) -> CanResult<(), CanError> {
        let channel = msg.channel();
        let _ = match msg.can_type() {
            CanType::Can => self.transmit_can(channel, vec![msg]),
            CanType::CanFd => self.transmit_canfd(channel, vec![msg]),
            CanType::CanXl => Err(CanError::NotSupportedError),
        }?;

        Ok(())
    }

    async fn receive(
        &self,
        channel: Self::Channel,
        timeout: Option<u32>,
    ) -> CanResult<Vec<Self::Frame>, CanError> {
        let mut results: Vec<CanMessage> = Vec::new();

        let count_can = self.get_can_num(channel, ZCanFrameType::CAN)?;
        if count_can > 0 {
            rsutil::trace!("RUST-CAN - received CAN: {}", count_can);
            let mut frames = self.receive_can(channel, count_can, timeout)?;
            results.append(&mut frames);
        }

        if self.device_type().canfd_support() {
            let count_fd = self.get_can_num(channel, ZCanFrameType::CANFD)?;
            if count_fd > 0 {
                rsutil::trace!("RUST-CAN - received CANFD: {}", count_fd);
                let mut frames = self.receive_canfd(channel, count_fd, timeout)?;
                results.append(&mut frames);
            }
        }

        Ok(results)
    }

    #[inline]
    fn shutdown(&mut self) {
        self.close()
    }
}

impl TryFrom<DeviceBuilder<u8>> for ZDriver {
    type Error = CanError;

    fn try_from(builder: DeviceBuilder<u8>) -> Result<Self, Self::Error> {
        let libpath = builder
            .get_other::<String>(constants::LIBPATH)?
            .ok_or(CanError::other_error("`libpath` not found`"))?;
        let dev_type = builder
            .get_other::<ZCanDeviceType>(constants::DEVICE_TYPE)?
            .ok_or(CanError::other_error("`device_type` not found`"))?;
        let dev_idx = builder
            .get_other::<u32>(constants::DEVICE_INDEX)?
            .ok_or(CanError::other_error("`device_index` not found`"))?;
        let derive = builder.get_other::<DeriveInfo>(constants::DERIVE_INFO)?;

        let mut device = Self::new(libpath, dev_type, dev_idx, derive)?;
        device.open()?;

        builder
            .channel_configs()
            .iter()
            .try_for_each(|(&chl, cfg)| device.init_can_chl(chl, cfg))?;

        Ok(device)
    }
}

#[allow(unused_variables)]
pub trait ZDevice {
    fn new(
        libpath: String,
        dev_type: ZCanDeviceType,
        dev_idx: u32,
        derive: Option<DeriveInfo>,
    ) -> Result<Self, CanError>
    where
        Self: Sized;
    fn device_type(&self) -> ZCanDeviceType;
    fn device_index(&self) -> u32;
    fn open(&mut self) -> Result<(), CanError>;
    fn close(&mut self);
    fn device_info(&self) -> Result<&ZDeviceInfo, CanError>;
    fn is_derive_device(&self) -> bool;
    fn is_online(&self) -> Result<bool, CanError> {
        Err(CanError::NotSupportedError)
    }
    fn timestamp(&self, channel: u8) -> Result<u64, CanError>;
}

#[allow(unused_variables)]
pub trait ZCan {
    fn init_can_chl(&mut self, channel: u8, cfg: &ChannelConfig) -> Result<(), CanError>;
    fn reset_can_chl(&mut self, channel: u8) -> Result<(), CanError>;
    // fn resistance_state(&self, dev_idx: u32, channel: u8) -> Result<(), CanError>;
    fn read_can_chl_status(&self, channel: u8) -> Result<ZCanChlStatus, CanError>;
    fn read_can_chl_error(&self, channel: u8) -> Result<ZCanChlError, CanError>;
    fn clear_can_buffer(&self, channel: u8) -> Result<(), CanError>;
    fn get_can_num(&self, channel: u8, can_type: ZCanFrameType) -> Result<u32, CanError>;
    fn receive_can(
        &self,
        channel: u8,
        size: u32,
        timeout: Option<u32>,
    ) -> Result<Vec<CanMessage>, CanError>;
    fn transmit_can(&self, channel: u8, frames: Vec<CanMessage>) -> Result<u32, CanError>;
    fn receive_canfd(
        &self,
        channel: u8,
        size: u32,
        timeout: Option<u32>,
    ) -> Result<Vec<CanMessage>, CanError> {
        Err(CanError::NotSupportedError)
    }
    fn transmit_canfd(&self, channel: u8, frames: Vec<CanMessage>) -> Result<u32, CanError> {
        Err(CanError::NotSupportedError)
    }
}

#[allow(unused_variables)]
pub trait ZLin {
    fn init_lin_chl(&mut self, channel: u8, cfg: ZLinChlCfg) -> Result<(), CanError> {
        Err(CanError::NotSupportedError)
    }
    fn reset_lin_chl(&mut self, channel: u8) -> Result<(), CanError> {
        Err(CanError::NotSupportedError)
    }
    fn clear_lin_buffer(&self, channel: u8) -> Result<(), CanError> {
        Err(CanError::NotSupportedError)
    }
    fn get_lin_num(&self, channel: u8) -> Result<u32, CanError> {
        Err(CanError::NotSupportedError)
    }
    fn receive_lin(
        &self,
        channel: u8,
        size: u32,
        timeout: Option<u32>,
    ) -> Result<Vec<ZLinFrame>, CanError> {
        Err(CanError::NotSupportedError)
    }
    fn transmit_lin(&self, channel: u8, frames: Vec<ZLinFrame>) -> Result<u32, CanError> {
        Err(CanError::NotSupportedError)
    }
    fn set_lin_subscribe(&self, channel: u8, cfg: Vec<ZLinSubscribe>) -> Result<(), CanError> {
        Err(CanError::NotSupportedError)
    }
    fn set_lin_publish(&self, channel: u8, cfg: Vec<ZLinPublish>) -> Result<(), CanError> {
        Err(CanError::NotSupportedError)
    }
    fn set_lin_publish_ext(&self, channel: u8, cfg: Vec<ZLinPublishEx>) -> Result<(), CanError> {
        Err(CanError::NotSupportedError)
    }
    fn wakeup_lin(&self, channel: u8) -> Result<(), CanError> {
        Err(CanError::NotSupportedError)
    }
    #[deprecated(since = "0.1.0", note = "This method is deprecated!")]
    fn set_lin_slave_msg(&self, channel: u8, msg: Vec<ZLinFrame>) -> Result<(), CanError> {
        Err(CanError::NotSupportedError)
    }
    #[deprecated(since = "0.1.0", note = "This method is deprecated!")]
    fn clear_lin_slave_msg(&self, channel: u8, pids: Vec<u8>) -> Result<(), CanError> {
        Err(CanError::NotSupportedError)
    }
}

#[allow(unused_variables)]
pub trait ZCloud {
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
    fn get_userdata(&self, update: Option<i32>) -> Result<ZCloudUserData, CanError> {
        Err(CanError::NotSupportedError)
    }
    fn receive_gps(
        &self,
        size: u32,
        timeout: Option<u32>,
    ) -> Result<Vec<ZCloudGpsFrame>, CanError> {
        Err(CanError::NotSupportedError)
    }
}

/// device is supported LIN
pub(crate) fn lin_support(dev_type: ZCanDeviceType) -> Result<(), CanError> {
    if !dev_type.lin_support() {
        return Err(CanError::NotSupportedError);
    }
    Ok(())
}

/// device is supported CLOUD
#[allow(dead_code)]
pub(crate) fn cloud_support(dev_type: ZCanDeviceType) -> Result<(), CanError> {
    if !dev_type.cloud_support() {
        return Err(CanError::NotSupportedError);
    }
    Ok(())
}

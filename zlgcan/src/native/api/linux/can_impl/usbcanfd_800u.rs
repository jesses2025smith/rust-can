use rs_can::{CanError, ChannelConfig};
use crate::{
    constants,
    native::{
        api::{USBCANFD800UApi, ZCanApi, ZChannelContext},
        can::{constants::BITRATE_CFG_FILENAME, common::CanChlCfgContext, CanMessage, ZCanChlCfg, ZCanChlError, ZCanChlStatus, ZCanChlType, ZCanFdFrameInner, ZCanFrame, ZCanFrameInner, ZCanFrameType}
    }
};

impl ZCanApi for USBCANFD800UApi<'_> {
    fn init_can_chl(&self, libpath: &str, context: &mut ZChannelContext, cfg: &ChannelConfig) -> Result<(), CanError> {
        unsafe {
            // init can channel
            let (dev_type, dev_hdl, channel) = (context.device.dev_type, context.device_handler()?, context.channel);
            let cfg_ctx = CanChlCfgContext::new(libpath)?;
            let bc_ctx = cfg_ctx.0.get(&(dev_type as u32).to_string())
                .ok_or(CanError::InitializeError(
                    format!("device: {} is not configured in {}", dev_type, BITRATE_CFG_FILENAME)
                ))?;
            let can_type = cfg.get_other::<ZCanChlType>(constants::CHANNEL_TYPE)?
                .unwrap_or(ZCanChlType::CAN);
            let cfg = ZCanChlCfg::new(
                dev_type,
                can_type,
                bc_ctx,
                cfg
            )?;
            let handler = match (self.ZCAN_InitCAN)(dev_hdl, channel as u32, &cfg) {
                Self::INVALID_CHANNEL_HANDLE => Err(
                    CanError::InitializeError(format!("`ZCAN_InitCAN` ret: {}", Self::INVALID_CHANNEL_HANDLE))
                ),
                handler => {
                    match (self.ZCAN_StartCAN)(handler) {
                        Self::STATUS_OK => Ok(handler),
                        code => Err(
                            CanError::InitializeError(format!("`ZCAN_InitCAN` ret: {}", code))
                        ),
                    }
                }
            }?;

            context.chl_hdl = Some(handler);
            Ok(())
        }
    }

    fn reset_can_chl(&self, context: &ZChannelContext) -> Result<(), CanError> {
        match unsafe { (self.ZCAN_ResetCAN)(context.channel_handler()?) } {
            Self::STATUS_OK => Ok(()),
            code => Err(
                CanError::OperationError(format!("`ZCAN_ResetCAN` ret: {}", code))
            ),
        }
    }

    fn read_can_chl_status(&self, context: &ZChannelContext) -> Result<ZCanChlStatus, CanError> {
        let mut status: ZCanChlStatus = Default::default();
        match unsafe { (self.ZCAN_ReadChannelStatus)(context.channel_handler()?, &mut status) } {
            Self::STATUS_OK => Ok(status),
            code => Err(
                CanError::OperationError(format!("`ZCAN_ReadChannelStatus` ret: {}", code))
            ),
        }
    }

    fn read_can_chl_error(&self, context: &ZChannelContext) -> Result<ZCanChlError, CanError> {
        let mut info: ZCanChlError = ZCanChlError { v1: Default::default() };
        match unsafe { (self.ZCAN_ReadChannelErrInfo)(context.channel_handler()?, &mut info) } {
            Self::STATUS_OK => Ok(info),
            code => Err(
                CanError::OperationError(format!("`ZCAN_ReadChannelErrInfo` ret: {}", code))
            ),
        }
    }

    fn clear_can_buffer(&self, context: &ZChannelContext) -> Result<(), CanError> {
        match unsafe { (self.ZCAN_ClearBuffer)(context.channel_handler()?) } {
            Self::STATUS_OK => Ok(()),
            code => Err(
                CanError::OperationError(format!("`ZCAN_ClearBuffer` ret: {}", code))
            ),
        }
    }

    fn get_can_num(&self, context: &ZChannelContext, can_type: ZCanFrameType) -> Result<u32, CanError> {
        let ret = unsafe { (self.ZCAN_GetReceiveNum)(context.channel_handler()?, can_type as u8) };
        if ret > 0 {
            rsutil::trace!("ZLGCAN - get receive {} number: {}.", can_type, ret);
        }
        Ok(ret)
    }

    fn receive_can(&self, context: &ZChannelContext, size: u32, timeout: u32) -> Result<Vec<CanMessage>, CanError> {
        let mut frames = Vec::new();
        frames.resize(size as usize, ZCanFrame { can: ZCanFrameInner { libother: Default::default() } });

        let ret = unsafe { (self.ZCAN_Receive)(context.channel_handler()?, frames.as_mut_ptr(), size, timeout) };
        if ret < size {
            rsutil::warn!("ZLGCAN - receive CAN frame expect: {}, actual: {}!", size, ret);
        }
        else if ret > 0 {
            rsutil::trace!("ZLGCAN - receive CAN frame: {}", ret);
        }

        Ok(frames.into_iter()
            .map(|mut frame| unsafe {
                frame.can.libother.set_channel(context.channel);
                frame.can.libother.into()
            })
            .collect::<Vec<_>>())
    }

    fn transmit_can(&self, context: &ZChannelContext, frames: Vec<CanMessage>) -> Result<u32, CanError> {
        let frames = frames.into_iter()
            .map(|frame| ZCanFrame { can: ZCanFrameInner { libother: frame.into() } })
            .collect::<Vec<_>>();

        let len = frames.len() as u32;
        let ret = unsafe { (self.ZCAN_Transmit)(context.channel_handler()?, frames.as_ptr(), len) };
        if ret < len {
            rsutil::warn!("ZLGCAN - transmit CAN frame expect: {}, actual: {}!", len, ret);
        }
        else {
            rsutil::trace!("ZLGCAN - transmit CAN frame: {}", ret);
        }
        Ok(ret)
    }

    fn receive_canfd(&self, context: &ZChannelContext, size: u32, timeout: u32) -> Result<Vec<CanMessage>, CanError> {
        let mut frames = Vec::new();
        frames.resize(size as usize, ZCanFrame { canfd: ZCanFdFrameInner { libother: Default::default() } });

        let ret = unsafe { (self.ZCAN_ReceiveFD)(context.channel_handler()?, frames.as_mut_ptr(), size, timeout) };
        if ret < size {
            rsutil::warn!("ZLGCAN - receive CAN-FD frame expect: {}, actual: {}!", size, ret);
        }
        else if ret > 0 {
            rsutil::trace!("ZLGCAN - receive CAN-FD frame: {}", ret);
        }

        Ok(frames.into_iter()
            .map(|mut frame| unsafe {
                frame.canfd.libother.set_channel(context.channel);
                frame.canfd.libother.into()
            })
            .collect::<Vec<_>>())
    }

    fn transmit_canfd(&self, context: &ZChannelContext, frames: Vec<CanMessage>) -> Result<u32, CanError> {
        let frames = frames.into_iter()
            .map(|frame| ZCanFrame { canfd: ZCanFdFrameInner { libother: frame.into() } })
            .collect::<Vec<_>>();

        let len = frames.len() as u32;
        let ret = unsafe { (self.ZCAN_TransmitFD)(context.channel_handler()?, frames.as_ptr(), len) };
        if ret < len {
            rsutil::warn!("ZLGCAN - transmit CANFD frame expect: {}, actual: {}!", len, ret);
        }
        else {
            rsutil::trace!("ZLGCAN - transmit CAN-FD frame: {}", ret);
        }
        Ok(ret)
    }
}

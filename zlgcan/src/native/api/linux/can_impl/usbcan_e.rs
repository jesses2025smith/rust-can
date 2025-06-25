
impl ZCanApi for USBCANEApi<'_> {
    fn init_can_chl(&self, libpath: &str, context: &mut ZChannelContext, cfg: &ChannelConfig) -> Result<(), CanError> {
        let dev_type = context.device_type();
        let dev_hdl = context.device_handler()?;
        let channel = context.channel() as u32;
        let cfg_ctx = CanChlCfgContext::new(libpath)?;
        let bc_ctx = cfg_ctx.0.get(&(dev_type as u32).to_string())
            .ok_or(CanError::InitializeError(
                format!("device: {} is not configured in {}", dev_type, BITRATE_CFG_FILENAME)
            ))?;
        unsafe {
            let handler = match dev_type {
                ZCanDeviceType::ZCAN_USBCAN_4E_U => {
                    match (self.ZCAN_InitCAN)(dev_hdl, channel, std::ptr::null()) as u32 {
                        Self::INVALID_CHANNEL_HANDLE =>
                            Err(CanError::InitializeError(format!("`ZCAN_InitCAN` ret: {}", Self::INVALID_CHANNEL_HANDLE))),
                        handler => Ok(handler),
                    }
                },
                ZCanDeviceType::ZCAN_USBCAN_8E_U => {
                    let can_type = cfg.get_other::<ZCanChlType>(CHANNEL_TYPE)?
                        .unwrap_or(ZCanChlType::CAN);
                    let cfg = ZCanChlCfg::new(
                        dev_type,
                        can_type,
                        bc_ctx,
                        cfg
                    )?;
                    match (self.ZCAN_InitCAN)(dev_hdl, channel, &cfg) as u32 {
                        Self::INVALID_CHANNEL_HANDLE => Err(CanError::InitializeError(format!("ZCAN_InitCAN ret: {}", Self::INVALID_CHANNEL_HANDLE))),
                        handler => {
                            match (self.ZCAN_StartCAN)(handler) as u32 {
                                Self::STATUS_OK => Ok(handler),
                                code => Err(CanError::InitializeError(format!("`ZCAN_StartCAN` ret: {}", code))),
                            }
                        }
                    }
                },
                _ => Err(CanError::NotSupportedError),
            }?;

            context.set_channel_handler(Some(handler));
            Ok(())
        }
    }

    fn reset_can_chl(&self, context: &ZChannelContext) -> Result<(), CanError> {
        match unsafe { (self.ZCAN_ResetCAN)(context.channel_handler()?) } as u32 {
            Self::STATUS_OK => Ok(()),
            code => Err(CanError::OperationError(format!("`ZCAN_ResetCAN` ret: {}", code))),
        }
    }

    fn read_can_chl_status(&self, context: &ZChannelContext) -> Result<ZCanChlStatus, CanError> {
        let mut status: ZCanChlStatus = Default::default();
        match unsafe { (self.ZCAN_ReadChannelStatus)(context.channel_handler()?, &mut status) } as u32 {
            Self::STATUS_OK => Ok(status),
            code => Err(CanError::OperationError(format!("`ZCAN_ReadChannelStatus` ret: {}", code))),
        }
    }

    fn read_can_chl_error(&self, context: &ZChannelContext) -> Result<ZCanChlError, CanError> {
        let mut info: ZCanChlError = ZCanChlError { v1: Default::default() };
        match unsafe { (self.ZCAN_ReadChannelErrInfo)(context.channel_handler()?, &mut info) } as u32  {
            Self::STATUS_OK => Ok(info),
            code => Err(CanError::OperationError(format!("`ZCAN_ReadChannelErrInfo` ret: {}", code))),
        }
    }

    fn clear_can_buffer(&self, context: &ZChannelContext) -> Result<(), CanError> {
        match unsafe { (self.ZCAN_ClearBuffer)(context.channel_handler()?) } as u32 {
            Self::STATUS_OK => Ok(()),
            code => Err(CanError::OperationError(format!("`ZCAN_ClearBuffer` ret: {}", code))),
        }
    }

    fn get_can_num(&self, context: &ZChannelContext, can_type: ZCanFrameType) -> Result<u32, CanError> {
        let ret = unsafe { (self.ZCAN_GetReceiveNum)(context.channel_handler()?, can_type as u8) };
        if ret > 0 {
            rsutil::trace!("ZLGCAN - get receive {} number: {}.", can_type, ret);
        }
        Ok(ret as u32)
    }

    fn receive_can(&self, context: &ZChannelContext, size: u32, timeout: u32) -> Result<Vec<CanMessage>, CanError> {
        let mut frames = Vec::new();
        frames.resize(size as usize, ZCanFrame { can: ZCanFrameInner { libother: Default::default() } });

        let ret = unsafe { (self.ZCAN_Receive)(context.channel_handler()?, frames.as_mut_ptr(), size, timeout) };
        let ret = ret as u32;
        if ret < size {
            rsutil::warn!("ZLGCAN - receive CAN frame expect: {}, actual: {}!", size, ret);
        }
        else if ret > 0 {
            rsutil::trace!("ZLGCAN - receive CAN frame: {}", ret);
        }

        Ok(frames.into_iter()
            .map(|mut frame| unsafe {
                frame.can.libother.set_channel(context.channel());
                frame.can.libother.into()
            })
            .collect::<Vec<_>>())
    }

    fn transmit_can(&self, context: &ZChannelContext, frames: Vec<CanMessage>) -> Result<u32, CanError> {
        let frames = frames.into_iter()
            .map(|mut frame| ZCanFrame { can: ZCanFrameInner { libother: frame.into() } })
            .collect::<Vec<_>>();

        let len = frames.len() as u32;
        let ret = unsafe { (self.ZCAN_Transmit)(context.channel_handler()?, frames.as_ptr(), len) };
        let ret = ret as u32;
        if ret < len {
            rsutil::warn!("ZLGCAN - transmit CAN frame expect: {}, actual: {}!", len, ret);
        }
        else {
            rsutil::trace!("ZLGCAN - transmit CAN frame: {}", ret);
        }
        Ok(ret)
    }
}

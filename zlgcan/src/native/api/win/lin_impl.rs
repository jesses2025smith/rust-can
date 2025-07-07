use crate::native::{
    api::{WinApi, ZChannelContext, ZLinApi},
    lin::{ZLinChlCfg, ZLinFrame, ZLinPublish, ZLinPublishEx, ZLinSubscribe},
};
use rs_can::CanError;

impl ZLinApi for WinApi<'_> {
    fn init_lin_chl(
        &self,
        context: &mut ZChannelContext,
        cfg: &ZLinChlCfg,
    ) -> Result<(), CanError> {
        unsafe {
            let dev_hdl = context.device_handler()?;
            let channel = context.channel;
            match (self.ZCAN_InitLIN)(dev_hdl, channel as u32, cfg) {
                Self::INVALID_CHANNEL_HANDLE => Err(CanError::OperationError(format!(
                    "`ZCAN_InitLIN` ret = {}",
                    Self::INVALID_CHANNEL_HANDLE
                ))),
                handler => match (self.ZCAN_StartLIN)(dev_hdl) {
                    Self::STATUS_OK => {
                        context.chl_hdl = Some(handler);
                        Ok(())
                    }
                    code => Err(CanError::InitializeError(format!(
                        "`ZCAN_StartLIN` ret = {}",
                        code
                    ))),
                },
            }
        }
    }
    fn reset_lin_chl(&self, context: &ZChannelContext) -> Result<(), CanError> {
        match unsafe { (self.ZCAN_ResetLIN)(context.channel_handler()?) } {
            Self::STATUS_OK => Ok(()),
            code => Err(CanError::OperationError(format!(
                "`ZCAN_ResetLIN` ret = {}",
                code
            ))),
        }
    }
    fn get_lin_num(&self, context: &ZChannelContext) -> Result<u32, CanError> {
        let ret = unsafe { (self.ZCAN_GetLINReceiveNum)(context.channel_handler()?) };
        if ret > 0 {
            rsutil::trace!("ZLGCAN - get receive LIN number: {}.", ret);
        }
        Ok(ret)
    }
    fn receive_lin(
        &self,
        context: &ZChannelContext,
        size: u32,
        timeout: u32,
    ) -> Result<Vec<ZLinFrame>, CanError> {
        let mut frames = Vec::new();
        frames.resize_with(size as usize, ZLinFrame::default_data);

        let ret = unsafe {
            (self.ZCAN_ReceiveLIN)(
                context.channel_handler()?,
                frames.as_mut_ptr(),
                size,
                timeout,
            )
        };
        if ret < size {
            rsutil::warn!(
                "ZLGCAN - receive LIN frame expect: {}, actual: {}!",
                size,
                ret
            );
        } else if ret > 0 {
            rsutil::trace!("ZLGCAN - receive LIN frame: {}", ret);
        }
        Ok(frames)
    }
    fn transmit_lin(
        &self,
        context: &ZChannelContext,
        frames: Vec<ZLinFrame>,
    ) -> Result<u32, CanError> {
        let len = frames.len() as u32;
        let ret =
            unsafe { (self.ZCAN_TransmitLIN)(context.channel_handler()?, frames.as_ptr(), len) };
        if ret < len {
            rsutil::warn!(
                "ZLGCAN - transmit LIN frame expect: {}, actual: {}!",
                len,
                ret
            );
        } else {
            rsutil::trace!("ZLGCAN - transmit LIN frame: {}", ret);
        }
        Ok(ret)
    }
    fn set_lin_subscribe(
        &self,
        context: &ZChannelContext,
        cfg: Vec<ZLinSubscribe>,
    ) -> Result<(), CanError> {
        let len = cfg.len() as u32;
        match unsafe { (self.ZCAN_SetLINSubscribe)(context.channel_handler()?, cfg.as_ptr(), len) }
        {
            Self::STATUS_OK => Ok(()),
            code => Err(CanError::OperationError(format!(
                "`ZCAN_SetLINSubscribe` ret = {}",
                code
            ))),
        }
    }
    fn set_lin_publish(
        &self,
        context: &ZChannelContext,
        cfg: Vec<ZLinPublish>,
    ) -> Result<(), CanError> {
        let len = cfg.len() as u32;
        match unsafe { (self.ZCAN_SetLINPublish)(context.channel_handler()?, cfg.as_ptr(), len) } {
            Self::STATUS_OK => Ok(()),
            code => Err(CanError::OperationError(format!(
                "`ZCAN_SetLINPublish` ret = {}",
                code
            ))),
        }
    }
    fn wakeup_lin(&self, context: &ZChannelContext) -> Result<(), CanError> {
        match unsafe { (self.ZCAN_WakeUpLIN)(context.channel_handler()?) } {
            Self::STATUS_OK => Ok(()),
            code => Err(CanError::OperationError(format!(
                "`ZCAN_WakeUpLIN` ret = {}",
                code
            ))),
        }
    }
    fn set_lin_publish_ex(
        &self,
        context: &ZChannelContext,
        cfg: Vec<ZLinPublishEx>,
    ) -> Result<(), CanError> {
        let len = cfg.len() as u32;
        match unsafe { (self.ZCAN_SetLINPublishEx)(context.channel_handler()?, cfg.as_ptr(), len) }
        {
            Self::STATUS_OK => Ok(()),
            code => Err(CanError::OperationError(format!(
                "`ZCAN_SetLINPublishEx` ret = {}",
                code
            ))),
        }
    }
    fn set_lin_slave_msg(
        &self,
        context: &ZChannelContext,
        msg: Vec<ZLinFrame>,
    ) -> Result<(), CanError> {
        let len = msg.len() as u32;
        match unsafe { (self.ZCAN_SetLINSlaveMsg)(context.channel_handler()?, msg.as_ptr(), len) } {
            Self::STATUS_OK => Ok(()),
            code => Err(CanError::OperationError(format!(
                "`ZCAN_SetLINSlaveMsg` ret = {}",
                code
            ))),
        }
    }
    fn clear_lin_slave_msg(
        &self,
        context: &ZChannelContext,
        pids: Vec<u8>,
    ) -> Result<(), CanError> {
        let len = pids.len() as u32;
        match unsafe {
            (self.ZCAN_ClearLINSlaveMsg)(context.channel_handler()?, pids.as_ptr(), len)
        } {
            Self::STATUS_OK => Ok(()),
            code => Err(CanError::OperationError(format!(
                "`ZCAN_ClearLINSlaveMsg` ret = {}",
                code
            ))),
        }
    }
}

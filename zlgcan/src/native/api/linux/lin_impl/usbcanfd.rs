use crate::native::{
    api::{USBCANFDApi, ZChannelContext, ZLinApi},
    lin::{ZLinChlCfg, ZLinFrame, ZLinPublish, ZLinSubscribe},
};
use rs_can::CanError;

impl ZLinApi for USBCANFDApi<'_> {
    fn init_lin_chl(
        &self,
        context: &mut ZChannelContext,
        cfg: &ZLinChlCfg,
    ) -> Result<(), CanError> {
        let (dev_type, dev_idx, channel) = (
            context.device.dev_type,
            context.device.dev_idx,
            context.channel,
        );
        unsafe {
            match (self.VCI_InitLIN)(dev_type as u32, dev_idx, channel as u32, cfg) {
                Self::STATUS_OK => {
                    match (self.VCI_StartLIN)(dev_type as u32, dev_idx, channel as u32) {
                        Self::STATUS_OK => Ok(()),
                        code => Err(CanError::InitializeError(format!(
                            "`VCI_StartLIN` ret: {}",
                            code
                        ))),
                    }
                }
                code => Err(CanError::InitializeError(format!(
                    "`VCI_InitLIN` ret: {}",
                    code
                ))),
            }
        }
    }
    fn reset_lin_chl(&self, context: &ZChannelContext) -> Result<(), CanError> {
        let (dev_type, dev_idx, channel) = (
            context.device.dev_type,
            context.device.dev_idx,
            context.channel,
        );
        match unsafe { (self.VCI_ResetLIN)(dev_type as u32, dev_idx, channel as u32) } {
            Self::STATUS_OK => Ok(()),
            code => Err(CanError::OperationError(format!(
                "`VCI_ResetLIN` ret: {}",
                code
            ))),
        }
    }
    fn clear_lin_buffer(&self, context: &ZChannelContext) -> Result<(), CanError> {
        let (dev_type, dev_idx, channel) = (
            context.device.dev_type,
            context.device.dev_idx,
            context.channel,
        );
        match unsafe { (self.VCI_ClearLINBuffer)(dev_type as u32, dev_idx, channel as u32) } {
            Self::STATUS_OK => Ok(()),
            code => Err(CanError::OperationError(format!(
                "`VCI_ClearLINBuffer` ret: {}",
                code
            ))),
        }
    }
    fn get_lin_num(&self, context: &ZChannelContext) -> Result<u32, CanError> {
        let (dev_type, dev_idx, channel) = (
            context.device.dev_type,
            context.device.dev_idx,
            context.channel,
        );
        let ret = unsafe { (self.VCI_GetLINReceiveNum)(dev_type as u32, dev_idx, channel as u32) };
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
        let (dev_type, dev_idx, channel) = (
            context.device.dev_type,
            context.device.dev_idx,
            context.channel,
        );
        let mut frames = Vec::new();
        frames.resize_with(size as usize, ZLinFrame::default_data);

        let ret = unsafe {
            (self.VCI_ReceiveLIN)(
                dev_type as u32,
                dev_idx,
                channel as u32,
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
        let (dev_type, dev_idx, channel) = (
            context.device.dev_type,
            context.device.dev_idx,
            context.channel,
        );
        let len = frames.len() as u32;
        let ret = unsafe {
            (self.VCI_TransmitLIN)(
                dev_type as u32,
                dev_idx,
                channel as u32,
                frames.as_ptr(),
                len,
            )
        };
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
        let (dev_type, dev_idx, channel) = (
            context.device.dev_type,
            context.device.dev_idx,
            context.channel,
        );
        let len = cfg.len() as u32;
        match unsafe {
            (self.VCI_SetLINSubscribe)(dev_type as u32, dev_idx, channel as u32, cfg.as_ptr(), len)
        } {
            Self::STATUS_OK => Ok(()),
            code => Err(CanError::OperationError(format!(
                "`VCI_SetLINSubscribe` ret: {}",
                code
            ))),
        }
    }
    fn set_lin_publish(
        &self,
        context: &ZChannelContext,
        cfg: Vec<ZLinPublish>,
    ) -> Result<(), CanError> {
        let (dev_type, dev_idx, channel) = (
            context.device.dev_type,
            context.device.dev_idx,
            context.channel,
        );
        let len = cfg.len() as u32;
        match unsafe {
            (self.VCI_SetLINPublish)(dev_type as u32, dev_idx, channel as u32, cfg.as_ptr(), len)
        } {
            Self::STATUS_OK => Ok(()),
            code => Err(CanError::OperationError(format!(
                "`VCI_SetLINPublish` ret: {}",
                code
            ))),
        }
    }
}

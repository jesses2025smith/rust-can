use crate::native::{
    api::{USBCANApi, ZCanApi, ZChannelContext},
    can::{
        common::{CanChlCfgContext, ZCanChlCfgInner},
        constants::BITRATE_CFG_FILENAME,
        CanMessage, ZCanChlError, ZCanChlStatus, ZCanFrame, ZCanFrameInner, ZCanFrameType,
    },
};
use rs_can::{CanError, ChannelConfig};

impl ZCanApi for USBCANApi<'_> {
    fn init_can_chl(
        &self,
        libpath: &str,
        context: &mut ZChannelContext,
        cfg: &ChannelConfig,
    ) -> Result<(), CanError> {
        let (dev_type, dev_idx, channel) = (
            context.device.dev_type,
            context.device.dev_idx,
            context.channel,
        );
        let cfg_ctx = CanChlCfgContext::new(libpath)?;
        let bc_ctx =
            cfg_ctx
                .0
                .get(&(dev_type as u32).to_string())
                .ok_or(CanError::InitializeError(format!(
                    "device: {} is not configured in {}",
                    dev_type, BITRATE_CFG_FILENAME
                )))?;
        unsafe {
            let dev_type = dev_type as u32;
            let channel = channel as u32;

            // let bitrate = cfg.bitrate();
            let cfg = ZCanChlCfgInner::try_from_with(bc_ctx, cfg)?;
            match (self.VCI_InitCAN)(dev_type, dev_idx, channel, &cfg) {
                Self::STATUS_OK => match (self.VCI_StartCAN)(dev_type, dev_idx, channel) {
                    Self::STATUS_OK => {
                        context.chl_hdl = None;
                        Ok(())
                    }
                    code => Err(CanError::InitializeError(format!(
                        "`VCI_StartCAN` ret: {}",
                        code
                    ))),
                },
                code => Err(CanError::InitializeError(format!(
                    "`VCI_InitCAN` ret: {}",
                    code
                ))),
            }
        }
    }

    fn reset_can_chl(&self, context: &ZChannelContext) -> Result<(), CanError> {
        let (dev_type, dev_idx, channel) = (
            context.device.dev_type,
            context.device.dev_idx,
            context.channel,
        );
        match unsafe { (self.VCI_ResetCAN)(dev_type as u32, dev_idx, channel as u32) } {
            Self::STATUS_OK => Ok(()),
            code => Err(CanError::OperationError(format!(
                "`VCI_ResetCAN` ret: {}",
                code
            ))),
        }
    }

    fn read_can_chl_status(&self, context: &ZChannelContext) -> Result<ZCanChlStatus, CanError> {
        let (dev_type, dev_idx, channel) = (
            context.device.dev_type,
            context.device.dev_idx,
            context.channel,
        );
        let mut status: ZCanChlStatus = Default::default();
        match unsafe {
            (self.VCI_ReadCANStatus)(dev_type as u32, dev_idx, channel as u32, &mut status)
        } {
            Self::STATUS_OK => Ok(status),
            code => Err(CanError::OperationError(format!(
                "`VCI_ReadCANStatus` ret: {}",
                code
            ))),
        }
    }

    fn read_can_chl_error(&self, context: &ZChannelContext) -> Result<ZCanChlError, CanError> {
        let (dev_type, dev_idx, channel) = (
            context.device.dev_type,
            context.device.dev_idx,
            context.channel,
        );
        let mut info = ZCanChlError {
            v1: Default::default(),
        };
        match unsafe { (self.VCI_ReadErrInfo)(dev_type as u32, dev_idx, channel as u32, &mut info) }
        {
            Self::STATUS_OK => Ok(info),
            code => Err(CanError::OperationError(format!(
                "`VCI_ReadErrInfo` ret: {}",
                code
            ))),
        }
    }

    fn clear_can_buffer(&self, context: &ZChannelContext) -> Result<(), CanError> {
        let (dev_type, dev_idx, channel) = (
            context.device.dev_type,
            context.device.dev_idx,
            context.channel,
        );
        match unsafe { (self.VCI_ClearBuffer)(dev_type as u32, dev_idx, channel as u32) } {
            Self::STATUS_OK => Ok(()),
            code => Err(CanError::OperationError(format!(
                "`VCI_ClearBuffer` ret: {}",
                code
            ))),
        }
    }

    fn get_can_num(
        &self,
        context: &ZChannelContext,
        can_type: ZCanFrameType,
    ) -> Result<u32, CanError> {
        let (dev_type, dev_idx, channel) = (
            context.device.dev_type,
            context.device.dev_idx,
            context.channel,
        );
        let mut _channel = channel as u32;
        match can_type {
            ZCanFrameType::CAN => {}
            ZCanFrameType::CANFD => _channel |= 0x8000_0000,
            ZCanFrameType::ALL => return Err(CanError::other_error("method not supported")),
        }
        let ret = unsafe { (self.VCI_GetReceiveNum)(dev_type as u32, dev_idx, _channel) };
        if ret > 0 {
            rsutil::trace!("ZLGCAN - get receive {} number: {}.", can_type, ret);
        }
        Ok(ret)
    }

    fn receive_can(
        &self,
        context: &ZChannelContext,
        size: u32,
        timeout: u32,
    ) -> Result<Vec<CanMessage>, CanError> {
        let (dev_type, dev_idx, channel) = (
            context.device.dev_type,
            context.device.dev_idx,
            context.channel,
        );
        let mut frames = Vec::new();
        frames.resize(
            size as usize,
            ZCanFrame {
                can: ZCanFrameInner {
                    libusbcan: Default::default(),
                },
            },
        );

        let ret = unsafe {
            (self.VCI_Receive)(
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
                "ZLGCAN - receive CAN frame expect: {}, actual: {}!",
                size,
                ret
            );
        } else if ret > 0 {
            rsutil::trace!("ZLGCAN - receive CAN frame: {}", ret);
        }

        Ok(frames
            .into_iter()
            .map(|frame| unsafe { frame.can.libusbcan.into() })
            .collect::<Vec<_>>())
    }

    fn transmit_can(
        &self,
        context: &ZChannelContext,
        frames: Vec<CanMessage>,
    ) -> Result<u32, CanError> {
        let frames = frames
            .into_iter()
            .map(|frame| ZCanFrame {
                can: ZCanFrameInner {
                    libusbcan: frame.into(),
                },
            })
            .collect::<Vec<_>>();

        let (dev_type, dev_idx, channel) = (
            context.device.dev_type,
            context.device.dev_idx,
            context.channel,
        );
        let len = frames.len() as u32;
        let ret = unsafe {
            (self.VCI_Transmit)(
                dev_type as u32,
                dev_idx,
                channel as u32,
                frames.as_ptr(),
                len,
            )
        };
        if ret < len {
            rsutil::warn!(
                "ZLGCAN - transmit CAN frame expect: {}, actual: {}!",
                len,
                ret
            );
        } else {
            rsutil::trace!("ZLGCAN - transmit CAN frame: {}", ret);
        }
        Ok(ret)
    }
}

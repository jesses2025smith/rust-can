use crate::{
    constants,
    native::{
        api::{WinApi, ZCanApi, ZChannelContext, ZDeviceApi},
        can::{
            self, ZCanChlCfg, ZCanChlError, ZCanChlStatus, ZCanChlType, ZCanFrame, ZCanFrameRx,
            ZCanFrameTx, ZCanFrameType,
        },
        constants::{
            BAUD_RATE, CANFD_ABIT_BAUD_RATE, CANFD_DBIT_BAUD_RATE, CLOCK, INTERNAL_RESISTANCE,
            PROTOCOL,
        },
        device::{CmdPath, ZCanDeviceType},
    },
};
use rs_can::{CanError, CanResult, ChannelConfig};
use rs_can::{MAX_FD_FRAME_SIZE, MAX_FRAME_SIZE};
use std::ffi::{c_void, CString};

impl ZCanApi for WinApi<'_> {
    fn init_can_chl(
        &self,
        libpath: &str,
        context: &mut ZChannelContext,
        cfg: &ChannelConfig,
    ) -> CanResult<()> {
        let cfg_ctx = can::common::CanChlCfgContext::new(libpath)?;
        let dev_type = context.device.dev_type;
        let bc_ctx =
            cfg_ctx
                .0
                .get(&(dev_type as u32).to_string())
                .ok_or(CanError::InitializeError(format!(
                    "device: {} is not configured in {}",
                    dev_type,
                    can::constants::BITRATE_CFG_FILENAME
                )))?;

        let channel = context.channel;
        unsafe {
            // configure the clock
            if let Some(clock) = bc_ctx.clock {
                let clock_path = CmdPath::new_path(CLOCK);
                let value = CString::new(clock.to_string())
                    .map_err(|e| CanError::OtherError(e.to_string()))?;
                self.set_value(context, &clock_path, value.as_ptr() as *const c_void)?;
            }
            // set channel resistance status
            if dev_type.has_resistance() {
                let state = cfg.termination.unwrap_or(true) as u32;
                let resistance_path = format!("{}/{}", channel, INTERNAL_RESISTANCE);
                let resistance_path = CmdPath::new_path(resistance_path.as_str());
                let value = CString::new(state.to_string())
                    .map_err(|e| CanError::OtherError(e.to_string()))?;
                self.set_value(context, &resistance_path, value.as_ptr() as *const c_void)?;
            }

            let can_type = cfg
                .get_other::<ZCanChlType>(constants::CHANNEL_TYPE)?
                .unwrap_or(ZCanChlType::default());
            if !matches!(
                dev_type,
                ZCanDeviceType::ZCAN_USBCAN1 | ZCanDeviceType::ZCAN_USBCAN2
            ) {
                // set channel protocol
                let protocol_path = format!("{}/{}", channel, PROTOCOL);
                let protocol_path = CmdPath::new_path(protocol_path.as_str());
                let value = CString::new((can_type as u32).to_string())
                    .map_err(|e| CanError::OtherError(e.to_string()))?;
                self.set_value(context, &protocol_path, value.as_ptr() as *const c_void)?;
            }

            // set channel bitrate
            let bitrate = cfg.nominal_bitrate;
            if dev_type.canfd_support() {
                let abitrate_path = format!("{}/{}", channel, CANFD_ABIT_BAUD_RATE);
                let abitrate_path = CmdPath::new_path(abitrate_path.as_str());
                let value = CString::new(bitrate.to_string())
                    .map_err(|e| CanError::OtherError(e.to_string()))?;
                self.set_value(context, &abitrate_path, value.as_ptr() as *const c_void)?;
                match can_type {
                    ZCanChlType::CANFD_ISO | ZCanChlType::CANFD_NON_ISO => {
                        let dbitrate = cfg.data_bitrate.unwrap_or(bitrate);
                        let dbitrate_path = format!("{}/{}", channel, CANFD_DBIT_BAUD_RATE);
                        let dbitrate_path = CmdPath::new_path(dbitrate_path.as_str());
                        let value = CString::new(dbitrate.to_string())
                            .map_err(|e| CanError::OtherError(e.to_string()))?;
                        self.set_value(context, &dbitrate_path, value.as_ptr() as *const c_void)?;
                    }
                    _ => {}
                }
            } else if !context.device.is_derive {
                let bitrate_path = format!("{}/{}", channel, BAUD_RATE);
                let bitrate_path = CmdPath::new_path(bitrate_path.as_str());
                let value = CString::new(bitrate.to_string())
                    .map_err(|e| CanError::OtherError(e.to_string()))?;
                self.set_value(context, &bitrate_path, value.as_ptr() as *const c_void)?;
            }

            let _cfg = ZCanChlCfg::new(dev_type, can_type, bc_ctx, cfg)?;
            match (self.ZCAN_InitCAN)(context.device_handler()?, channel as u32, &_cfg) {
                Self::INVALID_CHANNEL_HANDLE => Err(CanError::OperationError(format!(
                    "`ZCAN_InitCAN` ret = {}",
                    Self::INVALID_CHANNEL_HANDLE
                ))),
                handler => match (self.ZCAN_StartCAN)(handler) {
                    Self::STATUS_OK => {
                        context.chl_hdl = Some(handler);
                        Ok(())
                    }
                    code => Err(CanError::OperationError(format!(
                        "`ZCAN_StartCAN` ret = {}",
                        code
                    ))),
                },
            }
        }
    }

    fn reset_can_chl(&self, context: &ZChannelContext) -> CanResult<()> {
        match unsafe { (self.ZCAN_ResetCAN)(context.channel_handler()?) } {
            Self::STATUS_OK => Ok(()),
            code => Err(CanError::OperationError(format!(
                "`ZCAN_ResetCAN` ret = {}",
                code
            ))),
        }
    }

    fn read_can_chl_status(&self, context: &ZChannelContext) -> CanResult<ZCanChlStatus> {
        let mut status: ZCanChlStatus = Default::default();
        match unsafe { (self.ZCAN_ReadChannelStatus)(context.channel_handler()?, &mut status) } {
            Self::STATUS_OK => Ok(status),
            code => Err(CanError::OperationError(format!(
                "`ZCAN_ReadChannelStatus` ret = {}",
                code
            ))),
        }
    }

    fn read_can_chl_error(&self, context: &ZChannelContext) -> CanResult<ZCanChlError> {
        let mut info: ZCanChlError = ZCanChlError {
            v1: Default::default(),
        };
        match unsafe { (self.ZCAN_ReadChannelErrInfo)(context.channel_handler()?, &mut info) } {
            Self::STATUS_OK => Ok(info),
            code => Err(CanError::OperationError(format!(
                "`ZCAN_ReadChannelErrInfo` ret = {}",
                code
            ))),
        }
    }

    fn clear_can_buffer(&self, context: &ZChannelContext) -> CanResult<()> {
        match unsafe { (self.ZCAN_ClearBuffer)(context.channel_handler()?) } {
            Self::STATUS_OK => Ok(()),
            code => Err(CanError::OperationError(format!(
                "`ZCAN_ClearBuffer` ret = {}",
                code
            ))),
        }
    }

    fn get_can_num(&self, context: &ZChannelContext, can_type: ZCanFrameType) -> CanResult<u32> {
        let ret = unsafe { (self.ZCAN_GetReceiveNum)(context.channel_handler()?, can_type as u8) };
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
    ) -> CanResult<Vec<ZCanFrame>> {
        let mut frames = vec![ZCanFrameRx::<{ MAX_FRAME_SIZE }>::default(); size as usize];

        let ret = unsafe {
            (self.ZCAN_Receive)(
                context.channel_handler()?,
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
            .take(ret as usize)
            .map(|mut frame| {
                frame.frame.set_channel(context.channel);
                frame.into()
            })
            .collect::<Vec<_>>())
    }

    fn transmit_can(&self, context: &ZChannelContext, frames: Vec<ZCanFrame>) -> CanResult<u32> {
        let frames = frames
            .into_iter()
            .map(ZCanFrameTx::<{ MAX_FRAME_SIZE }>::from)
            .collect::<Vec<_>>();

        let len = frames.len() as u32;
        let chl_hdl = context.channel_handler()?;
        let ret = unsafe { (self.ZCAN_Transmit)(chl_hdl, frames.as_ptr(), len) };
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

    fn receive_canfd(
        &self,
        context: &ZChannelContext,
        size: u32,
        timeout: u32,
    ) -> CanResult<Vec<ZCanFrame>> {
        let mut frames = vec![ZCanFrameRx::<{ MAX_FD_FRAME_SIZE }>::default(); size as usize];

        let ret = unsafe {
            (self.ZCAN_ReceiveFD)(
                context.channel_handler()?,
                frames.as_mut_ptr(),
                size,
                timeout,
            )
        };
        if ret < size {
            rsutil::warn!(
                "ZLGCAN - receive CANFD frame expect: {}, actual: {}!",
                size,
                ret
            );
        } else if ret > 0 {
            rsutil::trace!("ZLGCAN - receive CANFD frame: {}", ret);
        }

        Ok(frames
            .into_iter()
            .take(ret as usize)
            .map(|mut frame| {
                frame.frame.set_channel(context.channel);
                frame.into()
            })
            .collect::<Vec<_>>())
    }

    fn transmit_canfd(&self, context: &ZChannelContext, frames: Vec<ZCanFrame>) -> CanResult<u32> {
        let frames = frames
            .into_iter()
            .map(ZCanFrameTx::<{ MAX_FD_FRAME_SIZE }>::from)
            .collect::<Vec<_>>();

        let len = frames.len() as u32;
        let chl_hdl = context.channel_handler()?;
        let ret = unsafe { (self.ZCAN_TransmitFD)(chl_hdl, frames.as_ptr(), len) };
        if ret < len {
            rsutil::warn!(
                "ZLGCAN - transmit CAN-FD frame expect: {}, actual: {}!",
                len,
                ret
            );
        } else {
            rsutil::trace!("ZLGCAN - transmit CAN-FD frame: {}", ret);
        }
        Ok(ret)
    }
}

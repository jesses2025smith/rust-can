use std::ffi::c_void;
use dlopen2::symbor::{Symbol, SymBorApi};
use rs_can::{CanError, ChannelConfig};

use crate::can::{ZCanFrame, ZCanChlError, ZCanChlStatus, ZCanFrameType, ZCanFrameInner, CanMessage};
use crate::device::{CmdPath, ZChannelContext, ZDeviceContext, ZDeviceInfo};
use crate::api::{ZCanApi, ZCloudApi, ZDeviceApi, ZLinApi};
use crate::can::{common::{ZCanChlCfgInner, CanChlCfgContext}, constant::BITRATE_CFG_FILENAME};

#[allow(non_snake_case)]
#[derive(Debug, Clone, SymBorApi)]
pub(crate) struct USBCANApi<'a> {
    /// EXTERN_C DWORD VCI_OpenDevice(DWORD DeviceType,DWORD DeviceInd,DWORD Reserved);
    VCI_OpenDevice: Symbol<'a, unsafe extern "C" fn(dev_type: u32, dev_index: u32, reserved: u32) -> u32>,
    ///EXTERN_C DWORD VCI_CloseDevice(DWORD DeviceType,DWORD DeviceInd);
    VCI_CloseDevice: Symbol<'a, unsafe extern "C" fn(dev_type: u32, dev_index: u32) -> u32>,
    /// EXTERN_C DWORD VCI_InitCAN(DWORD DeviceType, DWORD DeviceInd, DWORD CANInd, PVCI_INIT_CONFIG pInitConfig);
    VCI_InitCAN: Symbol<'a, unsafe extern "C" fn(dev_type: u32, dev_index: u32, channel: u32, cfg: *const ZCanChlCfgInner) -> u32>,

    /// EXTERN_C DWORD VCI_ReadBoardInfo(DWORD DeviceType,DWORD DeviceInd,PVCI_BOARD_INFO pInfo);
    VCI_ReadBoardInfo: Symbol<'a, unsafe extern "C" fn(dev_type: u32, dev_index: u32, info: *mut ZDeviceInfo) -> u32>,
    /// EXTERN_C DWORD VCI_ReadErrInfo(DWORD DeviceType,DWORD DeviceInd,DWORD CANInd,PVCI_ERR_INFO pErrInfo);
    VCI_ReadErrInfo: Symbol<'a, unsafe extern "C" fn(dev_type: u32, dev_index: u32, channel: u32, err: *mut ZCanChlError) -> u32>,
    /// EXTERN_C DWORD VCI_ReadCANStatus(DWORD DeviceType,DWORD DeviceInd,DWORD CANInd,PVCI_CAN_STATUS pCANStatus);
    VCI_ReadCANStatus: Symbol<'a, unsafe extern "C" fn(dev_type: u32, dev_index: u32, channel: u32, status: *mut ZCanChlStatus) -> u32>,
    /// EXTERN_C DWORD VCI_GetReference(DWORD DeviceType,DWORD DeviceInd,DWORD CANInd,DWORD RefType,PVOID pData);
    VCI_GetReference: Symbol<'a, unsafe extern "C" fn(dev_type: u32, dev_index: u32, channel: u32, cmd: u32, value: *mut c_void) -> u32>,
    /// EXTERN_C DWORD VCI_SetReference(DWORD DeviceType,DWORD DeviceInd,DWORD CANInd,DWORD RefType,PVOID pData);
    VCI_SetReference: Symbol<'a, unsafe extern "C" fn(dev_type: u32, dev_index: u32, channel: u32, cmd: u32, value: *const c_void) -> u32>,
    /// EXTERN_C ULONG VCI_GetReceiveNum(DWORD DeviceType,DWORD DeviceInd,DWORD CANInd);
    VCI_GetReceiveNum: Symbol<'a, unsafe extern "C" fn(dev_type: u32, dev_index: u32, channel: u32) -> u32>,
    /// EXTERN_C DWORD VCI_ClearBuffer(DWORD DeviceType,DWORD DeviceInd,DWORD CANInd);
    VCI_ClearBuffer: Symbol<'a, unsafe extern "C" fn(dev_type: u32, dev_index: u32, channel: u32) -> u32>,
    /// EXTERN_C DWORD VCI_StartCAN(DWORD DeviceType,DWORD DeviceInd,DWORD CANInd);
    VCI_StartCAN: Symbol<'a, unsafe extern "C" fn(dev_type: u32, dev_index: u32, channel: u32) -> u32>,
    /// EXTERN_C DWORD VCI_ResetCAN(DWORD DeviceType,DWORD DeviceInd,DWORD CANInd);
    VCI_ResetCAN: Symbol<'a, unsafe extern "C" fn(dev_type: u32, dev_index: u32, channel: u32) -> u32>,
    /// EXTERN_C ULONG VCI_Transmit(DWORD DeviceType,DWORD DeviceInd,DWORD CANInd,PVCI_CAN_OBJ pSend,UINT Len);
    VCI_Transmit: Symbol<'a, unsafe extern "C" fn(dev_type: u32, dev_index: u32, channel: u32, frames: *const ZCanFrame, len: u32) -> u32>,
    /// EXTERN_C ULONG VCI_Receive(DWORD DeviceType,DWORD DeviceInd,DWORD CANInd,PVCI_CAN_OBJ pReceive,UINT Len,INT WaitTime);
    VCI_Receive: Symbol<'a, unsafe extern "C" fn(dev_type: u32, dev_index: u32, channel: u32, frames: *mut ZCanFrame, size: u32, timeout: u32) -> u32>,
}

impl USBCANApi<'_> {
    // const INVALID_DEVICE_HANDLE: u32 = 0;
    // const INVALID_CHANNEL_HANDLE: u32 = 0;
    const STATUS_OK: u32 = 1;
}

impl ZDeviceApi for USBCANApi<'_> {
    fn open(&self, context: &mut ZDeviceContext) -> Result<(), CanError> {
        let (dev_type, dev_idx) = (context.device_type(), context.device_index());
        match unsafe { (self.VCI_OpenDevice)(dev_type as u32, dev_idx, 0) } {
            Self::STATUS_OK => Ok(()),
            code => Err(CanError::InitializeError(format!("`VCI_OpenDevice` ret: {}", code))),
        }
    }

    fn close(&self, context: &ZDeviceContext) -> Result<(), CanError> {
        let (dev_type, dev_idx) = (context.device_type(), context.device_index());
        match unsafe { (self.VCI_CloseDevice)(dev_type as u32, dev_idx) } {
            Self::STATUS_OK => Ok(()),
            code => Err(CanError::OperationError(format!("`VCI_CloseDevice` ret: {}", code))),
        }
    }

    fn read_device_info(&self, context: &ZDeviceContext) -> Result<ZDeviceInfo, CanError> {
        let mut info = ZDeviceInfo::default();
        let (dev_type, dev_idx) = (context.device_type(), context.device_index());
        match unsafe { (self.VCI_ReadBoardInfo)(dev_type as u32, dev_idx, &mut info) } {
            Self::STATUS_OK => Ok(info),
            code => Err(CanError::OperationError(format!("`VCI_ReadBoardInfo` ret: {}", code))),
        }
    }

    fn set_reference(&self, context: &ZChannelContext, cmd_path: &CmdPath, value: *const c_void) -> Result<(), CanError> {
        let (dev_type, dev_idx, channel) = (context.device_type(), context.device_index(), context.channel());
        let cmd = cmd_path.get_reference();
        match unsafe { (self.VCI_SetReference)(dev_type as u32, dev_idx, channel as u32, cmd, value) } {
            Self::STATUS_OK => Ok(()),
            code => Err(CanError::OperationError(format!("`VCI_SetReference` ret: {}", code))),
        }
    }

    fn get_reference(&self, context: &ZChannelContext, cmd_path: &CmdPath, value: *mut c_void) -> Result<(), CanError> {
        let (dev_type, dev_idx, channel) = (context.device_type(), context.device_index(), context.channel());
        let cmd = cmd_path.get_reference();
        match unsafe { (self.VCI_GetReference)(dev_type as u32, dev_idx, channel as u32, cmd, value) } {
            Self::STATUS_OK => Ok(()),
            code => Err(CanError::OperationError(format!("`VCI_GetReference` ret: {}", code))),
        }
    }
}

impl ZCanApi for USBCANApi<'_> {
    fn init_can_chl(&self, libpath: &str, context: &mut ZChannelContext, cfg: &ChannelConfig) -> Result<(), CanError> {
        let (dev_type, dev_idx, channel) = (context.device_type(), context.device_index(), context.channel());
        let cfg_ctx = CanChlCfgContext::new(libpath)?;
        let bc_ctx = cfg_ctx.0.get(&(dev_type as u32).to_string())
            .ok_or(CanError::InitializeError(
                format!("device: {} is not configured in {}", dev_type, BITRATE_CFG_FILENAME)
            ))?;
        unsafe {
            let dev_type = dev_type as u32;
            let channel = channel as u32;

            let bitrate = cfg.bitrate();
            let cfg = ZCanChlCfgInner::try_from_with(bc_ctx, cfg)?;
            match (self.VCI_InitCAN)(dev_type, dev_idx, channel, &cfg) {
                Self::STATUS_OK => {
                    match (self.VCI_StartCAN)(dev_type, dev_idx, channel) {
                        Self::STATUS_OK => {
                            context.set_channel_handler(None);
                            Ok(())
                        },
                        code => Err(CanError::InitializeError(format!("`VCI_StartCAN` ret: {}", code))),
                    }
                },
                code => Err(CanError::InitializeError(format!("`VCI_InitCAN` ret: {}", code))),
            }
        }
    }

    fn reset_can_chl(&self, context: &ZChannelContext) -> Result<(), CanError> {
        let (dev_type, dev_idx, channel) = (context.device_type(), context.device_index(), context.channel());
        match unsafe { (self.VCI_ResetCAN)(dev_type as u32, dev_idx, channel as u32) } {
            Self::STATUS_OK => Ok(()),
            code => Err(CanError::OperationError(format!("`VCI_ResetCAN` ret: {}", code))),
        }
    }

    fn read_can_chl_status(&self, context: &ZChannelContext) -> Result<ZCanChlStatus, CanError> {
        let (dev_type, dev_idx, channel) = (context.device_type(), context.device_index(), context.channel());
        let mut status: ZCanChlStatus = Default::default();
        match unsafe { (self.VCI_ReadCANStatus)(dev_type as u32, dev_idx, channel as u32, &mut status) } {
            Self::STATUS_OK => Ok(status),
            code => Err(CanError::OperationError(format!("`VCI_ReadCANStatus` ret: {}", code))),
        }
    }

    fn read_can_chl_error(&self, context: &ZChannelContext) -> Result<ZCanChlError, CanError> {
        let (dev_type, dev_idx, channel) = (context.device_type(), context.device_index(), context.channel());
        let mut info = ZCanChlError { v1: Default::default() };
        match unsafe { (self.VCI_ReadErrInfo)(dev_type as u32, dev_idx, channel as u32, &mut info) } {
            Self::STATUS_OK => Ok(info),
            code => Err(CanError::OperationError(format!("`VCI_ReadErrInfo` ret: {}", code))),
        }
    }

    fn clear_can_buffer(&self, context: &ZChannelContext) -> Result<(), CanError> {
        let (dev_type, dev_idx, channel) = (context.device_type(), context.device_index(), context.channel());
        match unsafe { (self.VCI_ClearBuffer)(dev_type as u32, dev_idx, channel as u32) } {
            Self::STATUS_OK => Ok(()),
            code => Err(CanError::OperationError(format!("`VCI_ClearBuffer` ret: {}", code))),
        }
    }

    fn get_can_num(&self, context: &ZChannelContext, can_type: ZCanFrameType) -> Result<u32, CanError> {
        let (dev_type, dev_idx, channel) = (context.device_type(), context.device_index(), context.channel());
        let mut _channel = channel as u32;
        match can_type {
            ZCanFrameType::CAN => {},
            ZCanFrameType::CANFD => _channel |= 0x8000_0000,
            ZCanFrameType::ALL => return Err(CanError::other_error("method not supported")),
        }
        let ret = unsafe { (self.VCI_GetReceiveNum)(dev_type as u32, dev_idx, _channel) };
        if ret > 0 {
            log::trace!("ZLGCAN - get receive {} number: {}.", can_type, ret);
        }
        Ok(ret)
    }

    fn receive_can(&self, context: &ZChannelContext, size: u32, timeout: u32) -> Result<Vec<CanMessage>, CanError> {
        let (dev_type, dev_idx, channel) = (context.device_type(), context.device_index(), context.channel());
        let mut frames = Vec::new();
        frames.resize(size as usize, ZCanFrame { can: ZCanFrameInner { libusbcan: Default::default() } });

        let ret = unsafe { (self.VCI_Receive)(dev_type as u32, dev_idx, channel as u32, frames.as_mut_ptr(), size, timeout) };
        if ret < size {
            log::warn!("ZLGCAN - receive CAN frame expect: {}, actual: {}!", size, ret);
        }
        else if ret > 0 {
            log::trace!("ZLGCAN - receive CAN frame: {}", ret);
        }

        Ok(frames.into_iter()
            .map(|mut frame| unsafe {
                frame.can.libusbcan.into()
            })
            .collect::<Vec<_>>())
    }

    fn transmit_can(&self, context: &ZChannelContext, frames: Vec<CanMessage>) -> Result<u32, CanError> {
        let frames = frames.into_iter()
            .map(|frame| ZCanFrame { can: ZCanFrameInner { libusbcan: frame.into() } })
            .collect::<Vec<_>>();

        let (dev_type, dev_idx, channel) = (context.device_type(), context.device_index(), context.channel());
        let len = frames.len() as u32;
        let ret = unsafe { (self.VCI_Transmit)(dev_type as u32, dev_idx, channel as u32, frames.as_ptr(), len) };
        if ret < len {
            log::warn!("ZLGCAN - transmit CAN frame expect: {}, actual: {}!", len, ret);
        }
        else {
            log::trace!("ZLGCAN - transmit CAN frame: {}", ret);
        }
        Ok(ret)
    }
}

impl ZLinApi for USBCANApi<'_> {}
impl ZCloudApi for USBCANApi<'_> {}

#[cfg(test)]
mod tests {
    use dlopen2::symbor::{Library, SymBorApi};
    use rs_can::{CanError, CanFrame, CanId, ChannelConfig};
    use crate::can::{ZCanChlMode, ZCanChlType, ZCanFrame, CanMessage, ZCanFrameInner};
    use crate::constants::LOAD_LIB_FAILED;
    use crate::device::{ZCanDeviceType, ZChannelContext, ZDeviceContext};
    use super::USBCANApi;
    use crate::api::{ZCanApi, ZDeviceApi};
    use crate::{CHANNEL_MODE, CHANNEL_TYPE};

    #[test]
    fn test_init_channel() -> anyhow::Result<()> {
        let dev_type = ZCanDeviceType::ZCAN_USBCAN1;
        let dev_idx = 0;
        let channel = 0;

        let so_path = "library/linux/x86_64/libusbcan.so";
        let lib = Library::open(so_path).expect(LOAD_LIB_FAILED);

        let api = unsafe { USBCANApi::load(&lib) }.expect("ZLGCAN - could not load symbols!");

        let mut cfg = ChannelConfig::new(500_000);
        cfg.add_other(CHANNEL_TYPE, Box::new(ZCanChlType::CAN as u8))
            .add_other(CHANNEL_MODE, Box::new(ZCanChlMode::Normal as u8));

        let mut context = ZDeviceContext::new(dev_type, dev_idx, false);
        api.open(&mut context)?;

        let dev_info = api.read_device_info(&context)?;
        println!("{:?}", dev_info);
        println!("{}", dev_info.id());
        println!("{}", dev_info.sn());
        println!("{}", dev_info.hardware_version());
        println!("{}", dev_info.firmware_version());
        println!("{}", dev_info.driver_version());
        println!("{}", dev_info.api_version());
        assert_eq!(dev_info.can_channels(), 1);
        assert!(!dev_info.canfd());

        let mut context = ZChannelContext::new(context, channel);
        api.init_can_chl("library", &mut context, &cfg)?;
        let frame = CanMessage::new(
            CanId::from_bits(0x7E0, Some(false)),
            [0x01, 0x02, 0x03].as_slice()
        )
            .ok_or(CanError::other_error("invalid data length"))?;
        let frame1 = CanMessage::new(
            CanId::from_bits(0x1888FF00, Some(true)),
            [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08].as_slice()
        )
            .ok_or(CanError::other_error("invalid data length"))?;
        let frames = vec![frame, frame1];
        let ret = api.transmit_can(&context, frames)?;
        assert_eq!(ret, 2);

        api.reset_can_chl(&context)?;

        api.close(context.device_context())?;

        Ok(())
    }
}


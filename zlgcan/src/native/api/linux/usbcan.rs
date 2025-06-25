use std::ffi::c_void;
use dlopen2::symbor::{Symbol, SymBorApi};

use crate::native::{
    can::{common::ZCanChlCfgInner, ZCanFrame, ZCanChlError, ZCanChlStatus},
    device::ZDeviceInfo,
};

#[allow(non_snake_case)]
#[derive(Debug, Clone, SymBorApi)]
pub(crate) struct USBCANApi<'a> {
    /// EXTERN_C DWORD VCI_OpenDevice(DWORD DeviceType,DWORD DeviceInd,DWORD Reserved);
    pub(crate) VCI_OpenDevice: Symbol<'a, unsafe extern "C" fn(dev_type: u32, dev_index: u32, reserved: u32) -> u32>,
    ///EXTERN_C DWORD VCI_CloseDevice(DWORD DeviceType,DWORD DeviceInd);
    pub(crate) VCI_CloseDevice: Symbol<'a, unsafe extern "C" fn(dev_type: u32, dev_index: u32) -> u32>,
    /// EXTERN_C DWORD VCI_InitCAN(DWORD DeviceType, DWORD DeviceInd, DWORD CANInd, PVCI_INIT_CONFIG pInitConfig);
    pub(crate) VCI_InitCAN: Symbol<'a, unsafe extern "C" fn(dev_type: u32, dev_index: u32, channel: u32, cfg: *const ZCanChlCfgInner) -> u32>,

    /// EXTERN_C DWORD VCI_ReadBoardInfo(DWORD DeviceType,DWORD DeviceInd,PVCI_BOARD_INFO pInfo);
    pub(crate) VCI_ReadBoardInfo: Symbol<'a, unsafe extern "C" fn(dev_type: u32, dev_index: u32, info: *mut ZDeviceInfo) -> u32>,
    /// EXTERN_C DWORD VCI_ReadErrInfo(DWORD DeviceType,DWORD DeviceInd,DWORD CANInd,PVCI_ERR_INFO pErrInfo);
    pub(crate) VCI_ReadErrInfo: Symbol<'a, unsafe extern "C" fn(dev_type: u32, dev_index: u32, channel: u32, err: *mut ZCanChlError) -> u32>,
    /// EXTERN_C DWORD VCI_ReadCANStatus(DWORD DeviceType,DWORD DeviceInd,DWORD CANInd,PVCI_CAN_STATUS pCANStatus);
    pub(crate) VCI_ReadCANStatus: Symbol<'a, unsafe extern "C" fn(dev_type: u32, dev_index: u32, channel: u32, status: *mut ZCanChlStatus) -> u32>,
    /// EXTERN_C DWORD VCI_GetReference(DWORD DeviceType,DWORD DeviceInd,DWORD CANInd,DWORD RefType,PVOID pData);
    pub(crate) VCI_GetReference: Symbol<'a, unsafe extern "C" fn(dev_type: u32, dev_index: u32, channel: u32, cmd: u32, value: *mut c_void) -> u32>,
    /// EXTERN_C DWORD VCI_SetReference(DWORD DeviceType,DWORD DeviceInd,DWORD CANInd,DWORD RefType,PVOID pData);
    pub(crate) VCI_SetReference: Symbol<'a, unsafe extern "C" fn(dev_type: u32, dev_index: u32, channel: u32, cmd: u32, value: *const c_void) -> u32>,
    /// EXTERN_C ULONG VCI_GetReceiveNum(DWORD DeviceType,DWORD DeviceInd,DWORD CANInd);
    pub(crate) VCI_GetReceiveNum: Symbol<'a, unsafe extern "C" fn(dev_type: u32, dev_index: u32, channel: u32) -> u32>,
    /// EXTERN_C DWORD VCI_ClearBuffer(DWORD DeviceType,DWORD DeviceInd,DWORD CANInd);
    pub(crate) VCI_ClearBuffer: Symbol<'a, unsafe extern "C" fn(dev_type: u32, dev_index: u32, channel: u32) -> u32>,
    /// EXTERN_C DWORD VCI_StartCAN(DWORD DeviceType,DWORD DeviceInd,DWORD CANInd);
    pub(crate) VCI_StartCAN: Symbol<'a, unsafe extern "C" fn(dev_type: u32, dev_index: u32, channel: u32) -> u32>,
    /// EXTERN_C DWORD VCI_ResetCAN(DWORD DeviceType,DWORD DeviceInd,DWORD CANInd);
    pub(crate) VCI_ResetCAN: Symbol<'a, unsafe extern "C" fn(dev_type: u32, dev_index: u32, channel: u32) -> u32>,
    /// EXTERN_C ULONG VCI_Transmit(DWORD DeviceType,DWORD DeviceInd,DWORD CANInd,PVCI_CAN_OBJ pSend,UINT Len);
    pub(crate) VCI_Transmit: Symbol<'a, unsafe extern "C" fn(dev_type: u32, dev_index: u32, channel: u32, frames: *const ZCanFrame, len: u32) -> u32>,
    /// EXTERN_C ULONG VCI_Receive(DWORD DeviceType,DWORD DeviceInd,DWORD CANInd,PVCI_CAN_OBJ pReceive,UINT Len,INT WaitTime);
    pub(crate) VCI_Receive: Symbol<'a, unsafe extern "C" fn(dev_type: u32, dev_index: u32, channel: u32, frames: *mut ZCanFrame, size: u32, timeout: u32) -> u32>,
}

impl USBCANApi<'_> {
    // const INVALID_DEVICE_HANDLE: u32 = 0;
    // const INVALID_CHANNEL_HANDLE: u32 = 0;
    pub(crate) const STATUS_OK: u32 = 1;
}

#[cfg(test)]
mod tests {
    use dlopen2::symbor::{Library, SymBorApi};
    use rs_can::{CanError, CanFrame, CanId, ChannelConfig};
    use crate::{
        api::{ZCanApi, ZChannelContext, ZDeviceApi, ZDeviceContext},
        can::{CanMessage, ZCanChlType, ZCanChlMode},
        constants,
        native::{
            device::ZCanDeviceType,
            constants::LOAD_LIB_FAILED
        },
    };
    use super::USBCANApi;

    #[test]
    fn test_init_channel() -> anyhow::Result<()> {
        let dev_type = ZCanDeviceType::ZCAN_USBCAN1;
        let dev_idx = 0;
        let channel = 0;

        let so_path = "library/linux/x86_64/libusbcan.so";
        let lib = Library::open(so_path).expect(LOAD_LIB_FAILED);

        let api = unsafe { USBCANApi::load(&lib) }.expect("ZLGCAN - could not load symbols!");

        let mut cfg = ChannelConfig::new(500_000);
        cfg.add_other(constants::CHANNEL_TYPE, Box::new(ZCanChlType::CAN))
            .add_other(constants::CHANNEL_MODE, Box::new(ZCanChlMode::Normal));

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

        api.close(&context.device)?;

        Ok(())
    }
}


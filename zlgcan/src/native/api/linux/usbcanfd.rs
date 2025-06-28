use crate::native::{
    can::{ZCanChlError, ZCanChlStatus, ZCanFdChlCfgInner, ZCanFrame},
    device::ZDeviceInfo,
    lin::{ZLinChlCfg, ZLinFrame, ZLinPublish, ZLinSubscribe},
};
use dlopen2::symbor::{SymBorApi, Symbol};
use std::ffi::{c_uint, c_void};

#[rustfmt::skip]
#[allow(non_snake_case)]
#[derive(Debug, Clone, SymBorApi)]
pub(crate) struct USBCANFDApi<'a> {
    ///EXTERN_C U32 ZCAN_API VCI_OpenDevice(U32 Type, U32 Card, U32 Reserved);
    pub(crate) VCI_OpenDevice: Symbol<'a, unsafe extern "C" fn(dev_type: c_uint, dev_idx: c_uint, reserved: c_uint) -> c_uint>,
    /// EXTERN_C U32 ZCAN_API VCI_CloseDevice(U32 Type, U32 Card);
    pub(crate) VCI_CloseDevice: Symbol<'a, unsafe extern "C" fn(dev_type: c_uint, dev_idx: c_uint) -> c_uint>,
    /// EXTERN_C U32 ZCAN_API VCI_InitCAN(U32 Type, U32 Card, U32 Port, ZCAN_INIT *pInit);
    pub(crate) VCI_InitCAN: Symbol<'a, unsafe extern "C" fn(dev_type: c_uint, dev_idx: c_uint, channel: c_uint, cfg: *const ZCanFdChlCfgInner) -> c_uint>,
    /// EXTERN_C U32 ZCAN_API VCI_ReadBoardInfo(U32 Type, U32 Card, ZCAN_DEV_INF *pInfo);
    pub(crate) VCI_ReadBoardInfo: Symbol<'a, unsafe extern "C" fn(dev_type: c_uint, dev_idx: c_uint, info: *mut ZDeviceInfo) -> c_uint>,
    /// EXTERN_C U32 ZCAN_API VCI_ReadErrInfo(U32 Type, U32 Card, U32 Port, ZCAN_ERR_MSG *pErr);
    pub(crate) VCI_ReadErrInfo: Symbol<'a, unsafe extern "C" fn(dev_type: c_uint, dev_idx: c_uint, channel: c_uint, err: *mut ZCanChlError) -> c_uint>,
    /// EXTERN_C U32 ZCAN_API VCI_ReadCANStatus(U32 Type, U32 Card, U32 Port, ZCAN_STAT *pStat);
    pub(crate) VCI_ReadCANStatus: Symbol<'a, unsafe extern "C" fn(dev_type: c_uint, dev_idx: c_uint, channel: c_uint, status: *mut ZCanChlStatus) -> c_uint>,
    /// EXTERN_C U32 ZCAN_API VCI_GetReference(U32 Type, U32 Card, U32 Port, U32 Ref, void *pData);
    pub(crate) VCI_GetReference: Symbol<'a, unsafe extern "C" fn(dev_type: c_uint, dev_idx: c_uint, channel: c_uint, cmd: c_uint, value: *mut c_void) -> c_uint>,
    /// EXTERN_C U32 ZCAN_API VCI_SetReference(U32 Type, U32 Card, U32 Port, U32 Ref, void *pData);
    pub(crate) VCI_SetReference: Symbol<'a, unsafe extern "C" fn(dev_type: c_uint, dev_idx: c_uint, channel: c_uint, cmd: c_uint, value: *const c_void) -> c_uint>,
    /// EXTERN_C U32 ZCAN_API VCI_GetReceiveNum(U32 Type, U32 Card, U32 Port);
    pub(crate) VCI_GetReceiveNum: Symbol<'a, unsafe extern "C" fn(dev_type: c_uint, dev_idx: c_uint, channel: c_uint) -> c_uint>,
    /// EXTERN_C U32 ZCAN_API VCI_ClearBuffer(U32 Type, U32 Card, U32 Port);
    pub(crate) VCI_ClearBuffer: Symbol<'a, unsafe extern "C" fn(dev_type: c_uint, dev_idx: c_uint, channel: c_uint) -> c_uint>,
    /// EXTERN_C U32 ZCAN_API VCI_StartCAN(U32 Type, U32 Card, U32 Port);
    pub(crate) VCI_StartCAN: Symbol<'a, unsafe extern "C" fn(dev_type: c_uint, dev_idx: c_uint, channel: c_uint) -> c_uint>,
    /// EXTERN_C U32 ZCAN_API VCI_ResetCAN(U32 Type, U32 Card, U32 Port);
    pub(crate) VCI_ResetCAN: Symbol<'a, unsafe extern "C" fn(dev_type: c_uint, dev_idx: c_uint, channel: c_uint) -> c_uint>,
    /// EXTERN_C U32 ZCAN_API VCI_Transmit(U32 Type, U32 Card, U32 Port, ZCAN_20_MSG *pData, U32 Count);
    pub(crate) VCI_Transmit: Symbol<'a, unsafe extern "C" fn(dev_type: c_uint, dev_idx: c_uint, channel: c_uint, frames: *const ZCanFrame, len: c_uint) -> c_uint>,
    /// EXTERN_C U32 ZCAN_API VCI_TransmitFD(U32 Type, U32 Card, U32 Port, ZCAN_FD_MSG *pData, U32 Count);
    pub(crate) VCI_TransmitFD: Symbol<'a, unsafe extern "C" fn(dev_type: c_uint, dev_idx: c_uint, channel: c_uint, frames: *const ZCanFrame, len: c_uint) -> c_uint>,
    /// EXTERN_C U32 ZCAN_API VCI_Receive(U32 Type, U32 Card, U32 Port, ZCAN_20_MSG *pData, U32 Count, U32 Time);
    pub(crate) VCI_Receive: Symbol<'a, unsafe extern "C" fn(dev_type: c_uint, dev_idx: c_uint, channel: c_uint, frames: *mut ZCanFrame, size: c_uint, timeout: c_uint) -> c_uint>,
    /// EXTERN_C U32 ZCAN_API VCI_ReceiveFD(U32 Type, U32 Card, U32 Port, ZCAN_FD_MSG *pData, U32 Count, U32 Time);
    pub(crate) VCI_ReceiveFD: Symbol<'a, unsafe extern "C" fn(dev_type: c_uint, dev_idx: c_uint, channel: c_uint, frames: *mut ZCanFrame, size: c_uint, timeout: c_uint) -> c_uint>,
    /// EXTERN_C U32 ZCAN_API VCI_Debug(U32 Debug);
    pub(crate) VCI_Debug: Symbol<'a, unsafe extern "C" fn(debug: c_uint) -> c_uint>,

    /// UINT VCI_InitLIN(U32 Type, U32 Card, U32 LinChn, PZCAN_LIN_INIT_CONFIG pLINInitConfig);
    pub(crate) VCI_InitLIN: Symbol<'a, unsafe extern "C" fn(dev_type: c_uint, dev_idx: c_uint, channel: c_uint, cfg: *const ZLinChlCfg) -> c_uint>,
    /// UINT VCI_StartLIN(U32 Type, U32 Card, U32 LinChn);
    pub(crate) VCI_StartLIN: Symbol<'a, unsafe extern "C" fn(dev_type: c_uint, dev_idx: c_uint, channel: c_uint) -> c_uint>,
    /// UINT VCI_ResetLIN(U32 Type, U32 Card, U32 LinChn);
    pub(crate) VCI_ResetLIN: Symbol<'a, unsafe extern "C" fn(dev_type: c_uint, dev_idx: c_uint, channel: c_uint) -> c_uint>,
    /// UINT VCI_TransmitLIN(U32 Type, U32 Card, U32 LinChn, PZCAN_LIN_MSG pSend, U32 Len);
    pub(crate) VCI_TransmitLIN: Symbol<'a, unsafe extern "C" fn(dev_type: c_uint, dev_idx: c_uint, channel: c_uint, frames: *const ZLinFrame, len: c_uint) -> c_uint>,
    /// UINT VCI_GetLINReceiveNum(U32 Type, U32 Card, U32 LinChn);
    pub(crate) VCI_GetLINReceiveNum: Symbol<'a, unsafe extern "C" fn(dev_type: c_uint, dev_idx: c_uint, channel: c_uint) -> c_uint>,
    /// EXTERN_C U32 VCI_ClearLINBuffer(U32 Type, U32 Card, U32 LinChn);
    pub(crate) VCI_ClearLINBuffer: Symbol<'a, unsafe extern "C" fn(dev_type: c_uint, dev_idx: c_uint, channel: c_uint) -> c_uint>,
    /// UINT VCI_ReceiveLIN(U32 Type, U32 Card, U32 LinChn, PZCAN_LIN_MSG pReceive, U32 Len,int WaitTime);
    pub(crate) VCI_ReceiveLIN: Symbol<'a, unsafe extern "C" fn(dev_type: c_uint, dev_idx: c_uint, channel: c_uint, frames: *mut ZLinFrame, size: c_uint, timeout: c_uint) -> c_uint>,
    /// UINT  VCI_SetLINSubscribe(U32 Type, U32 Card, U32 LinChn, PZCAN_LIN_SUBSCIBE_CFG pSend, U32 nSubscribeCount);
    pub(crate) VCI_SetLINSubscribe: Symbol<'a, unsafe extern "C" fn(dev_type: c_uint, dev_idx: c_uint, channel: c_uint, cfg: *const ZLinSubscribe, len: c_uint) -> c_uint>,
    /// UINT  VCI_SetLINPublish(U32 Type, U32 Card, U32 LinChn, PZCAN_LIN_PUBLISH_CFG pSend, U32 nPublishCount);
    pub(crate) VCI_SetLINPublish: Symbol<'a, unsafe extern "C" fn(dev_type: c_uint, dev_idx: c_uint, channel: c_uint, cfg: *const ZLinPublish, len: c_uint) -> c_uint>,

    // EXTERN_C U32 VCI_TransmitData(unsigned Type, unsigned Card, unsigned Port, ZCANDataObj *pData, unsigned Count);
    // VCI_TransmitData: Symbol<'a, unsafe extern "C" fn(dev_type: c_uint, dev_idx: c_uint, channel: c_uint, data: *const ZCANDataObj, len: c_uint) -> c_uint>,
    // EXTERN_C U32 VCI_ReceiveData(unsigned Type, unsigned Card, unsigned Port, ZCANDataObj *pData, unsigned Count, unsigned Time);
    // VCI_ReceiveData: Symbol<'a, unsafe extern "C" fn(dev_type: c_uint, dev_idx: c_uint, channel: c_uint, data: *mut ZCANDataObj, size: c_uint, timeout: c_uint) -> c_uint>,

    // EXTERN_C U32 VCI_UDS_Request(unsigned Type, unsigned Card, const ZCAN_UDS_REQUEST *req, ZCAN_UDS_RESPONSE *resp, U8 *dataBuf, U32 dataBufSize);
    // VCI_UDS_Request: Symbol<'a, unsafe extern "C" fn(dev_type: c_uint, dev_idx: c_uint, req: *const ZCAN_UDS_REQUEST, resp: *mut ZCAN_UDS_RESPONSE, buff: *mut c_uchar, buff_size: c_uint) -> c_uint>,
    // EXTERN_C U32 VCI_UDS_Control(unsigned Type, unsigned Card, const ZCAN_UDS_CTRL_REQ *ctrl, ZCAN_UDS_CTRL_RESP *resp);
    // VCI_UDS_Control: Symbol<'a, unsafe extern "C" fn(dev_type: c_uint, dev_idx: c_uint, req: *const ZCAN_UDS_REQUEST, resp: *mut ZCAN_UDS_RESPONSE) -> c_uint>,
}

impl USBCANFDApi<'_> {
    // const INVALID_DEVICE_HANDLE: u32 = 0;
    // const INVALID_CHANNEL_HANDLE: u32 = 0;
    pub(crate) const STATUS_OK: u32 = 1;
}

#[cfg(test)]
mod tests {
    use super::USBCANFDApi;
    use crate::{
        constants,
        native::{
            api::{ZCanApi, ZChannelContext, ZDeviceApi, ZDeviceContext},
            can::{CanMessage, ZCanChlMode, ZCanChlType},
            constants::LOAD_LIB_FAILED,
            device::ZCanDeviceType,
        },
    };
    use dlopen2::symbor::{Library, SymBorApi};
    use rs_can::{CanError, CanFrame, CanId, ChannelConfig};

    #[test]
    fn test_init_channel() -> anyhow::Result<()> {
        let dev_type = ZCanDeviceType::ZCAN_USBCANFD_200U;
        let dev_idx = 0;
        let channel = 0;
        let channels = 2;

        let so_path = "library/linux/x86_64/libusbcanfd.so";
        let lib = Library::open(so_path).expect(LOAD_LIB_FAILED);

        let api = unsafe { USBCANFDApi::load(&lib) }.expect("ZLGCAN - could not load symbols!");

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
        assert_eq!(dev_info.can_channels(), channels);
        assert!(dev_info.canfd());

        let mut context = ZChannelContext::new(context, channel);
        api.init_can_chl("library", &mut context, &cfg)?;
        let frame = CanMessage::new(
            CanId::from_bits(0x7E0, Some(false)),
            [0x01, 0x02, 0x03].as_slice(),
        )
        .ok_or(CanError::other_error("invalid data length"))?;
        let frame1 = CanMessage::new(
            CanId::from_bits(0x1888FF00, Some(true)),
            [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08].as_slice(),
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

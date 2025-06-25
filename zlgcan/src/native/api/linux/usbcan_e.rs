use std::ffi::{c_uchar, c_uint, CString};
use dlopen2::symbor::{Symbol, SymBorApi};
use rs_can::{CanError, ChannelConfig};

use crate::{
    constants,
    driver::Handler,
    native::{
        api::{ZChannelContext, ZDeviceContext, ZCanApi, ZCloudApi, ZDeviceApi, ZLinApi},
        can::{ZCanChlError, ZCanChlStatus, ZCanFrame, ZCanChlCfg, ZCanChlMode},
        constants::{channel_bitrate, channel_work_mode},
        device::{IProperty, SetValueFunc, ZDeviceInfo},
    },
};

#[allow(non_snake_case)]
#[derive(Debug, Clone, SymBorApi)]
pub(crate) struct USBCANEApi<'a> {
    /// DEVICE_HANDLE ZCAN_OpenDevice(UINT device_type, UINT device_index, UINT reserved);
    pub(crate) ZCAN_OpenDevice: Symbol<'a, unsafe extern "C" fn(dev_type: c_uint, dev_index: c_uint, reserved: c_uint) -> c_uint>,
    /// INT ZCAN_CloseDevice(DEVICE_HANDLE device_handle);
    pub(crate) ZCAN_CloseDevice: Symbol<'a, unsafe extern "C" fn(dev_hdl: c_uint) -> c_uint>,
    /// INT ZCAN_GetDeviceInf(DEVICE_HANDLE device_handle, ZCAN_DEVICE_INFO* pInfo);
    pub(crate) ZCAN_GetDeviceInf: Symbol<'a, unsafe extern "C" fn(dev_hdl: c_uint, info: *mut ZDeviceInfo) -> c_uint>,
    /// CHANNEL_HANDLE ZCAN_InitCAN(DEVICE_HANDLE device_handle, UINT can_index, ZCAN_CHANNEL_INIT_CONFIG* pInitConfig);
    pub(crate) ZCAN_InitCAN: Symbol<'a, unsafe extern "C" fn(dev_hdl: c_uint, channel: c_uint, cfg: *const ZCanChlCfg) -> c_uint>,
    /// INT ZCAN_StartCAN(CHANNEL_HANDLE channel_handle);
    pub(crate) ZCAN_StartCAN: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint) -> c_uint>,
    /// INT ZCAN_ResetCAN(CHANNEL_HANDLE channel_handle);
    pub(crate) ZCAN_ResetCAN: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint) -> c_uint>,
    /// INT ZCAN_ClearBuffer(CHANNEL_HANDLE channel_handle);
    pub(crate) ZCAN_ClearBuffer: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint) -> c_uint>,
    /// INT ZCAN_ReadChannelErrInfo(CHANNEL_HANDLE channel_handle, ZCAN_CHANNEL_ERR_INFO* pErrInfo);
    pub(crate) ZCAN_ReadChannelErrInfo: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, err: *mut ZCanChlError) -> c_uint>,
    /// INT ZCAN_ReadChannelStatus(CHANNEL_HANDLE channel_handle, ZCAN_CHANNEL_STATUS* pCANStatus);
    pub(crate) ZCAN_ReadChannelStatus: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, status: *mut ZCanChlStatus) -> c_uint>,
    /// INT ZCAN_Transmit(CHANNEL_HANDLE channel_handle, ZCAN_Transmit_Data* pTransmit, UINT len);
    pub(crate) ZCAN_Transmit: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, frames: *const ZCanFrame, len: c_uint) -> c_uint>,
    /// INT ZCAN_GetReceiveNum(CHANNEL_HANDLE channel_handle, BYTE type);
    pub(crate) ZCAN_GetReceiveNum: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, msg: c_uchar) -> c_uint>,
    /// INT ZCAN_Receive(CHANNEL_HANDLE channel_handle, ZCAN_Receive_Data* pReceive, UINT len, INT wait_time);
    pub(crate) ZCAN_Receive: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, frames: *const ZCanFrame, size: c_uint, timeout: c_uint) -> c_uint>,
    /// INT ZCAN_TransmitFD(CHANNEL_HANDLE channel_handle, ZCAN_TransmitFD_Data* pTransmit, UINT len);
    //ZCAN_TransmitFD: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, frames: *const ZCanFdFrame, len: c_uint) -> c_uint>,
    /// INT ZCAN_ReceiveFD(CHANNEL_HANDLE channel_handle, ZCAN_ReceiveFD_Data* pReceive, UINT len, INT wait_time);
    //ZCAN_ReceiveFD: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, frames: *const ZCanFdFrame, size: c_uint, timeout: c_uint) -> c_uint>,

    /// IProperty* GetIProperty(DEVICE_HANDLE device_handle);   //获取属性接口
    pub(crate) GetIProperty: Symbol<'a, unsafe extern "C" fn(dev_hdl: c_uint) -> *const IProperty>,
    /// INT ReleaseIProperty(IProperty * pIProperty);
    pub(crate) ReleaseIProperty: Symbol<'a, unsafe extern "C" fn(p: *const IProperty) -> c_uint>,
}

impl USBCANEApi<'_> {
    pub(crate) const INVALID_DEVICE_HANDLE: u32 = 0;
    pub(crate) const INVALID_CHANNEL_HANDLE: u32 = 0;
    pub(crate) const STATUS_OK: u32 = 0;
    pub(crate) fn init_can_chl_ex(
        &self,
        libpath: &str,
        dev_hdl: &mut Handler,
        channel: u8,
        cfg: &ChannelConfig,
    ) -> Result<(), CanError> {
        let p = self.self_get_property(dev_hdl.device_context())?;
        let set_value_func = p.SetValue;
        let mut error = None;

        if let Some(chl_hdl) = dev_hdl.find_can(channel) {
            self.reset_can_chl(chl_hdl).unwrap_or_else(|e| rsutil::warn!("{}", e));
            dev_hdl.remove_can(channel);
        }

        match self.start_channel(libpath, dev_hdl, channel, set_value_func, cfg) {
            Ok(context) => {
                dev_hdl.add_can(channel, context);
            },
            Err(e) => {
                error = Some(e);
            }
        }

        self.release_property(&p)?;

        match error {
            Some(e) => Err(e),
            None => Ok(()),
        }
    }
    #[inline]
    fn start_channel(
        &self,
        libpath: &str,
        dev_hdl: &mut Handler,
        channel: u8,
        set_value_func: SetValueFunc,
        cfg: &ChannelConfig
    ) -> Result<ZChannelContext, CanError> {
        let mut context = ZChannelContext::new(dev_hdl.device_context().clone(), channel);
        self.init_can_chl(libpath, &mut context, cfg)?; // ZCAN_InitCAN]
        // self.usbcan_4e_api.reset_can_chl(chl_hdl).unwrap_or_else(|e| rsutil::warn!("{}", e));
        let (chl_hdl, channel) = (context.channel_handler()?, context.channel);
        self.set_channel(channel, set_value_func, cfg)?;

        match unsafe { (self.ZCAN_StartCAN)(chl_hdl) as u32 } {
            Self::STATUS_OK => Ok(context),
            code => Err(CanError::InitializeError(format!("`ZCAN_StartCAN` ret: {}", code))),
        }
    }

    fn set_channel(
        &self,
        channel: u8,
        func: SetValueFunc,
        cfg: &ChannelConfig
    ) -> Result<(), CanError> {
        unsafe {
            let func = func.ok_or(CanError::other_error("method not supported"))?;
            let cmd_path = CString::new(channel_bitrate(channel))
                .map_err(|e| CanError::OtherError(e.to_string()))?;
            let bitrate = CString::new(cfg.bitrate().to_string())
                .map_err(|e| CanError::OtherError(e.to_string()))?;
            match func(cmd_path.as_ptr(), bitrate.as_ptr()) as u32 {
                Self::STATUS_OK => Ok(()),
                code => Err(CanError::OperationError(format!("{:?}, SetValue failed ret: {}", cmd_path, code))),
            }?;

            let cmd_path = CString::new(channel_work_mode(channel))
                .map_err(|e| CanError::OtherError(e.to_string()))?;
            let mode = CString::new(
                cfg.get_other::<ZCanChlMode>(constants::CHANNEL_MODE)?
                    .unwrap_or(ZCanChlMode::Normal) as u8
                    .to_string()
            )
                .map_err(|e| CanError::OtherError(e.to_string()))?;
            match func(cmd_path.as_ptr(), mode.as_ptr()) as u32 {
                Self::STATUS_OK => Ok(()),
                code => Err(CanError::OperationError(format!("{:?}, SetValue failed ret: {}", cmd_path, code))),
            }
        }
    }

    pub(crate) fn self_get_property(&self, context: &ZDeviceContext) -> Result<IProperty, CanError> {
        let ret = unsafe { (self.GetIProperty)(context.device_handler()?) };
        if ret.is_null() {
            Err(CanError::OperationError(format!("`GetIProperty` ret: {}", 0)))
        }
        else {
            unsafe { Ok(*ret) }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::CString;
    use dlopen2::symbor::{Library, SymBorApi};
    use crate::native::{
        constants::LOAD_LIB_FAILED,
        device::{ZCanDeviceType, ZDeviceInfo}
    };
    // use crate::api::ZDeviceApi;
    use super::USBCANEApi;

    #[test]
    fn usbcan_4e_u() -> anyhow::Result<()> {
        let dev_type = ZCanDeviceType::ZCAN_USBCAN_4E_U;
        let dev_idx = 0;
        let so_path = "library/linux/x86_64/libusbcan-4e.so";

        let lib = Library::open(so_path).expect(LOAD_LIB_FAILED);
        let mut handlers = Vec::new();

        unsafe {
            let api = USBCANEApi::load(&lib).expect("ZLGCAN - could not load symbols!");
            let dev_hdl = (api.ZCAN_OpenDevice)(dev_type as u32, dev_idx, 0);
            if dev_hdl == USBCANEApi::INVALID_DEVICE_HANDLE {
                println!("Can't open the device!");
                return Ok(());
            }
            let mut dev_info = ZDeviceInfo::default();
            let ret = (api.ZCAN_GetDeviceInf)(dev_hdl, &mut dev_info);
            if ret != USBCANEApi::STATUS_OK {
                println!("Can't get the device info!");
                return Ok(());
            }

            let p = (api.GetIProperty)(dev_hdl);
            if p.is_null() {
                println!("Get property failed!");
                return Ok(());
            }
            let func = (*p).SetValue.expect("Can't get SetValue function!");

            for chl in 0..4 {
                let chl_hdl = (api.ZCAN_InitCAN)(dev_hdl, chl, std::ptr::null());
                if chl_hdl == USBCANEApi::INVALID_CHANNEL_HANDLE {
                    println!("Init channel: {} failed!", chl);
                    break;
                }
                handlers.push(chl_hdl);
                let ret = (api.ZCAN_ResetCAN)(chl_hdl);
                if ret != USBCANEApi::STATUS_OK {
                    println!("Reset channel: {} failed!", chl);
                }

                let path = CString::new(format!("info/channel/channel_{}/baud_rate", chl))?;
                let bitrate = CString::new(500_000.to_string())?;

                // let func = (*p).SetValue.expect("Can't get SetValue function!");
                let ret = func(path.as_ptr(), bitrate.as_ptr());

                if ret as u32 != USBCANEApi::STATUS_OK {
                    println!("SetValue failed: {}!", ret);
                    break;
                }

                let ret = (api.ZCAN_StartCAN)(chl_hdl);
                if ret as u32 != USBCANEApi::STATUS_OK {
                    println!("ZCAN_StartCAN failed!");
                    break;
                }
            }

            let ret = (api.ReleaseIProperty)(p);
            if ret as u32 != USBCANEApi::STATUS_OK {
                println!("ReleaseIProperty failed!");
            }

            for handler in handlers {
                let ret = (api.ZCAN_ResetCAN)(handler);
                if ret as u32 != USBCANEApi::STATUS_OK {
                    println!("ZCAN_ResetCAN failed!");
                }
            }

            let ret = (api.ZCAN_CloseDevice)(dev_hdl);
            if ret as u32 != USBCANEApi::STATUS_OK {
                println!("ZCAN_CloseDevice failed!");
            }
        }

        Ok(())
    }
}

use crate::native::{
    api::{USBCANFD800UApi, ZChannelContext, ZDeviceApi, ZDeviceContext},
    device::{CmdPath, IProperty, ZDeviceInfo},
    util::c_str_to_string,
};
use rs_can::CanError;
use std::ffi::{c_char, CString};

impl ZDeviceApi for USBCANFD800UApi<'_> {
    fn open(&self, context: &mut ZDeviceContext) -> Result<(), CanError> {
        match unsafe { (self.ZCAN_OpenDevice)(context.dev_type as u32, context.dev_idx, 0) } {
            Self::INVALID_DEVICE_HANDLE => Err(CanError::InitializeError(format!(
                "`ZCAN_OpenDevice` ret: {}",
                Self::INVALID_DEVICE_HANDLE
            ))),
            v => {
                context.dev_hdl = Some(v);
                Ok(())
            }
        }
    }

    fn close(&self, context: &ZDeviceContext) -> Result<(), CanError> {
        match unsafe { (self.ZCAN_CloseDevice)(context.device_handler()?) } {
            Self::STATUS_OK => Ok(()),
            code => Err(CanError::OperationError(format!(
                "`ZCAN_CloseDevice` ret: {}",
                code
            ))),
        }
    }

    fn read_device_info(&self, context: &ZDeviceContext) -> Result<ZDeviceInfo, CanError> {
        let mut info = ZDeviceInfo::default();
        match unsafe { (self.ZCAN_GetDeviceInf)(context.device_handler()?, &mut info) } {
            Self::STATUS_OK => Ok(info),
            code => Err(CanError::OperationError(format!(
                "`ZCAN_GetDeviceInf` ret: {}",
                code
            ))),
        }
    }

    fn get_property(&self, context: &ZChannelContext) -> Result<IProperty, CanError> {
        let ret = unsafe { (self.GetIProperty)(context.channel_handler()?) };
        if ret.is_null() {
            Err(CanError::OperationError(format!(
                "`GetIProperty` ret: {}",
                0
            )))
        } else {
            unsafe { Ok(*ret) }
        }
    }

    fn release_property(&self, p: &IProperty) -> Result<(), CanError> {
        match unsafe { (self.ReleaseIProperty)(p) } {
            Self::STATUS_OK => Ok(()),
            code => Err(CanError::OperationError(format!(
                "`ReleaseIProperty` ret: {}",
                code
            ))),
        }
    }

    fn set_values(
        &self,
        context: &ZChannelContext,
        values: Vec<(CmdPath, *const c_char)>,
    ) -> Result<(), CanError> {
        unsafe {
            let p = self.get_property(context)?;
            match p.SetValue {
                Some(f) => {
                    for (cmd, value) in values {
                        let path = cmd.get_path();
                        // let _path = format!("{}/{}", path, channel);
                        let _path =
                            CString::new(path).map_err(|e| CanError::OtherError(e.to_string()))?;
                        match f(_path.as_ptr(), value) {
                            1 => (),
                            _ => rsutil::warn!("ZLGCAN - set `{}` failed!", path),
                        }
                    }

                    let _ = self.release_property(&p).is_err_and(|e| -> bool {
                        rsutil::warn!("{}", e);
                        true
                    });
                    Ok(())
                }
                None => Err(CanError::NotSupportedError),
            }
        }
    }

    fn get_values(
        &self,
        context: &ZChannelContext,
        paths: Vec<CmdPath>,
    ) -> Result<Vec<String>, CanError> {
        unsafe {
            let p = self.get_property(context)?;
            match p.GetValue {
                Some(f) => {
                    let mut result = Vec::new();
                    for cmd in paths {
                        let path = cmd.get_path();
                        let _path = CString::new(format!("{}/{}", path, context.channel))
                            .map_err(|e| CanError::OtherError(e.to_string()))?;
                        let ret = f(_path.as_ptr());
                        let v = c_str_to_string(ret)?;
                        result.push(v);
                    }

                    let _ = self.release_property(&p).is_err_and(|e| -> bool {
                        rsutil::warn!("{}", e);
                        true
                    });

                    Ok(result)
                }
                None => Err(CanError::NotSupportedError),
            }
        }
    }
}

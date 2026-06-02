use crate::native::{
    api::{WinApi, ZChannelContext, ZDeviceApi, ZDeviceContext},
    constants::{STATUS_OFFLINE, STATUS_ONLINE},
    device::{CmdPath, IProperty, ZDeviceInfo},
    util::c_str_to_string,
};
use rs_can::{CanError, CanResult};
use std::ffi::{c_char, c_void, CString};

impl ZDeviceApi for WinApi<'_> {
    fn open(&self, context: &mut ZDeviceContext) -> CanResult<()> {
        match unsafe { (self.ZCAN_OpenDevice)(context.dev_type as u32, context.dev_idx, 0) } {
            Self::INVALID_DEVICE_HANDLE => Err(CanError::OperationError(format!(
                "`ZCAN_OpenDevice` ret = {}",
                Self::INVALID_DEVICE_HANDLE
            ))),
            v => {
                context.dev_hdl = Some(v);
                Ok(())
            }
        }
    }
    fn close(&self, context: &ZDeviceContext) -> CanResult<()> {
        match unsafe { (self.ZCAN_CloseDevice)(context.device_handler()?) } {
            Self::STATUS_OK => Ok(()),
            code => Err(CanError::OperationError(format!(
                "`ZCAN_CloseDevice` ret = {}",
                code
            ))),
        }
    }
    fn read_device_info(&self, context: &ZDeviceContext) -> CanResult<ZDeviceInfo> {
        let mut info = ZDeviceInfo::default();
        match unsafe { (self.ZCAN_GetDeviceInf)(context.device_handler()?, &mut info) } {
            Self::STATUS_OK => Ok(info),
            code => Err(CanError::OperationError(format!(
                "`ZCAN_GetDeviceInf` ret = {}",
                code
            ))),
        }
    }
    fn is_online(&self, context: &ZDeviceContext) -> CanResult<bool> {
        unsafe {
            match (self.ZCAN_IsDeviceOnLine)(context.device_handler()?) {
                STATUS_ONLINE => Ok(true),
                STATUS_OFFLINE => Ok(false),
                code => Err(CanError::OperationError(format!(
                    "`ZCAN_IsDeviceOnLine` ret = {}",
                    code
                ))),
            }
        }
    }
    fn get_property(&self, context: &ZChannelContext) -> CanResult<IProperty> {
        unsafe {
            let ret = (self.GetIProperty)(context.device_handler()?);
            if ret.is_null() {
                Err(CanError::OperationError(format!(
                    "`GetIProperty` ret = {}",
                    0
                )))
            } else {
                Ok(*ret)
            }
        }
    }
    fn release_property(&self, p: &IProperty) -> CanResult<()> {
        unsafe {
            match (self.ReleaseIProperty)(p) {
                Self::STATUS_OK => Ok(()),
                code => Err(CanError::OperationError(format!(
                    "`ReleaseIProperty` ret = {}",
                    code
                ))),
            }
        }
    }
    fn set_value(
        &self,
        context: &ZChannelContext,
        cmd_path: &CmdPath,
        value: *const c_void,
    ) -> CanResult<()> {
        unsafe {
            let path = cmd_path.get_path();
            let _path = CString::new(path).map_err(|e| CanError::OtherError(e.to_string()))?;
            match (self.ZCAN_SetValue)(
                context.device_handler()?,
                _path.as_ptr() as *const c_char,
                value,
            ) {
                Self::STATUS_OK => Ok(()),
                code => Err(CanError::OperationError(format!(
                    "`ZCAN_SetValue` ret = {}",
                    code
                ))),
            }
        }
    }
    fn get_value(&self, context: &ZChannelContext, cmd_path: &CmdPath) -> CanResult<*const c_void> {
        unsafe {
            let path = cmd_path.get_path();
            let path = CString::new(path).map_err(|e| CanError::OtherError(e.to_string()))?;
            if context.device.dev_type.get_value_support() {
                let ret =
                    (self.ZCAN_GetValue)(context.device_handler()?, path.as_ptr() as *const c_char);
                if ret.is_null() {
                    Err(CanError::OperationError(format!(
                        "`ZCAN_GetValue` ret = {}",
                        0
                    )))
                } else {
                    Ok(ret)
                }
            } else {
                Err(CanError::NotSupportedError)
            }
        }
    }
    fn set_values(
        &self,
        context: &ZChannelContext,
        values: Vec<(CmdPath, *const c_char)>,
    ) -> CanResult<()> {
        unsafe {
            let p = self.get_property(context)?;
            match p.SetValue {
                Some(f) => {
                    for (cmd, value) in values {
                        let path = cmd.get_path();
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
    fn get_values(&self, context: &ZChannelContext, paths: Vec<CmdPath>) -> CanResult<Vec<String>> {
        unsafe {
            let p = self.get_property(context)?;
            let channel = context.channel;
            match p.GetValue {
                Some(f) => {
                    let mut result = Vec::new();
                    for cmd in paths {
                        let path = cmd.get_path();
                        let _path = CString::new(format!("{}/{}", path, channel))
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

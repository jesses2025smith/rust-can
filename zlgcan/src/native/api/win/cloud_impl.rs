use crate::native::{
    api::{WinApi, ZCloudApi, ZDeviceContext},
    cloud::{ZCloudGpsFrame, ZCloudServerInfo, ZCloudUserData},
};
use rs_can::CanError;
use std::ffi::CString;

impl ZCloudApi for WinApi<'_> {
    fn set_server(&self, server: ZCloudServerInfo) -> Result<(), CanError> {
        unsafe {
            (self.ZCLOUD_SetServerInfo)(
                server.http_url,
                server.http_port,
                server.mqtt_url,
                server.mqtt_port,
            )
        }

        Ok(())
    }
    fn connect_server(&self, username: &str, password: &str) -> Result<(), CanError> {
        let username = CString::new(username).map_err(|e| CanError::OtherError(e.to_string()))?;
        let password = CString::new(password).map_err(|e| CanError::OtherError(e.to_string()))?;
        match unsafe { (self.ZCLOUD_ConnectServer)(username.as_ptr(), password.as_ptr()) } {
            Self::STATUS_OK => Ok(()),
            code => Err(CanError::OperationError(format!(
                "`ZCLOUD_ConnectServer` ret = {}",
                code
            ))),
        }
    }
    fn is_connected_server(&self) -> Result<bool, CanError> {
        unsafe { Ok((self.ZCLOUD_IsConnected)()) }
    }
    fn disconnect_server(&self) -> Result<(), CanError> {
        match unsafe { (self.ZCLOUD_DisconnectServer)() } {
            0 => Ok(()),
            code => Err(CanError::OperationError(format!(
                "`ZCLOUD_DisconnectServer` ret = {}",
                code
            ))),
        }
    }
    fn get_userdata(&self, update: i32) -> Result<ZCloudUserData, CanError> {
        unsafe {
            let data = (self.ZCLOUD_GetUserData)(update);
            if data.is_null() {
                Err(CanError::OperationError(format!(
                    "`ZCLOUD_GetUserData` ret = {}",
                    0
                )))
            } else {
                Ok(*data)
            }
        }
    }
    fn receive_gps(
        &self,
        context: &ZDeviceContext,
        size: u32,
        timeout: u32,
    ) -> Result<Vec<ZCloudGpsFrame>, CanError> {
        let mut frames = Vec::new();
        frames.resize_with(size as usize, Default::default);

        let ret = unsafe {
            (self.ZCLOUD_ReceiveGPS)(
                context.device_handler()?,
                frames.as_mut_ptr(),
                size,
                timeout,
            )
        };
        if ret < size {
            rsutil::warn!(
                "ZLGCAN - receive GPS frame expect: {}, actual: {}!",
                size,
                ret
            );
        } else if ret > 0 {
            rsutil::trace!("ZLGCAN - receive GPS frame: {}", ret);
        }
        Ok(frames)
    }
}

use crate::native::{
    api::{USBCANEApi, ZChannelContext, ZDeviceApi, ZDeviceContext},
    device::{IProperty, ZDeviceInfo},
};
use rs_can::CanError;

impl ZDeviceApi for USBCANEApi<'_> {
    fn open(&self, context: &mut ZDeviceContext) -> Result<(), CanError> {
        let (dev_type, dev_idx) = (context.dev_type, context.dev_idx);
        match unsafe { (self.ZCAN_OpenDevice)(dev_type as u32, dev_idx, 0) } as u32 {
            Self::INVALID_DEVICE_HANDLE => Err(CanError::InitializeError(format!(
                "`ZCAN_OpenDevice` ret: {}",
                Self::INVALID_DEVICE_HANDLE
            ))),
            handler => {
                context.dev_hdl = Some(handler);
                Ok(())
            }
        }
    }

    fn close(&self, context: &ZDeviceContext) -> Result<(), CanError> {
        match unsafe { (self.ZCAN_CloseDevice)(context.device_handler()?) } as u32 {
            Self::STATUS_OK => Ok(()),
            code => Err(CanError::OperationError(format!(
                "ZCAN_CloseDevice ret: {}",
                code
            ))),
        }
    }

    fn read_device_info(&self, context: &ZDeviceContext) -> Result<ZDeviceInfo, CanError> {
        let mut info = ZDeviceInfo::default();
        match unsafe { (self.ZCAN_GetDeviceInf)(context.device_handler()?, &mut info) } as u32 {
            Self::STATUS_OK => Ok(info),
            code => Err(CanError::OperationError(format!(
                "ZCAN_GetDeviceInf ret: {}",
                code
            ))),
        }
    }

    fn get_property(&self, context: &ZChannelContext) -> Result<IProperty, CanError> {
        self.self_get_property(&context.device)
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
}

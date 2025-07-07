use crate::native::{
    api::{USBCANApi, ZChannelContext, ZDeviceApi, ZDeviceContext},
    device::{CmdPath, ZDeviceInfo},
};
use rs_can::CanError;
use std::ffi::c_void;

impl ZDeviceApi for USBCANApi<'_> {
    fn open(&self, context: &mut ZDeviceContext) -> Result<(), CanError> {
        let (dev_type, dev_idx) = (context.dev_type, context.dev_idx);
        match unsafe { (self.VCI_OpenDevice)(dev_type as u32, dev_idx, 0) } {
            Self::STATUS_OK => Ok(()),
            code => Err(CanError::InitializeError(format!(
                "`VCI_OpenDevice` ret: {}",
                code
            ))),
        }
    }

    fn close(&self, context: &ZDeviceContext) -> Result<(), CanError> {
        let (dev_type, dev_idx) = (context.dev_type, context.dev_idx);
        match unsafe { (self.VCI_CloseDevice)(dev_type as u32, dev_idx) } {
            Self::STATUS_OK => Ok(()),
            code => Err(CanError::OperationError(format!(
                "`VCI_CloseDevice` ret: {}",
                code
            ))),
        }
    }

    fn read_device_info(&self, context: &ZDeviceContext) -> Result<ZDeviceInfo, CanError> {
        let mut info = ZDeviceInfo::default();
        let (dev_type, dev_idx) = (context.dev_type, context.dev_idx);
        match unsafe { (self.VCI_ReadBoardInfo)(dev_type as u32, dev_idx, &mut info) } {
            Self::STATUS_OK => Ok(info),
            code => Err(CanError::OperationError(format!(
                "`VCI_ReadBoardInfo` ret: {}",
                code
            ))),
        }
    }

    fn set_reference(
        &self,
        context: &ZChannelContext,
        cmd_path: &CmdPath,
        value: *const c_void,
    ) -> Result<(), CanError> {
        let (dev_type, dev_idx, channel) = (
            context.device.dev_type,
            context.device.dev_idx,
            context.channel,
        );
        let cmd = cmd_path.get_reference();
        match unsafe {
            (self.VCI_SetReference)(dev_type as u32, dev_idx, channel as u32, cmd, value)
        } {
            Self::STATUS_OK => Ok(()),
            code => Err(CanError::OperationError(format!(
                "`VCI_SetReference` ret: {}",
                code
            ))),
        }
    }

    fn get_reference(
        &self,
        context: &ZChannelContext,
        cmd_path: &CmdPath,
        value: *mut c_void,
    ) -> Result<(), CanError> {
        let (dev_type, dev_idx, channel) = (
            context.device.dev_type,
            context.device.dev_idx,
            context.channel,
        );
        let cmd = cmd_path.get_reference();
        match unsafe {
            (self.VCI_GetReference)(dev_type as u32, dev_idx, channel as u32, cmd, value)
        } {
            Self::STATUS_OK => Ok(()),
            code => Err(CanError::OperationError(format!(
                "`VCI_GetReference` ret: {}",
                code
            ))),
        }
    }
}

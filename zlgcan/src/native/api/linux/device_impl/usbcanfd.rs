use crate::native::{
    api::{USBCANFDApi, ZChannelContext, ZDeviceApi, ZDeviceContext},
    device::{CmdPath, ZDeviceInfo},
};
use rs_can::{CanError, CanResult};
use std::ffi::c_void;

impl ZDeviceApi for USBCANFDApi<'_> {
    fn open(&self, context: &mut ZDeviceContext) -> CanResult<()> {
        let (dev_type, dev_idx) = (context.dev_type, context.dev_idx);
        match unsafe { (self.VCI_OpenDevice)(dev_type as u32, dev_idx, 0) } {
            Self::STATUS_OK => Ok(()),
            code => Err(CanError::InitializeError(format!(
                "`VCI_OpenDevice` ret: {}",
                code
            ))),
        }
    }

    fn close(&self, context: &ZDeviceContext) -> CanResult<()> {
        let (dev_type, dev_idx) = (context.dev_type, context.dev_idx);
        match unsafe { (self.VCI_CloseDevice)(dev_type as u32, dev_idx) } {
            Self::STATUS_OK => Ok(()),
            code => Err(CanError::OperationError(format!(
                "`VCI_CloseDevice` ret: {}",
                code
            ))),
        }
    }

    fn read_device_info(&self, context: &ZDeviceContext) -> CanResult<ZDeviceInfo> {
        let (dev_type, dev_idx) = (context.dev_type, context.dev_idx);
        let mut info = ZDeviceInfo::default();
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
    ) -> CanResult<()> {
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
    ) -> CanResult<()> {
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

    fn set_value(
        &self,
        context: &ZChannelContext,
        cmd_path: &CmdPath,
        value: *const c_void,
    ) -> CanResult<()> {
        self.set_reference(context, cmd_path, value)
    }

    fn get_value(&self, context: &ZChannelContext, cmd_path: &CmdPath) -> CanResult<*const c_void> {
        if context.device.dev_type.get_value_support() {
            let mut ret: Vec<u8> = Vec::new();
            ret.resize(16, 0);
            self.get_reference(context, cmd_path, ret.as_mut_ptr() as *mut c_void)?;
            Ok(ret.as_ptr() as *const c_void)
        } else {
            Err(CanError::NotSupportedError)
        }
    }

    fn debug(&self, level: u32) -> CanResult<()> {
        unsafe {
            match (self.VCI_Debug)(level) {
                Self::STATUS_OK => Ok(()),
                code => Err(CanError::OperationError(format!(
                    "`VCI_Debug` ret: {}",
                    code
                ))),
            }
        }
    }
}

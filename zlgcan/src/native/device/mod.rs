include!("property.rs");
include!("typedef.rs");

use crate::native::constants::CANFD_STR;
use rs_can::CanError;
use std::{
    ffi::{c_uchar, c_ushort, CString},
    fmt::{Display, Formatter},
};

const SN_LENGTH: usize = 20;
const ID_LENGTH: usize = 40;
const PAD_LENGTH: usize = 4;

/// The information about derive device.
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct DeriveInfo {
    pub canfd: bool,
    pub channels: u8,
    // pub resistance: bool,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ZDeviceInfo {
    pub(crate) hwv: c_ushort,            //**< hardware version */
    pub(crate) fwv: c_ushort,            //**< firmware version */
    pub(crate) drv: c_ushort,            //**< driver version */
    pub(crate) api: c_ushort,            //**< API version */
    pub(crate) irq: c_ushort,            //**< IRQ */
    pub(crate) chn: c_uchar,             //**< channels */
    pub(crate) sn: [c_uchar; SN_LENGTH], //**< serial number */
    pub(crate) id: [c_uchar; ID_LENGTH], //**< card id */
    #[allow(dead_code)]
    pub(crate) pad: [c_ushort; PAD_LENGTH],
}

impl Default for ZDeviceInfo {
    #[inline(always)]
    fn default() -> Self {
        Self {
            hwv: Default::default(),
            fwv: Default::default(),
            drv: Default::default(),
            api: Default::default(),
            irq: Default::default(),
            chn: Default::default(),
            sn: Default::default(),
            id: [Default::default(); ID_LENGTH],
            pad: Default::default(),
        }
    }
}

impl TryFrom<&DeriveInfo> for ZDeviceInfo {
    type Error = CanError;
    fn try_from(value: &DeriveInfo) -> Result<Self, Self::Error> {
        let device = if value.canfd {
            "Derive USBCANFD device"
        } else {
            "Derive USBCAN device"
        };
        let mut id = CString::new(device)
            .as_ref()
            .map_err(|e| CanError::OtherError(e.to_string()))?
            .as_bytes()
            .to_owned();
        id.resize(ID_LENGTH, 0);
        Ok(Self {
            chn: value.channels,
            id: id
                .try_into()
                .map_err(|v| CanError::OtherError(format!("{:?}", v)))?,
            ..Default::default()
        })
    }
}

impl ZDeviceInfo {
    #[inline(always)]
    fn version(ver: u16) -> String {
        let major = ((ver & 0xFF00) >> 8) as u8;
        let minor = (ver & 0xFF) as u8;
        let h_major = (major & 0xF0) >> 4;
        if h_major > 0 {
            format!(
                "V{:1}{:1}.{:1}{:1}",
                h_major,
                major & 0x0F,
                (minor & 0xF0) >> 4,
                minor & 0x0F
            )
        } else {
            format!(
                "V{:1}.{:1}{:1}",
                major & 0x0F,
                (minor & 0xF0) >> 4,
                minor & 0x0F
            )
        }
    }
    #[inline(always)]
    pub fn hardware_version(&self) -> String {
        Self::version(self.hwv)
    }

    #[inline(always)]
    pub fn firmware_version(&self) -> String {
        Self::version(self.fwv)
    }

    #[inline(always)]
    pub fn driver_version(&self) -> String {
        Self::version(self.drv)
    }

    #[inline(always)]
    pub fn api_version(&self) -> String {
        Self::version(self.api)
    }

    #[inline(always)]
    pub fn can_channels(&self) -> u8 {
        self.chn
    }

    // #[inline(always)]
    // pub fn lin_channels(&self) -> u8 {
    //     0   // TODO parse lin channel
    // }

    #[inline(always)]
    pub fn irq(&self) -> u16 {
        self.irq
    }

    #[inline(always)]
    pub fn sn(&self) -> String {
        String::from_iter(self.sn.iter().take_while(|c| **c != 0).map(|c| *c as char))
    }

    #[inline(always)]
    pub fn id(&self) -> String {
        String::from_iter(self.id.iter().take_while(|c| **c != 0).map(|c| *c as char))
    }

    #[inline(always)]
    pub fn canfd(&self) -> bool {
        self.id().contains(CANFD_STR)
    }
}

impl Display for ZDeviceInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Device Info")
            .field("\n   Serial Number", &self.sn())
            .field("\n              ID", &self.id())
            .field("\n    CAN channels", &self.can_channels())
            .field("\n CANFD supported", &self.canfd())
            .field("\n            IRQs", &self.irq())
            .field("\nHardware Version", &self.hardware_version())
            .field("\nFirmware Version", &self.firmware_version())
            .field("\n  Driver Version", &self.driver_version())
            .field("\n     Api Version", &self.api_version())
            .finish()
    }
}

/// use for batch setting parameters for device.
/// path used on windows and linux USBCANFD-4E|8E and USBCANFD-800U
/// reference only used on Linux USBCAN USBCANFD
#[allow(unused)]
pub(crate) union CmdPath<'a> {
    path: &'a str,
    reference: u32,
}

#[allow(unused)]
impl<'a> CmdPath<'a> {
    #[inline(always)]
    pub fn new_path(path: &'a str) -> Self {
        Self { path }
    }

    #[inline(always)]
    pub fn new_reference(value: u32) -> Self {
        Self { reference: value }
    }

    #[inline(always)]
    pub fn get_path(&self) -> &str {
        unsafe { self.path }
    }

    #[inline(always)]
    pub fn get_reference(&self) -> u32 {
        unsafe { self.reference }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn device_info_new() -> anyhow::Result<()> {
        let derive = DeriveInfo {
            canfd: false,
            channels: 2,
        };
        let device_info = ZDeviceInfo::try_from(&derive)?;
        assert_eq!(device_info.chn, 2);
        assert_eq!(device_info.id(), "Derive USBCAN device");

        let derive = DeriveInfo {
            canfd: true,
            channels: 2,
        };
        let device_info = ZDeviceInfo::try_from(&derive)?;
        assert_eq!(device_info.chn, 2);
        assert_eq!(device_info.id(), "Derive USBCANFD device");

        Ok(())
    }

    #[test]
    fn device_version() {
        let dev_info = ZDeviceInfo {
            hwv: 0x0001,
            fwv: 0x0101,
            drv: 0x0A01,
            api: 0x0237,
            irq: 8,
            chn: 3,
            sn: [0; SN_LENGTH],
            id: [0; ID_LENGTH],
            pad: [0; PAD_LENGTH],
        };
        assert_eq!(dev_info.hardware_version(), "V0.01");
        assert_eq!(dev_info.firmware_version(), "V1.01");
        assert_eq!(dev_info.driver_version(), "V10.01");
        assert_eq!(dev_info.api_version(), "V2.37");
    }
}

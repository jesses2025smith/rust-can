mod utils;

use zlgcan_rs::device::{DeriveInfo, ZCanDeviceType};
use self::utils::{can_device1, can_device2, device_open};

#[test]
fn usbcan_official1() -> anyhow::Result<()> {
    let dev_type = ZCanDeviceType::ZCAN_USBCAN1;
    let dev_idx = 0;
    let channels = 1;
    let available = 1;
    let mut driver = device_open(dev_type, dev_idx, None, channels, available, false)?;
    can_device1(&mut driver)?;
    Ok(())
}

#[test]
fn usbcan_derive1() -> anyhow::Result<()> {
    let dev_type = ZCanDeviceType::ZCAN_USBCAN1;
    let dev_idx = 0;
    let channels = 1;
    let available = 1;
    let canfd = false;

    let derive_info = DeriveInfo { canfd, channels };
    let mut driver = device_open(dev_type, dev_idx, Some(derive_info), channels, available, canfd)?;
    can_device1(&mut driver)?;
    Ok(())
}

#[test]
fn usbcan_official2() {
    // TODO has no this device
}

#[test]
fn usbcan_derive2() -> anyhow::Result<()> {
    let dev_type = ZCanDeviceType::ZCAN_USBCAN2;
    let dev_idx = 0;
    let channels = 2;
    let available = 2;
    let canfd = false;

    let derive_info = DeriveInfo { canfd, channels };
    let mut driver = device_open(dev_type, dev_idx, Some(derive_info), channels, available, canfd)?;
    can_device2(&mut driver, 0, 1)?;
    Ok(())
}


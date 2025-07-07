mod utils;

use self::utils::{can_device2, device_open};
use zlgcan_rs::device::ZCanDeviceType;

#[test]
fn usbcan_4eu() -> anyhow::Result<()> {
    let dev_type = ZCanDeviceType::ZCAN_USBCAN_4E_U;
    let dev_idx = 0;
    let channels = 4;
    let available = 4;
    let canfd = false;

    let mut driver = device_open(dev_type, dev_idx, None, channels, available, canfd)?;
    can_device2(&mut driver, 0, 1)?;
    Ok(())
}

#[test]
fn usbcan_8eu() -> anyhow::Result<()> {
    let dev_type = ZCanDeviceType::ZCAN_USBCAN_8E_U;
    let dev_idx = 0;
    let channels = 8;
    let available = 8;
    let canfd = false;

    let mut driver = device_open(dev_type, dev_idx, None, channels, available, canfd)?;
    can_device2(&mut driver, 0, 1)?;
    Ok(())
}

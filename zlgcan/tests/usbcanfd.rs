mod utils;

use zlgcan_rs::device::ZCanDeviceType;
use self::utils::{canfd_device2, device_open};

#[test]
fn usbcanfd_200u() -> anyhow::Result<()> {
    let dev_type = ZCanDeviceType::ZCAN_USBCANFD_200U;
    let dev_idx = 0;
    let channels = 2;
    let available = 2;
    let mut driver = device_open(dev_type, dev_idx, None, channels, available, true)?;
    canfd_device2(&mut driver, available,0, 1)?;
    Ok(())
}

/// `Attention:`
/// The USBCANFD-400U only supported channel0 and channel1 on Linux
#[test]
fn usbcanfd_400u() -> anyhow::Result<()> {
    // TODO USBCANFD-400U channel 3-4 is not supported
    let dev_type = ZCanDeviceType::ZCAN_USBCANFD_200U;
    let dev_idx = 0;
    let channels = 4;
    let available = 2;
    let mut driver = device_open(dev_type, dev_idx, None, channels, available, true)?;
    canfd_device2(&mut driver, available, 0, 1)?;
    Ok(())
}

mod utils;

use self::utils::{canfd_device2, device_open};
use zlgcan_rs::device::ZCanDeviceType;

#[test]
fn usbcanfd_800u() -> anyhow::Result<()> {
    let dev_type = ZCanDeviceType::ZCAN_USBCANFD_800U;
    let dev_idx = 0;
    let channels = 8;
    let available = 8;
    let canfd = true;

    let mut driver = device_open(dev_type, dev_idx, None, channels, available, canfd)?;
    canfd_device2(&mut driver, available, 0, 1)?;
    Ok(())
}

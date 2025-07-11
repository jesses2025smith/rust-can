use rs_can::{CanError, ChannelConfig, DeviceBuilder};
use zlgcan_rs::{
    can::{ZCanChlMode, ZCanChlType},
    device::ZCanDeviceType,
    driver::ZDriver,
    CHANNEL_MODE, CHANNEL_TYPE, DEVICE_INDEX, DEVICE_TYPE,
};

pub const CHANNEL: u8 = 0;

pub fn init_device() -> Result<ZDriver, CanError> {
    let dev_type = ZCanDeviceType::ZCAN_USBCANFD_200U;

    let mut builder = DeviceBuilder::new();

    let mut ch1_cfg = ChannelConfig::new(500_000);
    ch1_cfg
        .add_other(CHANNEL_MODE, Box::new(ZCanChlMode::Normal))
        .add_other(CHANNEL_TYPE, Box::new(ZCanChlType::CAN));

    let mut ch2_cfg = ChannelConfig::new(500_000);
    ch2_cfg
        .add_other(CHANNEL_MODE, Box::new(ZCanChlMode::Normal))
        .add_other(CHANNEL_TYPE, Box::new(ZCanChlType::CAN));

    builder
        .add_other(DEVICE_TYPE, Box::new(dev_type))
        .add_other(DEVICE_INDEX, Box::new(0u32))
        .add_config(0, ch1_cfg)
        .add_config(1, ch2_cfg);

    let device = builder.build::<ZDriver>()?;

    Ok(device)
}

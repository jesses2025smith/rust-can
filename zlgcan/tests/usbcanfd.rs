mod utils;

use std::{thread, time::{Duration, SystemTime}};

use anyhow::Ok;
use rs_can::ChannelConfig;
use zlgcan_rs::{can::{ZCanChlMode, ZCanChlType, ZCanFrameType}, device::ZCanDeviceType, driver::{ZDriver, ZCan}, CHANNEL_MODE, CHANNEL_TYPE};
use self::utils::{canfd_device2, device_open};

#[allow(unused)]
fn only_recv(driver: &mut ZDriver, available: u8, recv_ch: u8) -> anyhow::Result<()> {
    for i in 0..available {
        let mut cfg = ChannelConfig::new(500_000);
        cfg.set_data_bitrate(1_000_000)
            .add_other(CHANNEL_TYPE, Box::new(ZCanChlType::CANFD_ISO))
            .add_other(CHANNEL_MODE, Box::new(ZCanChlMode::Normal));
        driver.init_can_chl(i, &cfg)?;
    }

    let timeout = Duration::from_secs(20);
    let start_time = SystemTime::now();
    loop {
        let cnt = driver.get_can_num(recv_ch, ZCanFrameType::CAN)?;
        let cnt_fd = driver.get_can_num(recv_ch, ZCanFrameType::CANFD)?;

        if cnt > 0 {
            let frames = driver.receive_can(recv_ch, cnt, None)?;
            assert_eq!(frames.len() as u32, cnt);
            println!("received frame: {cnt}");
            frames.iter().for_each(|f| println!("{}", f));
        }

        if cnt_fd > 0 {
            // receive CANFD frames
            let frames = driver.receive_canfd(recv_ch, cnt_fd, None)?;
            assert_eq!(frames.len() as u32, cnt_fd);
            println!("received fd frame: {cnt_fd}");
            frames.iter().for_each(|f| println!("{}", f));
        }

        let elapsed_time = SystemTime::now().duration_since(start_time)?;
        if elapsed_time > timeout {
            break;
        }
        thread::sleep(Duration::from_millis(100));
    }

    Ok(())
}

#[test]
fn usbcanfd_200u() -> anyhow::Result<()> {
    let dev_type = ZCanDeviceType::ZCAN_USBCANFD_200U;
    let dev_idx = 0;
    let channels = 2;
    let available = 2;
    let mut driver = device_open(dev_type, dev_idx, None, channels, available, true)?;
    // only_recv(&mut driver, available, 0)?;
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

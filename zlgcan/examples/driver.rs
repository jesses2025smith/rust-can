mod utils;

use crate::utils::{init_device, CHANNEL};
use rs_can::{CanDevice, CanFrame, CanId};
use zlgcan_rs::can::CanMessage;

fn main() -> anyhow::Result<()> {
    let device = init_device()?;

    let data = vec![0x02, 0x10, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00];
    let mut msg = CanMessage::new(CanId::from(0x7DF), &data).unwrap();
    msg.set_channel(CHANNEL);

    device.transmit(msg, None)?;

    let results = device.receive(CHANNEL, None)?;
    println!("{:?}", results);

    Ok(())
}

mod utils;

use crate::utils::{init_device, CHANNEL};
use rs_can::{CanDevice, CanFrame, CanId};
use zlgcan_rs::can::ZCanFrame;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let device = init_device()?;

    let data = vec![0x02, 0x10, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00];
    let id = CanId::try_from(0x7DF).expect("valid standard id");
    let mut msg = ZCanFrame::new_can(id, &data)?;
    msg.set_channel(CHANNEL);

    device.transmit(msg, None).await?;

    let results = device.receive(CHANNEL, None).await?;
    println!("{:?}", results);

    Ok(())
}

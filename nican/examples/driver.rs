use std::time::Duration;
use nican_rs::{CanMessage, NiCan};
use rs_can::{CanDevice, CanFrame, CanId, ChannelConfig, DeviceBuilder};

fn main() -> anyhow::Result<()> {
    let channel = "CAN0";
    let mut builder = DeviceBuilder::new();
    builder.add_config(channel, ChannelConfig::new(500_000));
    let mut device = builder.build::<NiCan>()?;

    let data = vec![0x02, 0x10, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00];
    let mut count = 0;
    loop {
        let mut msg = CanMessage::new(CanId::from(0x7DF), data.as_slice()).unwrap();
        msg.set_channel(channel.into());
        device.transmit(msg, None)?;

        std::thread::sleep(Duration::from_millis(5));
        if let Ok(recv) = device.receive(channel.into(), Some(10)) {
            recv.into_iter()
                .for_each(|msg| println!("{}", msg));
        }
        std::thread::sleep(Duration::from_millis(100));

        count += 1;
        if count > 10 {
            break;
        }
    }

    device.close(channel.into())?;

    Ok(())
}



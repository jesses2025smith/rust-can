use rs_can::{CanDevice, CanError, CanFrame, DeviceBuilder};
use socketcan_rs::{CanMessage, SocketCan};

fn device_builder(iface: String) -> anyhow::Result<SocketCan, CanError> {
    let mut builder = DeviceBuilder::new();
    builder.add_config(iface, Default::default());
    builder.build()
}

#[tokio::test]
async fn test_driver() -> anyhow::Result<(), CanError> {
    let iface = "vcan0".to_string();

    let mut device1 = device_builder(iface.clone())?;
    let mut device2 = device_builder(iface.clone())?;

    loop {
        let data = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07];
        let mut message = CanMessage::new(0x1234, &data).unwrap();
        message.set_channel(iface.clone());
        match device1.transmit(message, None).await {
            Ok(()) => match device2.receive(iface.clone(), None).await {
                Ok(frames) =>
                    if !frames.is_empty() {
                        frames.into_iter()
                            .for_each(|f| println!("{}", f));
                        break;
                    }
                Err(e) => match e {
                    CanError::TimeoutError(_) => {},
                    e => eprintln!("{:?}", e),
                }
            },
            Err(_) => continue,
        }
    }

    device1.shutdown();
    device2.shutdown();

    Ok(())
}

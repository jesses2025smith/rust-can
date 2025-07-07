use rs_can::{CanDevice, CanError, CanFrame, DeviceBuilder};
use socketcan_rs::{CanMessage, SocketCan};

#[tokio::test]
async fn test_driver() -> anyhow::Result<(), CanError> {
    let iface = "vcan0".to_string();
    let mut builder = DeviceBuilder::new();
    builder.add_config(iface.clone(), Default::default());

    let device = builder.build::<SocketCan>()?;

    let data = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07];
    let mut message = CanMessage::new(0x1234, &data).unwrap();
    message.set_channel(iface.clone());
    device.transmit(message, None).await?;

    let data = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
    let mut message = CanMessage::new(0x1234, &data).unwrap();
    message.set_channel(iface.clone());
    device.transmit(message, None).await?;

    loop {
        match device.receive(iface.clone(), None).await {
            Ok(frames) =>
                if !frames.is_empty() {
                    frames.into_iter()
                        .for_each(|f| println!("{}", f));
                    break;
                }
            Err(e) => match e {
                CanError::TimeoutError(_) => {},
                e => return Err(e),
            }
        }
    }

    Ok(())
}

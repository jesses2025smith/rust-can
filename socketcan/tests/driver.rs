use rs_can::{CanDevice, CanError, CanFrame, DeviceBuilder};
use socketcan_rs::{CanMessage, SocketCan};

#[test]
fn test_driver() -> anyhow::Result<(), CanError> {
    let iface = "vcan0";
    let mut builder = DeviceBuilder::new();
    builder.add_config(iface, Default::default());

    let device: SocketCan = builder.build()?;

    let data = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x10];
    let mut message = CanMessage::new(0x1234, &data).unwrap();
    message.set_channel(iface.to_string());
    device.transmit(message, None)?;

    let data = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
    let mut message = CanMessage::new(0x1234, &data).unwrap();
    message.set_channel(iface.to_string());
    device.transmit(message, None)?;

    loop {
        match device.receive(iface.to_string(), None) {
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

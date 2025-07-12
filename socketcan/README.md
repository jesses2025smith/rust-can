[![Latest version](https://img.shields.io/crates/v/socketcan-rs.svg)](https://crates.io/crates/socketcan-rs)
[![Documentation](https://docs.rs/socketcan-rs/badge.svg)](https://docs.rs/socketcan-rs)
![LGPL](https://img.shields.io/badge/license-LGPL-green.svg)
![MIT](https://img.shields.io/badge/license-MIT-yellow.svg)

## Overview
**socketcan-rs** is a driver for SocketCAN device.

It is a part of rust-can driver.

### Prerequisites
- Rust 1.70 or higher
- Cargo (included with Rust)

### Adding to Your Project

To use **socketcan-rs** in your Rust project, add it as a dependency in your `Cargo.toml`:

```toml
[dependencies]
socketcan-rs = { version="lastest-version" }
rs-can = { version="lastest-version" }
```

```shell
sudo ip link add dev vcan0 type vcan
sudo ip link set dev vcan0 up
candump vcan0   # show vcan0 message
```

```rust
use rs_can::{CanDevice, CanError, CanFrame, DeviceBuilder};
use socketcan_rs::{CanMessage, SocketCan};

#[tokio::main]
async fn main() -> Result<(), CanError> {
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
```

## Contributing

We're always looking for users who have thoughts on how to make `socketcan-rs` better, or users with
interesting use cases.

Of course, we're also happy to accept code contributions for outstanding feature requests!
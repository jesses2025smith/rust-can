[![Latest version](https://img.shields.io/crates/v/socketcan-rs.svg)](https://crates.io/crates/socketcan-rs)
[![Documentation](https://docs.rs/socketcan-rs/badge.svg)](https://docs.rs/socketcan-rs)
![LGPL](https://img.shields.io/badge/license-LGPL-green.svg)
![MIT](https://img.shields.io/badge/license-MIT-yellow.svg)

## Overview
**socketcan-rs** is a driver for SocketCAN device.

It is a part of rust-can driver.

- v0.1.x is deprecated
- v0.2.x is sync
- v0.3.x and higher is async

### Prerequisites
- Rust 1.80 or higher
- Cargo (included with Rust)

### Adding to Your Project
```toml
[dependencies]
socketcan-rs = { version="lastest-version" }
rs-can = { version="lastest-version" }
```

### Use virtual can on Linux
```shell
sudo ip link add dev vcan0 type vcan
sudo ip link set dev vcan0 up
candump vcan0   # show vcan0 message
```

```rust
use rs_can::{CanDevice, CanError, CanFrame, DeviceBuilder};
use socketcan_rs::{CanMessage, SocketCan};

fn device_builder(iface: String) -> anyhow::Result<SocketCan, CanError> {
    let mut builder = DeviceBuilder::new();
    builder.add_config(iface, Default::default());
    builder.build()
}

#[tokio::main]
async fn main() -> Result<(), CanError> {
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
                    e => {
                        eprintln!("{:?}", e);
                    },
                }
            },
            Err(_) => continue,
        }
    }

    device1.shutdown();
    device2.shutdown();

    Ok(())
}
```

## Contributing

We're always looking for users who have thoughts on how to make `socketcan-rs` better, or users with
interesting use cases.

Of course, we're also happy to accept code contributions for outstanding feature requests!
[![Latest version](https://img.shields.io/crates/v/zlgcan.svg)](https://crates.io/crates/zlgcan)
[![Documentation](https://docs.rs/zlgcan/badge.svg)](https://docs.rs/zlgcan)
![LGPL](https://img.shields.io/badge/license-LGPL-green.svg)
![MIT](https://img.shields.io/badge/license-MIT-yellow.svg)
![Downloads](https://img.shields.io/crates/d/zlgcan)

## Overview
 **zlgcan** is a cross-platform driver for ZLG(周立功) device. Include windows and linux. 
 
 It is a part of rust-can driver.

 It also can use UDS-protocol directly.

 Please refer to `examples` for usage examples

- v0.1.x is deprecated
- v0.2.x is sync
- v0.3.x and higher is async
- the [official library](https://github.com/jesses2025smith/rust-can/tree/zlg-lib)

## Device list
 * USBCAN-I/II
 * USBCANFD-200U
 * USNCANFD-400U(only channel 1 and channel 2 can be used)
 * USBCANFD-800U

### Prerequisites
 - Rust 1.80 or higher
 - Cargo (included with Rust)

### Adding to Your Project

To use **zlgcan** in your Rust project, add it as a dependency in your `Cargo.toml`:

```toml
[dependencies]
rs-can = { version="lastest-version" }
zlgcan = { version="lastest-version" }
```

### Example

```rust
use rs_can::{CanDevice, CanError, CanFrame, ChannelConfig, DeviceBuilder};
use zlgcan_rs::{can::{CanMessage, ZCanChlMode, ZCanChlType}, device::ZCanDeviceType, driver::{ZDevice, ZDriver}, CHANNEL_MODE, CHANNEL_TYPE, DEVICE_INDEX, DEVICE_TYPE, LIBPATH};

#[tokio::main]
async fn main() -> Result<(), CanError> {
    let mut builder = DeviceBuilder::new();

    let mut ch1_cfg = ChannelConfig::new(500_000);
    ch1_cfg.add_other(CHANNEL_MODE, Box::new(ZCanChlMode::Normal))
        .add_other(CHANNEL_TYPE, Box::new(ZCanChlType::CAN));

    let mut ch2_cfg = ChannelConfig::new(500_000);
    ch2_cfg.add_other(CHANNEL_MODE, Box::new(ZCanChlMode::Normal))
        .add_other(CHANNEL_TYPE, Box::new(ZCanChlType::CAN));

    builder
        .add_other(LIBPATH, Box::new("library".to_string()))
        .add_other(DEVICE_TYPE, Box::new(ZCanDeviceType::ZCAN_USBCANFD_200U))
        .add_other(DEVICE_INDEX, Box::new(0))
        .add_config(0, ch1_cfg)
        .add_config(1, ch2_cfg);

    let device = builder.build::<ZDriver>()?;

    let data = vec![0x02, 0x10, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00];
    let mut msg = CanMessage::new(0x7DF, &data).unwrap();
    msg.set_channel(1);

    device.transmit(msg, None).await?;

    let results = device.receive(1, None).await?;
    println!("{:?}", results);

    Ok(())
}
```

### Create library and configuration

 * Create folder and ensure the file of folder like:
    ```shell
    library
     ├── bitrate.cfg.yaml
     ├── linux
     │   └── x86_64
     └── windows
         ├── x86
         └── x86_64
    ```
    and copy all files into correct directory.

    The basic [library](https://github.com/jesses2025smith/rust-can/blob/master/zlgcan/library).
    The [bitrate.cfg.yaml](https://github.com/jesses2025smith/rust-can/blob/master/zlgcan/library/bitrate.cfg.yaml)


 * Configure your device builder:
   ```rust
   fn main() {
       let mut builder = DeviceBuilder::new();
       builder
           .add_other(LIBPATH, Box::new("library".to_string()))
   }
   ```

### Known defects
 * The timestamp of frame is incorrect.

## Contributing

We're always looking for users who have thoughts on how to make `zlgcan` better, or users with
interesting use cases.  

Of course, we're also happy to accept code contributions for outstanding feature requests!
]()
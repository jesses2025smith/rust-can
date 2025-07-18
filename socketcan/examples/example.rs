use rs_can::{CanDevice, CanError, CanFrame, DeviceBuilder};
use socketcan_rs::{CanMessage, SocketCan};
use std::time::Duration;
use tokio::{signal::ctrl_c, time::sleep};

fn device_builder(iface: String) -> anyhow::Result<SocketCan, CanError> {
    let mut builder = DeviceBuilder::new();
    builder.add_config(iface, Default::default());
    builder.build()
}

#[tokio::main]
async fn main() -> anyhow::Result<(), CanError> {
    let iface = "vcan0".to_string();

    let mut device1 = device_builder(iface.clone())?;
    let mut device2 = device_builder(iface.clone())?;

    let dev_clone1 = device1.clone();
    let dev_clone2 = device2.clone();
    let iface_clone = iface.clone();
    let send_task = tokio::spawn(async move {
        loop {
            let data = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07];
            let mut message = CanMessage::new(0x1234, &data).unwrap();
            message.set_channel(iface_clone.clone());
            if let Err(e) = dev_clone1.transmit(message, None).await {
                eprintln!("transmit device error {:?}", e);
            };

            sleep(Duration::from_millis(500)).await;
        }
    });

    let recv_task = tokio::spawn(async move {
        loop {
            match dev_clone2.receive(iface.clone(), None).await {
                Ok(frames) =>
                    if !frames.is_empty() {
                        frames.into_iter()
                            .for_each(|f| println!("{}", f));
                    }
                Err(_) => sleep(Duration::from_millis(100)).await,
            }
        }
    });


    let _guard = scopeguard::guard((), |_| {
        futures::executor::block_on(async {
            send_task.abort();
            recv_task.abort();
            device1.shutdown();
            device2.shutdown();
            println!("program exit normally");
        });
    });

    match ctrl_c().await {
        Ok(()) => {
            println!("\nCtrl+C Signal, exiting...");
        }
        Err(err) => {
            eprintln!("Ctrl+C error: {:?}", err);
        }
    }

    Ok(())
}

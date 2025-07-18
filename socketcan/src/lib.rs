mod constants;
pub use constants::*;
mod driver;
pub use driver::*;
mod frame;
pub use frame::*;
mod socket;
pub use socket::*;

use rs_can::{CanDevice, CanError, CanFilter, CanFrame, CanResult, DeviceBuilder};
use std::{sync::Arc, time::Duration};

#[async_trait::async_trait]
impl CanDevice for SocketCan {
    type Channel = String;
    type Frame = CanMessage;

    #[inline(always)]
    fn opened_channels(&self) -> Vec<Self::Channel> {
        self.sockets.iter()
            .map(|(c, _)| c.clone())
            .collect()
    }

    #[inline(always)]
    async fn transmit(&self, msg: Self::Frame, timeout: Option<u32>) -> CanResult<(), CanError> {
        match timeout {
            Some(timeout) => self.write_timeout(msg, Duration::from_millis(timeout as u64)),
            None => self.write(msg),
        }
    }

    #[inline(always)]
    async fn receive(&self, channel: Self::Channel, timeout: Option<u32>) -> CanResult<Vec<Self::Frame>, CanError> {
        let timeout = timeout.unwrap_or(0);
        let mut msg = self.read_timeout(&channel, Duration::from_millis(timeout as u64))?;
        msg.set_channel(channel);
            // .set_direct(CanDirect::Receive);
        Ok(vec![msg, ])
    }

    #[inline(always)]
    fn shutdown(&mut self) {
        match Arc::get_mut(&mut self.sockets) {
            Some(s) => s.clear(),
            None => (),
        }
    }
}

impl TryFrom<DeviceBuilder<String>> for SocketCan {
    type Error = CanError;

    fn try_from(builder: DeviceBuilder<String>) -> Result<Self, Self::Error> {
        let mut device = SocketCan::new();
        builder.channel_configs()
            .iter()
            .try_for_each(|(chl, cfg)| {
                let canfd = cfg.get_other::<bool>(CANFD)?
                    .unwrap_or_default();
                device.init_channel(chl, canfd)?;

                if let Some(filters) = cfg.get_other::<Vec<CanFilter>>(FILTERS)? {
                    device.set_filters(chl, &filters)?;
                }

                if let Some(loopback) = cfg.get_other::<bool>(LOOPBACK)? {
                    device.set_loopback(chl, loopback)?;
                }

                if let Some(recv_own_msg) = cfg.get_other::<bool>(RECV_OWN_MSG)? {
                    device.set_recv_own_msgs(chl, recv_own_msg)?;
                }

                Ok(())
            })?;

        Ok(device)
    }
}

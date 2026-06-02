mod driver;
mod frame;
mod socket;

pub use self::{driver::*, frame::*, socket::*};

use rs_can::{CanDevice, CanDirection, CanError, CanFrame, CanResult, ChannelMode, DeviceBuilder};
use std::{sync::Arc, time::Duration};

#[async_trait::async_trait]
impl CanDevice for SocketCan {
    type Channel = String;
    type Frame = SocketCanFrame;

    fn new(builder: DeviceBuilder<String>) -> CanResult<Self> {
        let mut device = SocketCan::new();
        builder
            .channel_configs()
            .iter()
            .try_for_each(|(chl, cfg)| {
                let canfd = cfg.data_bitrate.is_some();
                device.init_channel(chl, canfd)?;

                if !cfg.filters.is_empty() {
                    device.set_filters(chl, &cfg.filters)?;
                }

                if let Some(mode) = cfg.mode {
                    match mode {
                        ChannelMode::Normal => {}
                        ChannelMode::Loopback => device.set_loopback(chl, true)?,
                        ChannelMode::ListenOnly => return Err(CanError::NotSupportedError),
                        ChannelMode::OneShot => return Err(CanError::NotSupportedError),
                    }
                }

                if let Some(recv_own_msg) = cfg.recv_own_msg {
                    device.set_recv_own_msgs(chl, recv_own_msg)?;
                }

                Ok(())
            })?;

        Ok(device)
    }

    #[inline(always)]
    fn opened_channels(&self) -> Vec<Self::Channel> {
        self.sockets.iter().map(|(c, _)| c.clone()).collect()
    }

    #[inline(always)]
    async fn transmit(&self, msg: Self::Frame, timeout: Option<u32>) -> CanResult<()> {
        let mut msg = msg;
        msg.set_direction(CanDirection::Transmit);
        match timeout {
            Some(timeout) => self.write_timeout(msg, Duration::from_millis(timeout as u64)),
            None => self.write(msg),
        }
    }

    #[inline(always)]
    async fn receive(
        &self,
        channel: Self::Channel,
        timeout: Option<u32>,
    ) -> CanResult<Vec<Self::Frame>> {
        let mut msg = match timeout {
            Some(timeout) => self.read_timeout(&channel, Duration::from_millis(timeout as u64))?,
            None => self.read(&channel)?,
        };
        msg.set_channel(channel);
        // .set_direct(CanDirect::Receive);
        Ok(vec![msg])
    }

    #[inline(always)]
    fn shutdown(&mut self) {
        match Arc::get_mut(&mut self.sockets) {
            Some(s) => s.clear(),
            None => (),
        }
    }
}

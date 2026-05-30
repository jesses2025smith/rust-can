mod api;
mod constant;
mod constants;
mod driver;
mod frame;

pub use self::{constants::*, driver::*, frame::*};

use rs_can::{CanDevice, CanFilter, CanResult, DeviceBuilder};

#[async_trait::async_trait]
impl CanDevice for NiCan {
    type Channel = String;
    type Frame = NiCanFrame;

    fn new(builder: DeviceBuilder<String>) -> CanResult<Self> {
        let libpath = builder.get_other::<String>(LIBPATH)?;
        let mut device = NiCan::new(libpath.as_deref())?;
        builder
            .channel_configs()
            .iter()
            .try_for_each(|(chl, cfg)| {
                let filters = cfg
                    .get_other::<Vec<CanFilter>>(FILTERS)?
                    .unwrap_or_default();
                let bitrate = cfg.nominal_bitrate;
                let log_error = cfg.get_other::<bool>(LOG_ERROR)?.unwrap_or_default();

                device.open(chl, filters, bitrate, log_error)
            })?;

        Ok(device)
    }

    #[inline]
    fn is_closed(&self) -> bool {
        self.channels.is_empty()
    }

    #[inline]
    fn opened_channels(&self) -> Vec<Self::Channel> {
        self.channels.keys().map(|v| v.clone()).collect()
    }

    #[inline]
    async fn transmit(&self, msg: Self::Frame, _: Option<u32>) -> CanResult<()> {
        self.transmit_can(msg)
    }

    #[inline]
    async fn receive(
        &self,
        channel: Self::Channel,
        timeout: Option<u32>,
    ) -> CanResult<Vec<Self::Frame>> {
        self.receive_can(channel, timeout)
    }

    #[inline]
    fn shutdown(&mut self) {
        self.channels.iter().for_each(|(c, ctx)| {
            let ret = unsafe { (self.ncCloseObject)(ctx.handle) };

            if let Err(e) = self.check_status(c, ret) {
                rsutil::warn!(
                    "{} error {} when close",
                    Self::channel_info(c),
                    self.status_to_str(e)
                );
            }
        });

        self.channels.clear();
    }
}

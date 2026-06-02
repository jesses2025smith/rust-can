mod constants;
pub use constants::*;
pub mod driver;
mod native;
pub use native::*;

use crate::{
    can::{ZCanFrame, ZCanFrameType},
    device::{DeriveInfo, ZCanDeviceType},
    driver::{ZCan, ZDevice, ZDriver},
};
use rs_can::{CanDevice, CanError, CanFrame, CanKind, CanResult, DeviceBuilder};

#[async_trait::async_trait]
impl CanDevice for ZDriver {
    type Channel = u8;
    type Frame = ZCanFrame;

    fn new(builder: DeviceBuilder<u8>) -> CanResult<Self> {
        let libpath = builder
            .get_other::<String>(LIBPATH)?
            .ok_or(CanError::InitializeError(format!(
                "`{}` not found",
                LIBPATH
            )))?;
        let dev_type =
            builder
                .get_other::<ZCanDeviceType>(DEVICE_TYPE)?
                .ok_or(CanError::InitializeError(format!(
                    "`{}` not found",
                    DEVICE_TYPE
                )))?;
        let dev_idx = builder
            .get_other::<u32>(DEVICE_INDEX)?
            .ok_or(CanError::InitializeError(format!(
                "`{}` not found",
                DEVICE_INDEX
            )))?;
        let derive = builder.get_other::<DeriveInfo>(DERIVE_INFO)?;

        let mut device = Self::native(libpath, dev_type, dev_idx, derive)?;
        device.open()?;

        builder
            .channel_configs()
            .iter()
            .try_for_each(|(&chl, cfg)| device.init_can_chl(chl, cfg))?;

        Ok(device)
    }

    #[inline]
    fn opened_channels(&self) -> Vec<Self::Channel> {
        match &self.handler {
            Some(v) => v.can_channels().keys().map(|v| v.clone()).collect(),
            None => vec![],
        }
    }

    async fn transmit(&self, msg: Self::Frame, _: Option<u32>) -> CanResult<()> {
        let channel = msg.channel();
        let _ = match msg.kind() {
            CanKind::Classical => self.transmit_can(channel, vec![msg]),
            CanKind::FD => self.transmit_canfd(channel, vec![msg]),
            CanKind::XL => Err(CanError::NotSupportedError),
        }?;

        Ok(())
    }

    async fn receive(
        &self,
        channel: Self::Channel,
        timeout: Option<u32>,
    ) -> CanResult<Vec<Self::Frame>> {
        let mut results: Vec<ZCanFrame> = Vec::new();

        let count_can = self.get_can_num(channel, ZCanFrameType::CAN)?;
        if count_can > 0 {
            rsutil::trace!("RUST-CAN - received CAN: {}", count_can);
            let mut frames = self.receive_can(channel, count_can, timeout)?;
            results.append(&mut frames);
        }

        if self.device_type().canfd_support() {
            let count_fd = self.get_can_num(channel, ZCanFrameType::CANFD)?;
            if count_fd > 0 {
                rsutil::trace!("RUST-CAN - received CANFD: {}", count_fd);
                let mut frames = self.receive_canfd(channel, count_fd, timeout)?;
                results.append(&mut frames);
            }
        }

        Ok(results)
    }

    #[inline]
    fn shutdown(&mut self) {
        self.close()
    }
}

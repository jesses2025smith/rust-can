use crate::{
    error::Error,
    frame::{
        identifier::{Filter, Id},
        Frame,
    },
    CanResult,
};
use derive_getters::Getters;
use serde::{Deserialize, Serialize};
use std::{
    any::{type_name, Any},
    collections::HashMap,
    fmt::{self, Debug, Display},
    hash::Hash,
    sync::Weak,
};

#[async_trait::async_trait]
pub trait Listener<C, F>: Send + Sync
where
    F: Frame,
{
    fn as_any(&self) -> &dyn Any;
    /// Callback when frame transmit success.
    async fn on_frame_transmitted(&self, channel: C, id: Id);
    /// Callback when frames received.
    async fn on_frame_received(&self, frames: Weak<Vec<F>>);
}

#[async_trait::async_trait]
pub trait Device: Send + Sync {
    type Channel: Hash + Eq + Display + 'static;
    type Frame: Frame<Channel = Self::Channel>;

    fn new(builder: DeviceBuilder<Self::Channel>) -> CanResult<Self>
    where
        Self: Sized;

    #[inline]
    fn is_closed(&self) -> bool {
        self.opened_channels().is_empty()
    }
    /// get all channels that has opened
    fn opened_channels(&self) -> Vec<Self::Channel>;
    /// Transmit a CAN or CAN-FD Frame.
    async fn transmit(&self, msg: Self::Frame, timeout: Option<u32>) -> CanResult<()>;
    /// Receive CAN and CAN-FD Frames.
    async fn receive(
        &self,
        channel: Self::Channel,
        timeout: Option<u32>,
    ) -> CanResult<Vec<Self::Frame>>;
    /// Close CAN device.
    fn shutdown(&mut self);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum ChannelMode {
    Normal,
    ListenOnly,
    Loopback,
    OneShot,
}

#[derive(Default, Deserialize, Serialize)]
pub struct ChannelConfig {
    /// Nominal bitrate for both CAN and CAN-FD, and data bitrate for CAN-FD.
    pub nominal_bitrate: u32,
    /// Data bitrate for CAN-FD, if not set, it will be the same as nominal bitrate.
    pub data_bitrate: Option<u32>,
    /// Whether the channel has termination resistor, if not set, it will be determined by device implementation.
    pub termination: Option<bool>,
    /// Channel mode, if not set, it will be normal mode. Loopback and ListenOnly modes are useful for testing.
    pub mode: Option<ChannelMode>,
    /// Whether the device can receive its own transmitted messages, if not set, it will be determined by device implementation.
    pub recv_own_msg: Option<bool>,
    /// Filters for the channel, if not set, it will receive all frames. If the device does not support filtering, it will be ignored.
    pub filters: Vec<Filter>,
    #[serde(skip)]
    others: HashMap<String, Box<dyn Any + Send + Sync>>,
}

impl Debug for ChannelConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ChannelConfig")
            .field("nominal_bitrate", &self.nominal_bitrate)
            .field("data_bitrate", &self.data_bitrate)
            .field("termination", &self.termination)
            .field("mode", &self.mode)
            .field("filters", &self.filters)
            .finish_non_exhaustive()
    }
}

impl ChannelConfig {
    pub fn new(bitrate: u32) -> Self {
        Self {
            nominal_bitrate: bitrate,
            ..Default::default()
        }
    }

    pub fn set_data_bitrate(&mut self, bitrate: u32) -> &mut Self {
        self.data_bitrate = Some(bitrate);
        self
    }

    pub fn set_termination(&mut self, termination: bool) -> &mut Self {
        self.termination = Some(termination);
        self
    }

    pub fn set_channel_mode(&mut self, mode: ChannelMode) -> &mut Self {
        self.mode = Some(mode);
        self
    }

    pub fn set_recv_own_msg(&mut self, recv_own_msg: bool) -> &mut Self {
        self.recv_own_msg = Some(recv_own_msg);
        self
    }

    pub fn add_other(&mut self, name: &str, other: Box<dyn Any + Send + Sync>) -> &mut Self {
        self.others.insert(name.into(), other);
        self
    }

    pub fn get_other<T: Clone + 'static>(&self, name: &str) -> CanResult<Option<T>> {
        get_other(&self.others, name)
    }
}

#[derive(Default, Getters)]
pub struct DeviceBuilder<K: Hash + Eq> {
    #[getter(rename = "channel_configs")]
    configs: HashMap<K, ChannelConfig>,
    others: HashMap<String, Box<dyn Any + Send + Sync>>,
}

impl<K: Hash + Eq + Debug> Debug for DeviceBuilder<K> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DeviceBuilder")
            .field("configs", &self.configs)
            .finish_non_exhaustive()
    }
}

impl<K: Hash + Eq + Default> DeviceBuilder<K> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_config(&mut self, channel: K, cfg: ChannelConfig) -> &mut Self {
        self.configs.insert(channel.into(), cfg);
        self
    }

    pub fn add_other(&mut self, name: &str, cfg: Box<dyn Any + Send + Sync>) -> &mut Self {
        self.others.insert(name.into(), cfg);
        self
    }

    pub fn get_other<T: Clone + 'static>(&self, name: &str) -> CanResult<Option<T>> {
        get_other(&self.others, name)
    }

    pub fn build<T: Device<Channel = K>>(self) -> CanResult<T> {
        T::new(self)
    }
}

#[inline(always)]
fn get_other<T: Clone + 'static>(
    others: &HashMap<String, Box<dyn Any + Send + Sync>>,
    name: &str,
) -> CanResult<Option<T>> {
    match others.get(name) {
        Some(v) => Ok(Some(
            v.downcast_ref::<T>()
                .ok_or(Error::OtherError(format!(
                    "type mismatched for `{}` expected: `{}`",
                    name,
                    type_name::<T>()
                )))?
                .clone(),
        )),
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_other_returns_inserted_value() {
        let mut builder = DeviceBuilder::<u8>::new();
        builder.add_other("answer", Box::new(42_u32));

        let value = builder.get_other::<u32>("answer").unwrap();
        assert_eq!(value, Some(42));
    }

    #[test]
    fn get_other_reports_type_mismatch() {
        let mut builder = DeviceBuilder::<u8>::new();
        builder.add_other("answer", Box::new(42_u32));

        let err = builder.get_other::<String>("answer").unwrap_err();
        assert!(matches!(err, Error::OtherError(_)));
    }
}

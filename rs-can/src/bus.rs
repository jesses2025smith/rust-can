use crate::{device::Device, CanResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BusState {
    ErrorActive,
    ErrorPassive,
    BusOff,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ErrorCounters {
    pub tx: u32,
    pub rx: u32,
}

pub trait BusDiagnostic: Device {
    fn bus_state(&self, channel: Self::Channel) -> CanResult<BusState>;
    fn error_counters(&self, channel: Self::Channel) -> CanResult<ErrorCounters>;
    fn recover_bus_off(&self, channel: Self::Channel) -> CanResult<()>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BusCapabilities {
    pub can_fd: bool,
    pub bitrate_switch: bool,
    pub error_state_indicator: bool,
    pub listen_only: bool,
    pub loopback: bool,
    pub hardware_timestamp: bool,
    pub bus_diagnostics: bool,
}

pub trait BusCapability: Device {
    fn capabilities(&self) -> BusCapabilities;
}

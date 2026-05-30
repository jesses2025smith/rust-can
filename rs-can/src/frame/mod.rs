pub(crate) mod identifier;

use self::identifier::{CanFdFlags, Id};
use crate::utils;
use crate::CanResult;
use std::fmt::{Display, Formatter, Write};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimestampSource {
    System,
    Hardware,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Timestamp {
    pub nanos: u128,
    pub source: TimestampSource,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum FrameFormat {
    #[default]
    Data,
    Remote,
    Error,
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
    #[default]
    Classical,
    FD,
    XL,
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Direction {
    #[default]
    Transmit,
    Receive,
}

impl Display for Direction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Transmit => f.write_str("Tx"),
            Self::Receive => f.write_str("Rx"),
        }
    }
}

/// CAN 2.0 | CAN 1.0
pub trait Frame: Send + Sync {
    type Channel: Display;

    fn new_can(id: Id, data: &[u8]) -> CanResult<Self>
    where
        Self: Sized;

    fn new_remote(id: Id, dlc: u8) -> CanResult<Self>
    where
        Self: Sized;

    fn new_can_fd(id: Id, data: &[u8], flags: CanFdFlags) -> CanResult<Self>
    where
        Self: Sized;

    fn id(&self) -> Id;
    fn channel(&self) -> Self::Channel;
    fn set_channel(&mut self, v: Self::Channel) -> &mut Self
    where
        Self: Sized;

    fn kind(&self) -> Kind;
    fn format(&self) -> FrameFormat;

    fn data(&self) -> &[u8];
    fn len(&self) -> usize;
    fn dlc(&self) -> CanResult<u8> {
        utils::can_dlc(self.len(), self.kind())
    }

    fn direction(&self) -> Direction;
    fn set_direction(&mut self, d: Direction) -> &mut Self
    where
        Self: Sized;

    fn timestamp(&self) -> Option<Timestamp>;
    fn set_timestamp(&mut self, ts: Option<Timestamp>) -> &mut Self
    where
        Self: Sized;

    fn is_bitrate_switch(&self) -> bool;
    fn set_bitrate_switch(&mut self, v: bool) -> &mut Self
    where
        Self: Sized;

    fn is_remote(&self) -> bool {
        matches!(self.format(), FrameFormat::Remote)
    }

    fn is_error_frame(&self) -> bool {
        matches!(self.format(), FrameFormat::Error)
    }

    fn is_extended(&self) -> bool {
        matches!(self.id(), Id::Extended(_))
    }

    fn is_esi(&self) -> bool;
    fn set_esi(&mut self, v: bool) -> &mut Self
    where
        Self: Sized;
}

impl<T: Display> Display for dyn Frame<Channel = T> {
    /// Output Frame as `asc` String.
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let data_str = if self.is_remote() {
            " ".to_owned()
        } else {
            self.data().iter().fold(String::new(), |mut out, &b| {
                let _ = write!(out, "{b:02x} ");
                out
            })
        };

        match self.kind() {
            Kind::Classical => {
                let timestamp_secs = self
                    .timestamp()
                    .map(|ts| ts.nanos as f64 / 1_000_000_000.)
                    .unwrap_or_default();
                write!(
                    f,
                    "{:.3} {} {}{: <4} {} {} {} {}",
                    timestamp_secs,
                    self.channel(),
                    format!("{: >8x}", self.id().as_raw()),
                    if self.is_extended() { "x" } else { "" },
                    self.direction(),
                    // if self.is_rx() { "Rx" } else { "Tx" },
                    if self.is_remote() { "r" } else { "d" },
                    format!("{: >2}", self.len()),
                    data_str,
                )
            }
            Kind::FD => {
                let timestamp_secs = self
                    .timestamp()
                    .map(|ts| ts.nanos as f64 / 1_000_000_000.)
                    .unwrap_or_default();
                let dlc = self.dlc().map_err(|_| std::fmt::Error)?;
                let mut flags = 1 << 12;
                write!(
                    f,
                    "{:.3} CANFD {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {}",
                    timestamp_secs,
                    self.channel(),
                    self.direction(),
                    // if self.is_rx() { "Rx" } else { "Tx" },
                    format!("{: >8x}", self.id().as_raw()),
                    if self.is_bitrate_switch() {
                        flags |= 1 << 13;
                        1
                    } else {
                        0
                    },
                    if self.is_esi() {
                        flags |= 1 << 14;
                        1
                    } else {
                        0
                    },
                    format!("{: >2}", dlc),
                    format!("{: >2}", self.len()),
                    data_str,
                    format!("{: >8}", 0), // message_duration
                    format!("{: <4}", 0), // message_length
                    format!("{: >8x}", flags),
                    format!("{: >8}", 0), // crc
                    format!("{: >8}", 0), // bit_timing_conf_arb
                    format!("{: >8}", 0), // bit_timing_conf_data
                    format!("{: >8}", 0), // bit_timing_conf_ext_arb
                    format!("{: >8}", 0), // bit_timing_conf_ext_data
                )
            }
            Kind::XL => {
                // TODO
                write!(f, "CANXL Frame")
            }
        }
    }
}

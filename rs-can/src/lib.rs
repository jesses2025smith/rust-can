mod bus;
pub mod can_utils;
mod constants;
mod device;
mod error;
mod frame;

pub(crate) use can_utils as utils;

pub type CanResult<R> = Result<R, crate::error::Error>;

pub use crate::{
    constants::*,
    device::{
        ChannelConfig, ChannelMode, Device as CanDevice, DeviceBuilder, Listener as CanListener,
    },
    error::Error as CanError,
    frame::{
        identifier::{
            CanFdFlags, CanXlFlags, ExtendedId, Filter as CanFilter, Id as CanId, IdentifierFlags,
            StandardId,
        },
        Direction as CanDirection, Frame as CanFrame, FrameFormat, Kind as CanKind, Timestamp,
        TimestampSource,
    },
};
pub use bus::*;

use crate::{
    constants::{DEFAULT_PADDING, MAX_FD_FRAME_SIZE, MAX_FRAME_SIZE, MAX_XL_FRAME_SIZE},
    error::Error,
    frame::{Kind, Timestamp, TimestampSource},
    CanResult,
};
use std::time::{SystemTime, UNIX_EPOCH};

/// resize data with default padding.
#[inline]
pub fn data_resize(data: &mut Vec<u8>, size: usize) {
    data.resize(size, DEFAULT_PADDING);
}

#[deprecated(since = "0.4.0", note = "deprecated in favor of can_kind_by_len")]
#[inline]
pub fn can_kind_by_len(len: usize) -> CanResult<Kind> {
    match len {
        ..=MAX_FRAME_SIZE => Ok(Kind::Classical),
        ..=MAX_FD_FRAME_SIZE => Ok(Kind::FD),
        ..=MAX_XL_FRAME_SIZE => Ok(Kind::XL),
        _ => Err(Error::OtherError("length of frame is out of range!".into())),
    }
}

/// get CAN dlc
#[inline]
pub fn can_dlc(length: usize, kind: Kind) -> CanResult<u8> {
    match kind {
        Kind::Classical => match length {
            ..=MAX_FRAME_SIZE => Ok(length as u8),
            _ => Err(Error::OtherError("length of frame is out of range!".into())),
        },
        Kind::FD => match length {
            ..=MAX_FRAME_SIZE => Ok(length as u8),
            9..=12 => Ok(9),
            13..=16 => Ok(10),
            17..=20 => Ok(11),
            21..=24 => Ok(12),
            25..=32 => Ok(13),
            33..=48 => Ok(14),
            49..=MAX_FD_FRAME_SIZE => Ok(15),
            _ => Err(Error::OtherError("length of frame is out of range!".into())),
        },
        Kind::XL => Err(Error::OtherError("XL is not supported now!".into())),
    }
}

#[inline]
pub fn system_timestamp() -> Timestamp {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(v) => Timestamp {
            nanos: v.as_nanos(),
            source: TimestampSource::System,
        },
        Err(e) => {
            rsutil::warn!("RUST-CAN - SystemTimeError: {0} when conversion failed!", e);
            Timestamp {
                nanos: 0,
                source: TimestampSource::Unknown,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(deprecated)]
    #[test]
    fn can_kind_by_len_classifies_lengths() {
        assert_eq!(can_kind_by_len(8).unwrap(), Kind::Classical);
        assert_eq!(can_kind_by_len(9).unwrap(), Kind::FD);
        assert_eq!(can_kind_by_len(64).unwrap(), Kind::FD);
        assert_eq!(can_kind_by_len(65).unwrap(), Kind::XL);
        assert_eq!(can_kind_by_len(2048).unwrap(), Kind::XL);
        assert!(can_kind_by_len(2049).is_err());
    }

    #[test]
    fn can_fd_dlc_matches_canonical_mapping() {
        assert_eq!(can_dlc(8, Kind::FD).unwrap(), 8);
        assert_eq!(can_dlc(9, Kind::FD).unwrap(), 9);
        assert_eq!(can_dlc(12, Kind::FD).unwrap(), 9);
        assert_eq!(can_dlc(13, Kind::FD).unwrap(), 10);
        assert_eq!(can_dlc(16, Kind::FD).unwrap(), 10);
        assert_eq!(can_dlc(17, Kind::FD).unwrap(), 11);
        assert_eq!(can_dlc(20, Kind::FD).unwrap(), 11);
        assert_eq!(can_dlc(21, Kind::FD).unwrap(), 12);
        assert_eq!(can_dlc(24, Kind::FD).unwrap(), 12);
        assert_eq!(can_dlc(25, Kind::FD).unwrap(), 13);
        assert_eq!(can_dlc(32, Kind::FD).unwrap(), 13);
        assert_eq!(can_dlc(33, Kind::FD).unwrap(), 14);
        assert_eq!(can_dlc(48, Kind::FD).unwrap(), 14);
        assert_eq!(can_dlc(49, Kind::FD).unwrap(), 15);
        assert_eq!(can_dlc(64, Kind::FD).unwrap(), 15);
    }
}

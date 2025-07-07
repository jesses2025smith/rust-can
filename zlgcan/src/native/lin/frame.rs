use crate::native::lin::enums::{ZLinCheckSumMode, ZLinDataType};
use rs_can::CanError;
use std::ffi::{c_uchar, c_ulong, c_ushort};

#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct ZLinRxData {
    pub timestamp: c_ulong,
    pub len: c_uchar,
    pub dir: c_uchar,
    pub chk_sum: c_uchar,
    pub reserved: [c_uchar; 13usize],
    pub data: [c_uchar; 8usize],
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct ZLinData {
    pub pid: c_uchar,
    pub rx_data: ZLinRxData,
    pub reserved: [c_uchar; 7usize],
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct LinErrData {
    pub timestamp: c_ulong,
    pub pid: c_uchar,
    pub len: c_uchar,
    pub data: [c_uchar; 8usize],
    pub err_data: c_ushort,
    pub dir: c_uchar,
    pub chk_sum: c_uchar,
    pub reserved: [c_uchar; 10usize],
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct LinEventData {
    pub timestamp: c_ulong,
    pub event: c_uchar,
    pub reserved: [c_uchar; 7usize],
}

#[allow(non_snake_case)]
#[repr(C)]
pub union ZLinFrameDataUnion {
    data: ZLinData,
    err: LinErrData,
    event: LinEventData,
    raw: [c_uchar; 46usize],
}

impl ZLinFrameDataUnion {
    pub fn from_data(data: ZLinData) -> Self {
        Self { data }
    }

    pub fn from_error(err: LinErrData) -> Self {
        Self { err }
    }

    pub fn from_event(event: LinEventData) -> Self {
        Self { event }
    }

    pub fn from_raw(raw: [c_uchar; 46usize]) -> Self {
        Self { raw }
    }
}

#[repr(C)]
pub struct ZLinFrame {
    pub chl: c_uchar,
    pub data_type: c_uchar,
    pub data: ZLinFrameDataUnion,
}

impl ZLinFrame {
    pub fn default_data() -> Self {
        Self {
            chl: Default::default(),
            data_type: ZLinDataType::TypeData as u8,
            data: ZLinFrameDataUnion::from_data(Default::default()),
        }
    }
    pub fn default_err() -> Self {
        Self {
            chl: Default::default(),
            data_type: ZLinDataType::TypeError as u8,
            data: ZLinFrameDataUnion::from_error(Default::default()),
        }
    }
    pub fn default_event() -> Self {
        Self {
            chl: Default::default(),
            data_type: ZLinDataType::TypeEvent as u8,
            data: ZLinFrameDataUnion::from_event(Default::default()),
        }
    }
}

impl ZLinFrame {
    pub fn new(chl: u8, data_type: ZLinDataType, data: ZLinFrameDataUnion) -> Self {
        Self {
            chl,
            data_type: data_type as u8,
            data,
        }
    }
}

#[allow(non_snake_case)]
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct ZLinSubscribe {
    pub ID: c_uchar,
    pub dataLen: c_uchar,
    pub chkSumMode: c_uchar,
    #[allow(dead_code)]
    pub reserved: [c_uchar; 5usize],
}

#[allow(non_snake_case)]
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct ZLinPublish {
    pub ID: c_uchar,
    pub dataLen: c_uchar,
    pub data: [c_uchar; 8usize],
    pub chkSumMode: c_uchar,
    #[allow(dead_code)]
    pub reserved: [c_uchar; 5usize],
}

#[allow(non_snake_case)]
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ZLinPublishEx {
    ID: c_uchar,
    dataLen: c_uchar,
    data: [c_uchar; 64usize],
    chkSumMode: c_uchar,
    reserved: [c_uchar; 5usize],
}

impl ZLinPublishEx {
    pub fn new<T>(pid: u8, data: T, cs_mode: ZLinCheckSumMode) -> Result<Self, CanError>
    where
        T: AsRef<[u8]>,
    {
        let mut data = Vec::from(data.as_ref());
        let len = data.len();
        match len {
            0..=64 => {
                data.resize(64usize, 0);
                Ok(Self {
                    ID: pid,
                    dataLen: len as u8,
                    data: data
                        .try_into()
                        .map_err(|_| CanError::other_error("invalid data length"))?,
                    chkSumMode: cs_mode as c_uchar,
                    reserved: Default::default(),
                })
            }
            _ => Err(CanError::other_error("parameter not supported")),
        }
    }
}

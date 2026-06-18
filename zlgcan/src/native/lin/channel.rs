use crate::native::lin::enums::{ZLinCheckSumMode, ZLinMode};
use rs_can::{CanError, CanResult};
#[cfg(not(target_os = "windows"))]
use std::ffi::c_ushort;
use std::ffi::{c_uchar, c_uint};

#[allow(non_snake_case)]
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ZLinChlCfg {
    linMode: c_uchar,    // 是否作为主机，0-从机，1-主机
    chkSumMode: c_uchar, // 校验方式，1-经典校验 2-增强校验 3-自动(对应eZLINChkSumMode的模式)
    #[cfg(target_os = "windows")]
    maxLength: c_uchar,  // 最大数据长度，8~64
    #[cfg(target_os = "windows")]
    reserved: c_uchar,   // 保留
    #[cfg(not(target_os = "windows"))]
    reserved: c_ushort,  // Linux ABI keeps these two bytes reserved.
    linBaud: c_uint,     // 波特率，取值1000~20000
}

impl ZLinChlCfg {
    /// Create LIN channel configuration.
    /// max_len is required only windows.
    pub fn new(
        mode: ZLinMode,
        cs_mode: ZLinCheckSumMode,
        bitrate: u32,
        max_len: Option<u8>,
    ) -> CanResult<Self> {
        if !(1000..=20000).contains(&bitrate) {
            return Err(CanError::other_error("parameter not supported"));
        }

        #[cfg(target_os = "windows")]
        match max_len {
            Some(v) => match v {
                8..=64 => Ok(Self {
                    linMode: mode as c_uchar,
                    chkSumMode: cs_mode as c_uchar,
                    maxLength: v,
                    reserved: Default::default(),
                    linBaud: bitrate,
                }),
                _ => Err(CanError::other_error("parameter not supported")),
            },
            None => Ok(Self {
                linMode: mode as c_uchar,
                chkSumMode: cs_mode as c_uchar,
                maxLength: Default::default(),
                reserved: Default::default(),
                linBaud: bitrate,
            }),
        }

        #[cfg(not(target_os = "windows"))]
        {
            let _ = max_len;
            Ok(Self {
                linMode: mode as c_uchar,
                chkSumMode: cs_mode as c_uchar,
                reserved: Default::default(),
                linBaud: bitrate,
            })
        }
    }
}

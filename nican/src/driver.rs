use crate::{api::*, constant, CanMessage};
use rs_can::{CanError, CanFilter, CanFrame};
use std::{
    collections::HashMap,
    ffi::{c_char, CStr, CString},
};
use windows::{
    core::PCSTR,
    Win32::Foundation::HMODULE,
    Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress},
};

#[derive(Debug, Clone)]
struct NiCanContext {
    pub(crate) handle: NCTYPE_OBJH,
    pub(crate) filters: Vec<CanFilter>,
    pub(crate) bitrate: u32,
    pub(crate) log_errors: bool,
}

#[allow(non_snake_case)]
#[derive(Debug, Clone)]
pub struct NiCan {
    pub(crate) _dll: HMODULE,
    pub(crate) channels: HashMap<String, NiCanContext>,
    pub(crate) ncConfig: unsafe extern "system" fn(
        NCTYPE_STRING,
        NCTYPE_UINT32,
        NCTYPE_ATTRID_P,
        NCTYPE_UINT32_P,
    ) -> NCTYPE_STATUS,
    pub(crate) ncOpenObject: unsafe extern "system" fn(NCTYPE_STRING, NCTYPE_OBJH_P) -> NCTYPE_STATUS,
    pub(crate) ncAction: unsafe extern "system" fn(NCTYPE_OBJH, NCTYPE_OPCODE, NCTYPE_UINT32) -> NCTYPE_STATUS,
    pub(crate) ncCloseObject: unsafe extern "system" fn(NCTYPE_OBJH) -> NCTYPE_STATUS,
    pub(crate) ncWrite: unsafe extern "system" fn(NCTYPE_OBJH, NCTYPE_UINT32, NCTYPE_ANY_P) -> NCTYPE_STATUS,
    pub(crate) ncRead: unsafe extern "system" fn(NCTYPE_OBJH, NCTYPE_UINT32, NCTYPE_ANY_P) -> NCTYPE_STATUS,
    pub(crate) ncWaitForState: unsafe extern "system" fn(
        NCTYPE_OBJH,
        NCTYPE_STATE,
        NCTYPE_DURATION,
        NCTYPE_STATE_P,
    ) -> NCTYPE_STATUS,
    pub(crate) ncStatusToString:
        unsafe extern "system" fn(NCTYPE_STATUS, NCTYPE_UINT32, NCTYPE_STRING) -> NCTYPE_STATUS,
}

unsafe impl Send for NiCan {}
unsafe impl Sync for NiCan {}

impl NiCan {
    pub fn new(dll_path: Option<&str>) -> Result<Self, CanError> {
        let dll_path = dll_path.unwrap_or(r"Nican.dll");
        let dll_path = PCSTR::from_raw(dll_path.as_ptr());
        unsafe {
            let dll =
                GetModuleHandleA(dll_path).map_err(|e| CanError::InitializeError(e.to_string()))?;

            Ok(Self {
                _dll: dll,
                channels: Default::default(),
                ncConfig: std::mem::transmute(GetProcAddress(
                    dll,
                    PCSTR::from_raw(b"ncConfig\0".as_ptr()),
                )),
                ncOpenObject: std::mem::transmute(GetProcAddress(
                    dll,
                    PCSTR::from_raw(b"ncOpenObject\0".as_ptr()),
                )),
                ncAction: std::mem::transmute(GetProcAddress(
                    dll,
                    PCSTR::from_raw(b"ncAction\0".as_ptr()),
                )),
                ncCloseObject: std::mem::transmute(GetProcAddress(
                    dll,
                    PCSTR::from_raw(b"ncCloseObject\0".as_ptr()),
                )),
                ncWrite: std::mem::transmute(GetProcAddress(
                    dll,
                    PCSTR::from_raw(b"ncWrite\0".as_ptr()),
                )),
                ncRead: std::mem::transmute(GetProcAddress(
                    dll,
                    PCSTR::from_raw(b"ncRead\0".as_ptr()),
                )),
                ncWaitForState: std::mem::transmute(GetProcAddress(
                    dll,
                    PCSTR::from_raw(b"ncWaitForState\0".as_ptr()),
                )),
                ncStatusToString: std::mem::transmute(GetProcAddress(
                    dll,
                    PCSTR::from_raw(b"ncStatusToString\0".as_ptr()),
                )),
            })
        }
    }

    pub fn open(
        &mut self,
        channel: &str,
        filters: Vec<CanFilter>,
        bitrate: u32,
        log_errors: bool,
    ) -> Result<(), CanError> {
        let mut attr_id = vec![NC_ATTR_START_ON_OPEN, NC_ATTR_LOG_COMM_ERRS];
        let mut attr_val = vec![1, if log_errors { 1 } else { 0 }];

        match filters.len() {
            0 => {
                attr_id.extend([
                    NC_ATTR_CAN_COMP_STD,
                    NC_ATTR_CAN_MASK_STD,
                    NC_ATTR_CAN_COMP_XTD,
                    NC_ATTR_CAN_MASK_XTD,
                ]);
                attr_val.extend([0; 4])
            }
            _ => filters.iter().for_each(|f| {
                attr_id.extend([NC_ATTR_CAN_COMP_XTD, NC_ATTR_CAN_MASK_XTD]);
                if f.extended {
                    attr_val.extend([f.can_id | NC_FL_CAN_ARBID_XTD, f.can_mask]);
                } else {
                    attr_val.extend([f.can_id, f.can_mask]);
                }
            }),
        }

        attr_id.push(NC_ATTR_BAUD_RATE);
        attr_val.push(bitrate);

        let chl_ascii = CString::new(channel).map_err(|e| CanError::OtherError(e.to_string()))?;
        let ret = unsafe {
            (self.ncConfig)(
                chl_ascii.clone().into_raw(),
                attr_id.len() as NCTYPE_UINT32,
                attr_id.as_mut_ptr() as NCTYPE_ATTRID_P,
                attr_val.as_mut_ptr() as NCTYPE_UINT32_P,
            )
        };
        if ret != 0 {
            return Err(CanError::InitializeError(
                "device configration error".into(),
            ));
        }

        let mut handle = 0;
        let ret = unsafe { (self.ncOpenObject)(chl_ascii.into_raw(), &mut handle) };
        if ret != 0 {
            return Err(CanError::InitializeError("device open error".into()));
        }

        self.channels.insert(
            channel.into(),
            NiCanContext {
                handle,
                filters,
                bitrate,
                log_errors,
            },
        );

        Ok(())
    }

    pub fn reset(&mut self, channel: String) -> Result<(), CanError> {
        match self.channels.get(&channel) {
            Some(ctx) => {
                let ret = unsafe { (self.ncAction)(ctx.handle, NC_OP_RESET as NCTYPE_OPCODE, 0) };

                self.check_status(channel.as_str(), ret).map_err(|r| {
                    let info = format!(
                        "{} error {} when reset",
                        Self::channel_info(&channel),
                        self.status_to_str(r)
                    );
                    rsutil::warn!("{}", info);

                    CanError::OperationError(info)
                })
            }
            None => Err(CanError::channel_not_opened(Self::channel_info(&channel))),
        }
    }

    pub fn close(&mut self, channel: String) -> Result<(), CanError> {
        match self.channels.get(&channel) {
            Some(ctx) => {
                let ret = unsafe { (self.ncCloseObject)(ctx.handle) };
                self.channels.remove(&channel);

                self.check_status(channel.as_str(), ret).map_err(|r| {
                    let info = format!(
                        "{} error {} when close",
                        Self::channel_info(&channel),
                        self.status_to_str(r)
                    );
                    rsutil::warn!("{}", info);

                    CanError::OperationError(info)
                })
            }
            None => Err(CanError::channel_not_opened(Self::channel_info(&channel))),
        }
    }

    pub fn transmit_can(&self, msg: CanMessage) -> Result<(), CanError> {
        let channel = msg.channel();
        match self.channels.get(&channel) {
            Some(ctx) => {
                let raw_msg = msg.into();

                let ret = unsafe {
                    (self.ncWrite)(
                        ctx.handle,
                        std::mem::size_of::<NCTYPE_CAN_FRAME>() as NCTYPE_UINT32,
                        &raw_msg as *const NCTYPE_CAN_FRAME as NCTYPE_ANY_P,
                    )
                };

                if let Err(r) = self.check_status(channel.as_str(), ret) {
                    rsutil::warn!(
                        "{} error {} when transmit",
                        Self::channel_info(&channel),
                        self.status_to_str(r)
                    )
                }

                Ok(())
            }
            None => Err(CanError::channel_not_opened(Self::channel_info(&channel))),
        }
    }

    pub fn receive_can(
        &self,
        channel: String,
        timeout: Option<u32>,
    ) -> Result<Vec<CanMessage>, CanError> {
        match self.channels.get(&channel) {
            Some(ctx) => {
                if let Err(ret) = self.wait_for_state(channel.as_str(), ctx.handle, timeout) {
                    let info = format!("{} wait for state timeout", Self::channel_info(&channel));
                    if ret == constant::CanErrFunctionTimeout as NCTYPE_STATUS {
                        rsutil::warn!("{}", info);
                    }
                    return Err(CanError::channel_timeout(Self::channel_info(&channel)));
                }

                let raw_msg = NCTYPE_CAN_STRUCT {
                    Timestamp: NCTYPE_UINT64 {
                        LowPart: Default::default(),
                        HighPart: Default::default(),
                    },
                    ArbitrationId: Default::default(),
                    FrameType: Default::default(),
                    DataLength: Default::default(),
                    Data: Default::default(),
                };

                let ret = unsafe {
                    (self.ncRead)(
                        ctx.handle,
                        std::mem::size_of::<NCTYPE_CAN_STRUCT>() as NCTYPE_UINT32,
                        &raw_msg as *const NCTYPE_CAN_STRUCT as NCTYPE_ANY_P,
                    )
                };

                if let Err(r) = self.check_status(channel.as_str(), ret) {
                    let info = format!(
                        "{} error {} when receive",
                        Self::channel_info(&channel),
                        self.status_to_str(r)
                    );
                    rsutil::warn!("{}", info);
                    return Err(CanError::OperationError(info));
                }

                let mut msg = <NCTYPE_CAN_STRUCT as TryInto<CanMessage>>::try_into(raw_msg)?;
                msg.set_channel(channel.clone());

                Ok(vec![msg])
            }
            None => Err(CanError::channel_not_opened(Self::channel_info(&channel))),
        }
    }

    #[inline]
    pub fn channel_info(channel: &str) -> String {
        format!("NI-CAN: {}", channel)
    }

    #[inline]
    pub fn filters(&self, channel: String) -> Result<Vec<CanFilter>, CanError> {
        self.channel_util(channel, |ctx| Ok(ctx.filters.clone()))
    }

    #[inline]
    pub fn bitrate(&self, channel: String) -> Result<u32, CanError> {
        self.channel_util(channel, |ctx| Ok(ctx.bitrate))
    }

    #[inline]
    pub fn is_log_errors(&self, channel: String) -> Result<bool, CanError> {
        self.channel_util(channel, |ctx| Ok(ctx.log_errors))
    }

    #[inline]
    fn channel_util<R>(
        &self,
        channel: String,
        cb: fn(ctx: &NiCanContext) -> Result<R, CanError>,
    ) -> Result<R, CanError> {
        match self.channels.get(&channel) {
            Some(ctx) => cb(ctx),
            None => Err(CanError::channel_not_opened(Self::channel_info(&channel))),
        }
    }

    fn wait_for_state(
        &self,
        channel: &str,
        handle: NCTYPE_OBJH,
        timeout: Option<u32>,
    ) -> Result<(), NCTYPE_STATUS> {
        let timeout = timeout.unwrap_or(NC_DURATION_INFINITE) as NCTYPE_DURATION;

        let mut state = 0;
        let ret = unsafe {
            (self.ncWaitForState)(
                handle,
                NC_ST_READ_AVAIL as NCTYPE_STATE,
                timeout,
                &mut state,
            )
        };

        self.check_status(channel, ret)
    }

    pub(crate) fn check_status(&self, channel: &str, result: NCTYPE_STATUS) -> Result<(), NCTYPE_STATUS> {
        if result > 0 {
            rsutil::warn!(
                "{} {}",
                Self::channel_info(channel),
                self.status_to_str(result)
            );
            Ok(())
        } else if result < 0 {
            Err(result)
        } else {
            Ok(())
        }
    }

    pub(crate) fn status_to_str(&self, code: NCTYPE_STATUS) -> String {
        let mut err = [0u8; 1024];
        unsafe {
            (self.ncStatusToString)(
                code,
                err.len() as NCTYPE_UINT32,
                err.as_mut_ptr() as NCTYPE_STRING,
            )
        };
        let cstr = unsafe { CStr::from_ptr(err.as_ptr() as *const c_char) };

        cstr.to_str().unwrap_or("Unknown").to_string()
    }
}

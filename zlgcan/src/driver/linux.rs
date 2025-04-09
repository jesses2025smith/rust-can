use std::{path::PathBuf, sync::Arc};
use dlopen2::symbor::Container;
use rs_can::{CanError, ChannelConfig};

use crate::can::{CanMessage, ZCanChlError, ZCanChlStatus, ZCanFrameType};
use crate::device::{DeriveInfo, Handler, ZCanDeviceType, ZChannelContext, ZDeviceContext, ZDeviceInfo};
use crate::lin::{ZLinChlCfg, ZLinFrame, ZLinPublish, ZLinSubscribe};
use crate::api::{USBCANApi, USBCANEApi, USBCANFDApi, USBCANFD800UApi, ZCanApi, ZDeviceApi, ZLinApi};
use crate::driver::{lin_support, ZDevice};
use crate::utils;

#[cfg(target_arch = "x86")]
const LIB_PATH: &str = "linux/x86/";
#[cfg(target_arch = "x86_64")]
const LIB_PATH: &str = "linux/x86_64/";

#[derive(Clone)]
pub struct ZCanDriver {
    pub(crate) libpath:           String,
    pub(crate) handler:           Option<Handler>,
    pub(crate) usbcan_api:        Arc<Container<USBCANApi<'static>>>,
    pub(crate) usbcan_4e_api:     Arc<Container<USBCANEApi<'static>>>,
    pub(crate) usbcan_8e_api:     Arc<Container<USBCANEApi<'static>>>,
    pub(crate) usbcanfd_api:      Arc<Container<USBCANFDApi<'static>>>,
    pub(crate) usbcanfd_800u_api: Arc<Container<USBCANFD800UApi<'static>>>,
    pub(crate) dev_type:          ZCanDeviceType,
    pub(crate) dev_idx:           u32,
    pub(crate) derive:            Option<DeriveInfo>,
}

impl ZDevice for ZCanDriver {
    fn new(libpath: String, dev_type: u32, dev_idx: u32, derive: Option<DeriveInfo>) -> Result<Self, CanError> {
        let dev_type = ZCanDeviceType::try_from(dev_type)?;
        let mut path = PathBuf::from(&libpath);
        path.push(LIB_PATH);
        Ok(Self {
            libpath,
            handler: Default::default(),
            usbcan_api: Arc::new(unsafe { Container::load(&utils::get_libpath(path.clone(), "libusbcan.so")) }
                .map_err(|e| CanError::InitializeError(e.to_string()))?),
            usbcan_4e_api: Arc::new(unsafe { Container::load(&utils::get_libpath(path.clone(), "libusbcan-4e.so")) }
                .map_err(|e| CanError::InitializeError(e.to_string()))?),
            usbcan_8e_api: Arc::new(unsafe { Container::load(&utils::get_libpath(path.clone(), "libusbcan-8e.so")) }
                .map_err(|e| CanError::InitializeError(e.to_string()))?),
            usbcanfd_api: Arc::new(unsafe { Container::load(&utils::get_libpath(path.clone(), "libusbcanfd.so")) }
                .map_err(|e| CanError::InitializeError(e.to_string()))?),
            usbcanfd_800u_api: Arc::new(unsafe { Container::load(&utils::get_libpath(path.clone(), "libusbcanfd800u.so")) }
                .map_err(|e| CanError::InitializeError(e.to_string()))?),
            dev_type,
            dev_idx,
            derive,
        })
    }

    fn device_type(&self) -> ZCanDeviceType {
        self.dev_type
    }

    fn device_index(&self) -> u32 {
        self.dev_idx
    }

    fn open(&mut self) -> Result<(), CanError> {
        let mut context = ZDeviceContext::new(self.dev_type, self.dev_idx, self.derive.is_some());
        let dev_info: ZDeviceInfo;
        match self.dev_type {
            ZCanDeviceType::ZCAN_USBCAN1
            | ZCanDeviceType::ZCAN_USBCAN2 => {
                self.usbcan_api.open(&mut context)?;
                match self.derive {
                    Some(v) => {
                        dev_info = ZDeviceInfo::try_from(&v)?;
                    },
                    None => dev_info = self.usbcan_api.read_device_info(&context)?,
                }
            },
            ZCanDeviceType::ZCAN_USBCAN_4E_U => {
                self.usbcan_4e_api.open(&mut context)?;
                dev_info = self.usbcan_4e_api.read_device_info(&context)?;
            },
            ZCanDeviceType::ZCAN_USBCAN_8E_U => {
                self.usbcan_8e_api.open(&mut context)?;
                dev_info = self.usbcan_8e_api.read_device_info(&context)?;
            },
            ZCanDeviceType::ZCAN_USBCANFD_MINI
            | ZCanDeviceType::ZCAN_USBCANFD_100U
            | ZCanDeviceType::ZCAN_USBCANFD_200U => {
                self.usbcanfd_api.open(&mut context)?;
                dev_info = self.usbcanfd_api.read_device_info(&context)?;
            },
            ZCanDeviceType::ZCAN_USBCANFD_800U => {
                self.usbcanfd_800u_api.open(&mut context)?;
                dev_info = self.usbcanfd_800u_api.read_device_info(&context)?;
            },
            _ => return Err(CanError::NotSupportedError),
        };
        self.handler = Some(Handler::new(context, dev_info));
        Ok(())
    }

    fn close(&mut self) {
        if let Some(dev_hdl) = &mut self.handler {
            let cans = dev_hdl.can_channels();
            let lins = dev_hdl.lin_channels();

            match self.dev_type {
                ZCanDeviceType::ZCAN_USBCAN1
                | ZCanDeviceType::ZCAN_USBCAN2 => {
                    for (idx, context) in cans {
                        log::info!("ZLGCAN - closing CAN channel: {}", *idx);
                        self.usbcan_api.reset_can_chl(context)
                            .unwrap_or_else(|e| log::warn!("{}", e));
                    }

                    self.usbcan_api.close(dev_hdl.device_context())
                        .unwrap_or_else(|e| log::warn!("{}", e));
                },
                ZCanDeviceType::ZCAN_USBCAN_4E_U => {
                    for (idx, context) in cans {
                        log::info!("ZLGCAN - closing CAN channel: {}", *idx);
                        self.usbcan_4e_api.reset_can_chl(context)
                            .unwrap_or_else(|e| log::warn!("{}", e));
                    }

                    self.usbcan_4e_api.close(dev_hdl.device_context())
                        .unwrap_or_else(|e| log::warn!("{}", e));
                },
                ZCanDeviceType::ZCAN_USBCAN_8E_U => {
                    for (idx, context) in cans {
                        log::info!("ZLGCAN - closing CAN channel: {}", *idx);
                        self.usbcan_8e_api.reset_can_chl(context)
                            .unwrap_or_else(|e| log::warn!("{}", e));
                    }
                    self.usbcan_8e_api.close(dev_hdl.device_context())
                        .unwrap_or_else(|e| log::warn!("{}", e));
                },
                ZCanDeviceType::ZCAN_USBCANFD_MINI
                | ZCanDeviceType::ZCAN_USBCANFD_100U
                | ZCanDeviceType::ZCAN_USBCANFD_200U => {
                    for (idx, context) in cans {
                        log::info!("ZLGCAN - closing CAN channel: {}", *idx);
                        self.usbcanfd_api.reset_can_chl(context)
                            .unwrap_or_else(|e| log::warn!("{}", e));
                    }

                    for (idx, context) in lins {
                        log::info!("ZLGCAN - closing LIN channel: {}", *idx);
                        self.usbcanfd_api.reset_lin_chl(context)
                            .unwrap_or_else(|e| log::warn!("{}", e));
                    }

                    self.usbcanfd_api.close(dev_hdl.device_context())
                        .unwrap_or_else(|e| log::warn!("{}", e))
                },
                ZCanDeviceType::ZCAN_USBCANFD_800U => {
                    for (idx, context) in cans {
                        log::info!("ZLGCAN - closing CAN channel: {}", *idx);
                        self.usbcanfd_800u_api.reset_can_chl(context)
                            .unwrap_or_else(|e| log::warn!("{}", e));
                    }

                    self.usbcanfd_800u_api.close(dev_hdl.device_context())
                        .unwrap_or_else(|e| log::warn!("{}", e));
                },
                _ => log::warn!("{:?}", CanError::NotSupportedError),
            }
            self.handler = None;
        }
    }

    fn device_info(&self) -> Result<&ZDeviceInfo, CanError> {
        match &self.handler {
            Some(v) => Ok(v.device_info()),
            None => Err(CanError::device_not_opened()),
        }
    }

    fn is_derive_device(&self) -> bool {
        self.derive.is_some()
    }

    fn init_can_chl(&mut self, channel: u8, cfg: &ChannelConfig) -> Result<(), CanError> {
        match &mut self.handler {
            Some(dev_hdl) => {
                let dev_info = dev_hdl.device_info();
                let channels = dev_info.can_channels();
                if channel >= channels {
                    return Err(CanError::OtherError(format!("CAN channel: {} out of length!", channel)));
                }

                if self.dev_type == ZCanDeviceType::ZCAN_USBCAN_4E_U {
                    return self.usbcan_4e_api.init_can_chl_ex(&self.libpath, dev_hdl, channels, &cfg);
                }

                let mut context = ZChannelContext::new(dev_hdl.device_context().clone(), channel);
                match self.dev_type {
                    ZCanDeviceType::ZCAN_USBCAN1
                    | ZCanDeviceType::ZCAN_USBCAN2 => {
                        if let Some(context) = dev_hdl.find_can(channel) {
                            self.usbcan_api.reset_can_chl(context).unwrap_or_else(|e| log::warn!("{}", e));
                            dev_hdl.remove_can(channel);
                        }
                        self.usbcan_api.init_can_chl(&self.libpath, &mut context, &cfg)?;
                    },
                    // ZCanDeviceType::ZCAN_USBCAN_4E_U => {
                    //     if let Some(chl_hdl) = dev_hdl.find_can(idx) {
                    //         self.usbcan_4e_api.reset_can_chl(chl_hdl).unwrap_or_else(|e| log::warn!("{}", e));
                    //         dev_hdl.remove_can(idx);
                    //     }
                    //     chl_hdl = self.usbcan_4e_api.init_can_chl(dev_hdl.device_handler(), idx, cfg)?;
                    // },
                    ZCanDeviceType::ZCAN_USBCAN_8E_U => {
                        if let Some(chl_hdl) = dev_hdl.find_can(channel) {
                            self.usbcan_8e_api.reset_can_chl(chl_hdl).unwrap_or_else(|e| log::warn!("{}", e));
                            dev_hdl.remove_can(channel);
                        }
                        self.usbcan_8e_api.init_can_chl(&self.libpath, &mut context, &cfg)?;
                    },
                    ZCanDeviceType::ZCAN_USBCANFD_MINI
                    | ZCanDeviceType::ZCAN_USBCANFD_100U
                    | ZCanDeviceType::ZCAN_USBCANFD_200U => {
                        if let Some(context) = dev_hdl.find_can(channel) {
                            self.usbcanfd_api.reset_can_chl(context)?;
                            dev_hdl.remove_can(channel);
                        }
                        self.usbcanfd_api.init_can_chl(&self.libpath, &mut context, &cfg)?;
                    },
                    ZCanDeviceType::ZCAN_USBCANFD_800U => {
                        if let Some(chl_hdl) = dev_hdl.find_can(channel) {
                            self.usbcanfd_800u_api.reset_can_chl(chl_hdl).unwrap_or_else(|e| log::warn!("{}", e));
                            dev_hdl.remove_can(channel);
                        }
                        self.usbcanfd_800u_api.init_can_chl_ex(self.dev_type, self.dev_idx, channel, &cfg)?;
                        self.usbcanfd_800u_api.init_can_chl(&self.libpath, &mut context, &cfg)?;
                    },
                    _ => return Err(CanError::NotSupportedError),
                }

                dev_hdl.add_can(channel, context);
                Ok(())
            },
            None => Err(CanError::device_not_opened()),
        }
    }

    fn reset_can_chl(&mut self, channel: u8) -> Result<(), CanError> {
        match &mut self.handler {
            Some(dev_hdl) => {
                match dev_hdl.find_can(channel) {
                    Some(context) => {
                        match self.dev_type {
                            ZCanDeviceType::ZCAN_USBCAN1
                            | ZCanDeviceType::ZCAN_USBCAN2 => {
                                self.usbcan_api.reset_can_chl(context)?;
                            },
                            ZCanDeviceType::ZCAN_USBCAN_4E_U => {
                                self.usbcan_4e_api.reset_can_chl(context)?;
                            },
                            ZCanDeviceType::ZCAN_USBCAN_8E_U => {
                                self.usbcan_8e_api.reset_can_chl(context)?;
                            },
                            ZCanDeviceType::ZCAN_USBCANFD_MINI
                            | ZCanDeviceType::ZCAN_USBCANFD_100U
                            | ZCanDeviceType::ZCAN_USBCANFD_200U => {
                                self.usbcanfd_api.reset_can_chl(context)?;
                            },
                            ZCanDeviceType::ZCAN_USBCANFD_800U => {
                                self.usbcanfd_800u_api.reset_can_chl(context)?;
                            },
                            _ => return Err(CanError::NotSupportedError),
                        }
                        dev_hdl.remove_can(channel);
                        Ok(())
                    },
                    None => Err(CanError::channel_not_opened(channel)),
                }
            },
            None => Err(CanError::device_not_opened()),
        }
    }

    fn read_can_chl_status(&self, channel: u8) -> Result<ZCanChlStatus, CanError> {
        match self.dev_type {
            ZCanDeviceType::ZCAN_USBCAN1
            | ZCanDeviceType::ZCAN_USBCAN2 => {
                self.can_handler(channel, |context| {
                    self.usbcan_api.read_can_chl_status(context)
                })
            },
            ZCanDeviceType::ZCAN_USBCAN_4E_U => {
                self.can_handler(channel, |context| {
                    self.usbcan_4e_api.read_can_chl_status(context)
                })
            },
            ZCanDeviceType::ZCAN_USBCAN_8E_U => {
                self.can_handler(channel, |context| {
                    self.usbcan_8e_api.read_can_chl_status(context)
                })
            },
            ZCanDeviceType::ZCAN_USBCANFD_MINI
            | ZCanDeviceType::ZCAN_USBCANFD_100U
            | ZCanDeviceType::ZCAN_USBCANFD_200U => {
                self.can_handler(channel, |context| {
                    self.usbcanfd_api.read_can_chl_status(context)
                })
            },
            ZCanDeviceType::ZCAN_USBCANFD_800U => {
                self.can_handler(channel, |chl_hdl| {
                    self.usbcanfd_800u_api.read_can_chl_status(chl_hdl)
                })
            },
            _ => Err(CanError::NotSupportedError),
        }
    }

    fn read_can_chl_error(&self, channel: u8) -> Result<ZCanChlError, CanError> {
        match self.dev_type {
            ZCanDeviceType::ZCAN_USBCAN1
            | ZCanDeviceType::ZCAN_USBCAN2 => {
                self.can_handler(channel, |context| {
                    self.usbcan_api.read_can_chl_error(context)
                })
            },
            ZCanDeviceType::ZCAN_USBCAN_4E_U => {
                self.can_handler(channel, |context| {
                    self.usbcan_4e_api.read_can_chl_error(context)
                })
            },
            ZCanDeviceType::ZCAN_USBCAN_8E_U => {
                self.can_handler(channel, |context| {
                    self.usbcan_8e_api.read_can_chl_error(context)
                })
            },
            ZCanDeviceType::ZCAN_USBCANFD_MINI
            | ZCanDeviceType::ZCAN_USBCANFD_100U
            | ZCanDeviceType::ZCAN_USBCANFD_200U => {
                self.can_handler(channel, |context| {
                    self.usbcanfd_api.read_can_chl_error(context)
                })
            },
            ZCanDeviceType::ZCAN_USBCANFD_800U => {
                self.can_handler(channel, |context| {
                    self.usbcanfd_800u_api.read_can_chl_error(context)
                })
            },
            _ => Err(CanError::device_not_opened()),
        }
    }

    fn clear_can_buffer(&self, channel: u8) -> Result<(), CanError> {
        match self.dev_type {
            ZCanDeviceType::ZCAN_USBCAN1
            | ZCanDeviceType::ZCAN_USBCAN2 => {
                self.can_handler(channel, |context| {
                    self.usbcan_api.clear_can_buffer(context)
                })
            },
            ZCanDeviceType::ZCAN_USBCAN_4E_U => {
                self.can_handler(channel, |context| {
                    self.usbcan_4e_api.clear_can_buffer(context)
                })
            },
            ZCanDeviceType::ZCAN_USBCAN_8E_U => {
                self.can_handler(channel, |context| {
                    self.usbcan_8e_api.clear_can_buffer(context)
                })
            },
            ZCanDeviceType::ZCAN_USBCANFD_MINI
            | ZCanDeviceType::ZCAN_USBCANFD_100U
            | ZCanDeviceType::ZCAN_USBCANFD_200U => {
                self.can_handler(channel, |context| {
                    self.usbcanfd_api.clear_can_buffer(context)
                })
            },
            ZCanDeviceType::ZCAN_USBCANFD_800U => {
                self.can_handler(channel, |context| {
                    self.usbcanfd_800u_api.clear_can_buffer(context)
                })
            },
            _ => Err(CanError::NotSupportedError),
        }
    }

    fn get_can_num(&self, channel: u8, can_type: ZCanFrameType) -> Result<u32, CanError> {
        match self.dev_type {
            ZCanDeviceType::ZCAN_USBCAN1
            | ZCanDeviceType::ZCAN_USBCAN2 => {
                self.can_handler(channel, |context| {
                    self.usbcan_api.get_can_num(context, can_type)
                })
            },
            ZCanDeviceType::ZCAN_USBCAN_4E_U => {
                self.can_handler(channel, |context| {
                    self.usbcan_4e_api.get_can_num(context, can_type)
                })
            },
            ZCanDeviceType::ZCAN_USBCAN_8E_U => {
                self.can_handler(channel, |context| {
                    self.usbcan_8e_api.get_can_num(context, can_type)
                })
            },
            ZCanDeviceType::ZCAN_USBCANFD_MINI
            | ZCanDeviceType::ZCAN_USBCANFD_100U
            | ZCanDeviceType::ZCAN_USBCANFD_200U => {
                self.can_handler(channel, |context| {
                    self.usbcanfd_api.get_can_num(context, can_type)
                })
            },
            ZCanDeviceType::ZCAN_USBCANFD_800U => {
                self.can_handler(channel, |context| {
                    self.usbcanfd_800u_api.get_can_num(context, can_type)
                })
            },
            _ => Err(CanError::device_not_opened()),
        }
    }

    fn receive_can(&self, channel: u8, size: u32, timeout: Option<u32>) -> Result<Vec<CanMessage>, CanError> {
        let timeout = timeout.unwrap_or(u32::MAX);
        match self.dev_type {
            ZCanDeviceType::ZCAN_USBCAN1
            | ZCanDeviceType::ZCAN_USBCAN2 => {
                self.can_handler(channel, |context| {
                    self.usbcan_api.receive_can(context, size, timeout)
                })
            },
            ZCanDeviceType::ZCAN_USBCAN_4E_U => {
                self.can_handler(channel, |context| {
                    self.usbcan_4e_api.receive_can(context, size, timeout)
                })
            },
            ZCanDeviceType::ZCAN_USBCAN_8E_U => {
                self.can_handler(channel, |context| {
                    self.usbcan_8e_api.receive_can(context, size, timeout)
                })
            },
            ZCanDeviceType::ZCAN_USBCANFD_MINI
            | ZCanDeviceType::ZCAN_USBCANFD_100U
            | ZCanDeviceType::ZCAN_USBCANFD_200U => {
                self.can_handler(channel, |context| {
                    self.usbcanfd_api.receive_can(context, size, timeout)
                })
            },
            ZCanDeviceType::ZCAN_USBCANFD_800U => {
                self.can_handler(channel, |context| {
                    self.usbcanfd_800u_api.receive_can(context, size, timeout)
                })
            },
            _ => Err(CanError::NotSupportedError),
        }
    }

    fn transmit_can(&self, channel: u8, frames: Vec<CanMessage>) -> Result<u32, CanError> {
        match self.dev_type {
            ZCanDeviceType::ZCAN_USBCAN1
            | ZCanDeviceType::ZCAN_USBCAN2 => {
                self.can_handler(channel, |context| {
                    self.usbcan_api.transmit_can(context, frames)
                })
            },
            ZCanDeviceType::ZCAN_USBCAN_4E_U => {
                self.can_handler(channel, |context| {
                    self.usbcan_4e_api.transmit_can(context, frames)
                })
            },
            ZCanDeviceType::ZCAN_USBCAN_8E_U => {
                self.can_handler(channel, |context| {
                    self.usbcan_8e_api.transmit_can(context, frames)
                })
            },
            ZCanDeviceType::ZCAN_USBCANFD_MINI
            | ZCanDeviceType::ZCAN_USBCANFD_100U
            | ZCanDeviceType::ZCAN_USBCANFD_200U => {
                self.can_handler(channel, |context| {
                    self.usbcanfd_api.transmit_can(context, frames)
                })
            },
            ZCanDeviceType::ZCAN_USBCANFD_800U => {
                self.can_handler(channel, |context| {
                    self.usbcanfd_800u_api.transmit_can(context, frames)
                })
            },
            _ => Err(CanError::NotSupportedError),
        }
    }

    fn receive_canfd(&self, channel: u8, size: u32, timeout: Option<u32>) -> Result<Vec<CanMessage>, CanError> {
        let timeout = timeout.unwrap_or(u32::MAX);
        match self.dev_type {
            ZCanDeviceType::ZCAN_USBCANFD_MINI
            | ZCanDeviceType::ZCAN_USBCANFD_100U
            | ZCanDeviceType::ZCAN_USBCANFD_200U => {
                self.can_handler(channel, |context| {
                    self.usbcanfd_api.receive_canfd(context, size, timeout)
                })
            },
            ZCanDeviceType::ZCAN_USBCANFD_800U => {
                self.can_handler(channel, |context| {
                    self.usbcanfd_800u_api.receive_canfd(context, size, timeout)
                })
            },
            _ => Err(CanError::NotSupportedError),
        }
    }

    fn transmit_canfd(&self, channel: u8, frames: Vec<CanMessage>) -> Result<u32, CanError> {
        match self.dev_type {
            ZCanDeviceType::ZCAN_USBCANFD_MINI | ZCanDeviceType::ZCAN_USBCANFD_100U | ZCanDeviceType::ZCAN_USBCANFD_200U => {
                self.can_handler(channel, |context| {
                    self.usbcanfd_api.transmit_canfd(context, frames)
                })
            },
            ZCanDeviceType::ZCAN_USBCANFD_800U => {
                self.can_handler(channel, |context| {
                    self.usbcanfd_800u_api.transmit_canfd(context, frames)
                })
            },
            _ => Err(CanError::NotSupportedError),
        }
    }

    fn init_lin_chl(&mut self, channel: u8, cfg: ZLinChlCfg) -> Result<(), CanError> {
        lin_support(self.dev_type)?;
        match &mut self.handler {
            Some(dev_hdl) => {
                let channels = 2;   //dev_info.lin_channels();  // TODO
                if channel >= channels {
                    return Err(CanError::OtherError(format!("LIN channel: {} is out of channels!", channel)));
                }

                let mut context = ZChannelContext::new(dev_hdl.device_context().clone(), channel);
                match self.dev_type {
                    ZCanDeviceType::ZCAN_USBCANFD_200U => {
                        if let Some(context) = dev_hdl.find_lin(channel) {
                            self.usbcanfd_api.reset_lin_chl(context)?;
                            dev_hdl.remove_lin(channel);
                        }

                        self.usbcanfd_api.init_lin_chl(&mut context, &cfg)?;
                    },
                    _ => return Err(CanError::NotSupportedError),
                }

                dev_hdl.add_lin(channel, context);

                Ok(())
            },
            None => Err(CanError::device_not_opened()),
        }
    }

    fn reset_lin_chl(&mut self, channel: u8) -> Result<(), CanError> {
        lin_support(self.dev_type)?;
        match &mut self.handler {
            Some(dev_hdl) => {
                match dev_hdl.find_lin(channel) {
                    Some(context) => {
                        match self.dev_type {
                            ZCanDeviceType::ZCAN_USBCANFD_200U => {
                                self.usbcanfd_api.reset_lin_chl(context)
                            },
                            _ => Err(CanError::NotSupportedError),
                        }
                    },
                    None => Err(CanError::channel_not_opened(channel)),
                }
            },
            None => Err(CanError::device_not_opened()),
        }
    }

    fn clear_lin_buffer(&self, channel: u8) -> Result<(), CanError> {
        match self.dev_type {
            ZCanDeviceType::ZCAN_USBCANFD_200U => {
                self.lin_handler(channel, |context| {
                    self.usbcanfd_api.clear_lin_buffer(context)
                })
            },
            _ => Err(CanError::NotSupportedError),
        }
    }

    fn get_lin_num(&self, channel: u8) -> Result<u32, CanError> {
        match self.dev_type {
            ZCanDeviceType::ZCAN_USBCANFD_200U => {
                self.lin_handler(channel, |context| {
                    self.usbcanfd_api.get_lin_num(context)
                })
            },
            _ => Err(CanError::NotSupportedError),
        }
    }

    fn receive_lin(&self, channel: u8, size: u32, timeout: Option<u32>) -> Result<Vec<ZLinFrame>, CanError> {
        let timeout = timeout.unwrap_or(u32::MAX);
        match self.dev_type {
            ZCanDeviceType::ZCAN_USBCANFD_200U => {
                self.lin_handler(channel, |context| {
                    self.usbcanfd_api.receive_lin(context, size, timeout)
                })
            },
            _ => Err(CanError::NotSupportedError),
        }
    }

    fn transmit_lin(&self, channel: u8, frames: Vec<ZLinFrame>) -> Result<u32, CanError> {
        match self.dev_type {
            ZCanDeviceType::ZCAN_USBCANFD_200U => {
                self.lin_handler(channel, |context| {
                    self.usbcanfd_api.transmit_lin(context, frames)
                })
            },
            _ => Err(CanError::NotSupportedError),
        }
    }

    fn set_lin_subscribe(&self, channel: u8, cfg: Vec<ZLinSubscribe>) -> Result<(), CanError> {
        match self.dev_type {
            ZCanDeviceType::ZCAN_USBCANFD_200U => {
                self.lin_handler(channel, |context| {
                    self.usbcanfd_api.set_lin_subscribe(context, cfg)
                })
            },
            _ => Err(CanError::NotSupportedError),
        }
    }

    fn set_lin_publish(&self, channel: u8, cfg: Vec<ZLinPublish>) -> Result<(), CanError> {
        match self.dev_type {
            ZCanDeviceType::ZCAN_USBCANFD_200U => {
                self.lin_handler(channel, |context| {
                    self.usbcanfd_api.set_lin_publish(context, cfg)
                })
            },
            _ => Err(CanError::NotSupportedError),
        }
    }

    fn wakeup_lin(&self, channel: u8) -> Result<(), CanError> {
        match self.dev_type {
            ZCanDeviceType::ZCAN_USBCANFD_200U => {
                self.lin_handler(channel, |context| {
                    self.usbcanfd_api.wakeup_lin(context)
                })
            },
            _ => Err(CanError::NotSupportedError),
        }
    }

    #[allow(deprecated)]
    fn set_lin_slave_msg(&self, channel: u8, msg: Vec<ZLinFrame>) -> Result<(), CanError> {
        match self.dev_type {
            ZCanDeviceType::ZCAN_USBCANFD_200U => {
                self.lin_handler(channel, |context| {
                    self.usbcanfd_api.set_lin_slave_msg(context, msg)
                })
            },
            _ => Err(CanError::NotSupportedError),
        }
    }

    #[allow(deprecated)]
    fn clear_lin_slave_msg(&self, channel: u8, pids: Vec<u8>) -> Result<(), CanError> {
        match self.dev_type {
            ZCanDeviceType::ZCAN_USBCANFD_200U => {
                self.lin_handler(channel, |context| {
                    self.usbcanfd_api.clear_lin_slave_msg(context, pids)
                })
            },
            _ => Err(CanError::NotSupportedError),
        }
    }

    #[inline]
    fn timestamp(&self, channel: u8) -> Result<u64, CanError> {
        self.can_handler(channel, |context| Ok(context.timestamp()))
    }

    fn device_handler<C, T>(&self, callback: C) -> Result<T, CanError>
        where
            C: FnOnce(&Handler) -> Result<T, CanError> {
        match &self.handler {
            Some(v) => callback(v),
            None => Err(CanError::device_not_opened()),
        }
    }
}


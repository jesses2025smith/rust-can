/// `typedef.rs` defined the zlgcan device type and some function supported feature.
// use rs_can::CanError;

#[allow(non_camel_case_types, dead_code)]
#[repr(C)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ZCanDeviceType {
    Undefined                          = 0,
    ZCAN_PCI5121                       = 1,
    ZCAN_PCI9810                       = 2,
    ZCAN_USBCAN1                       = 3,
    ZCAN_USBCAN2                       = 4,
    ZCAN_PCI9820                       = 5,
    ZCAN_CAN232                        = 6,
    ZCAN_PCI5110                       = 7,
    ZCAN_CANLITE                       = 8,
    ZCAN_ISA9620                       = 9,
    ZCAN_ISA5420                       = 10,
    ZCAN_PC104CAN                      = 11,
    ZCAN_CANETUDP                      = 12,
    // ZCAN_CANETE                        = 12,
    ZCAN_DNP9810                       = 13,
    ZCAN_PCI9840                       = 14,
    ZCAN_PC104CAN2                     = 15,
    ZCAN_PCI9820I                      = 16,
    ZCAN_CANETTCP                      = 17,
    ZCAN_PCIE_9220                     = 18,
    ZCAN_PCI5010U                      = 19,
    ZCAN_USBCAN_E_U                    = 20,
    ZCAN_USBCAN_2E_U                   = 21,
    ZCAN_PCI5020U                      = 22,
    ZCAN_EG20T_CAN                     = 23,
    ZCAN_PCIE9221                      = 24,
    ZCAN_WIFICAN_TCP                   = 25,
    ZCAN_WIFICAN_UDP                   = 26,
    ZCAN_PCIe9120                      = 27,
    ZCAN_PCIe9110                      = 28,
    ZCAN_PCIe9140                      = 29,
    ZCAN_USBCAN_4E_U                   = 31,
    ZCAN_CANDTU_200UR                  = 32,
    ZCAN_CANDTU_MINI                   = 33,
    ZCAN_USBCAN_8E_U                   = 34,
    ZCAN_CANREPLAY                     = 35,
    ZCAN_CANDTU_NET                    = 36,
    ZCAN_CANDTU_100UR                  = 37,
    ZCAN_PCIE_CANFD_100U               = 38,
    ZCAN_PCIE_CANFD_200U               = 39,
    ZCAN_PCIE_CANFD_400U               = 40,
    ZCAN_USBCANFD_200U                 = 41,
    ZCAN_USBCANFD_100U                 = 42,
    ZCAN_USBCANFD_MINI                 = 43,
    ZCAN_CANFDCOM_100IE                = 44,
    ZCAN_CANSCOPE                      = 45,
    ZCAN_CLOUD                         = 46,
    ZCAN_CANDTU_NET_400                = 47,
    // ZCAN_CANFDNET_TCP                  = 48,
    ZCAN_CANFDNET_200U_TCP             = 48,
    // ZCAN_CANFDNET_UDP                  = 49,
    ZCAN_CANFDNET_200U_UDP             = 49,
    // ZCAN_CANFDWIFI_TCP                 = 50,
    ZCAN_CANFDWIFI_100U_TCP            = 50,
    // ZCAN_CANFDWIFI_UDP                 = 51,
    ZCAN_CANFDWIFI_100U_UDP            = 51,
    ZCAN_CANFDNET_400U_TCP             = 52,
    ZCAN_CANFDNET_400U_UDP             = 53,
    ZCAN_CANFDBLUE_200U                = 54,
    ZCAN_CANFDNET_100U_TCP             = 55,
    ZCAN_CANFDNET_100U_UDP             = 56,
    ZCAN_CANFDNET_800U_TCP             = 57,
    ZCAN_CANFDNET_800U_UDP             = 58,
    ZCAN_USBCANFD_800U                 = 59,
    ZCAN_PCIE_CANFD_100U_EX            = 60,
    ZCAN_PCIE_CANFD_400U_EX            = 61,
    ZCAN_PCIE_CANFD_200U_MINI          = 62,
    ZCAN_PCIE_CANFD_200U_M2            = 63,
    ZCAN_CANFDDTU_400_TCP              = 64,
    ZCAN_CANFDDTU_400_UDP              = 65,
    ZCAN_CANFDWIFI_200U_TCP            = 66,
    ZCAN_CANFDWIFI_200U_UDP            = 67,
    ZCAN_CANFDDTU_800ER_TCP            = 68,
    ZCAN_CANFDDTU_800ER_UDP            = 69,
    ZCAN_CANFDDTU_800EWGR_TCP          = 70,
    ZCAN_CANFDDTU_800EWGR_UDP          = 71,
    ZCAN_CANFDDTU_600EWGR_TCP          = 72,
    ZCAN_CANFDDTU_600EWGR_UDP          = 73,
    ZCAN_CANFDDTU_CASCADE_TCP          = 74,
    ZCAN_CANFDDTU_CASCADE_UDP          = 75,
    ZCAN_USBCANFD_400U                 = 76,
    ZCAN_CANFDDTU_200U                 = 77,
    ZCAN_ZPSCANFD_TCP                  = 78,
    ZCAN_ZPSCANFD_USB                  = 79,
    ZCAN_CANFDBRIDGE_PLUS              = 80,
    ZCAN_CANFDDTU_300U                 = 81,
    ZCAN_PCIE_CANFD_800U               = 82,
    ZCAN_PCIE_CANFD_1200U              = 83,
    ZCAN_MINI_PCIE_CANFD               = 84,
    ZCAN_USBCANFD_800H                 = 85,

    ZCAN_OFFLINE_DEVICE                = 98,
    ZCAN_VIRTUAL_DEVICE                = 99,
}

impl ZCanDeviceType {
    /// Check the device can use fd frame
    pub fn canfd_support(&self) -> bool {
        matches!(
            self,
            Self::ZCAN_CANFDBLUE_200U |
            Self::ZCAN_CANFDCOM_100IE |
            Self::ZCAN_CANFDDTU_400_TCP | Self::ZCAN_CANFDDTU_400_UDP |
            Self::ZCAN_CANFDDTU_600EWGR_TCP | Self::ZCAN_CANFDDTU_600EWGR_UDP |
            Self::ZCAN_CANFDDTU_800ER_TCP | Self::ZCAN_CANFDDTU_800ER_UDP |
            Self::ZCAN_CANFDDTU_800EWGR_TCP | Self::ZCAN_CANFDDTU_800EWGR_UDP |
            Self::ZCAN_CANFDNET_100U_TCP | Self::ZCAN_CANFDNET_100U_UDP |
            Self::ZCAN_CANFDNET_200U_TCP | Self::ZCAN_CANFDNET_200U_UDP |
            Self::ZCAN_CANFDNET_400U_TCP | Self::ZCAN_CANFDNET_400U_UDP |
            Self::ZCAN_CANFDNET_800U_TCP | Self::ZCAN_CANFDNET_800U_UDP |
            Self::ZCAN_CANFDWIFI_100U_TCP | Self::ZCAN_CANFDWIFI_100U_UDP |
            Self::ZCAN_CANFDWIFI_200U_TCP | Self::ZCAN_CANFDWIFI_200U_UDP |
            Self::ZCAN_PCIE_CANFD_100U | Self::ZCAN_PCIE_CANFD_100U_EX |
            Self::ZCAN_PCIE_CANFD_200U | Self::ZCAN_PCIE_CANFD_200U_MINI | Self::ZCAN_PCIE_CANFD_200U_M2 |
            Self::ZCAN_PCIE_CANFD_400U | Self::ZCAN_PCIE_CANFD_400U_EX |
            Self::ZCAN_USBCANFD_MINI | Self::ZCAN_USBCANFD_100U | Self::ZCAN_USBCANFD_200U | Self::ZCAN_USBCANFD_400U | Self::ZCAN_USBCANFD_800U
        )
    }
    /// Check the device is supported LIN
    pub const fn lin_support(&self) -> bool{
        matches!(
            self,
            Self::ZCAN_USBCANFD_200U | Self::ZCAN_USBCANFD_400U
        )
    }

    pub const fn has_resistance(&self) -> bool {
        !matches!{
            self,
            Self::ZCAN_USBCAN1 | Self::ZCAN_USBCAN2
        }
    }

    pub const fn cloud_support(&self) -> bool {
        matches!(
            self,
            Self::ZCAN_USBCANFD_800U
        )
    }

    pub const fn filter_record_support(&self) -> bool {
        matches!(
            self,
            Self::ZCAN_PCI5010U | Self::ZCAN_PCI5020U |
            Self::ZCAN_USBCAN_2E_U | Self::ZCAN_USBCAN_4E_U
        )
    }

    pub const fn auto_send_support(&self) -> bool {
        matches!(
            self,
            Self::ZCAN_USBCAN_2E_U | Self::ZCAN_USBCAN_4E_U | Self::ZCAN_USBCAN_8E_U
        )
    }
    /// set value then read and check the value if true
    /// TODO
    pub const fn get_value_support(&self) -> bool {
        true
    }
}

#[cfg(target_os = "linux")]
impl ZCanDeviceType {
    #[inline(always)]
    pub const fn is_usbcan(&self) -> bool {
        matches!(
            self,
            Self::ZCAN_USBCAN1 | Self::ZCAN_USBCAN2
        )
    }
    #[inline(always)]
    pub const fn is_usbcan_4e_u(&self) -> bool {
        matches!(
            self,
            Self::ZCAN_USBCAN_4E_U
        )
    }
    #[inline(always)]
    pub const fn is_usbcan_8e_u(&self) -> bool {
        matches!(
            self,
            Self::ZCAN_USBCAN_8E_U
        )
    }
    #[inline(always)]
    pub const fn is_usbcanfd(&self) -> bool {
        matches!(
            self,
            Self::ZCAN_USBCANFD_MINI
            | Self::ZCAN_USBCANFD_100U
            | Self::ZCAN_USBCANFD_200U
            | Self::ZCAN_USBCANFD_400U
        )
    }
    #[inline(always)]
    pub const fn is_usbcanfd_800u(&self) -> bool {
        matches!(
            self,
            Self::ZCAN_USBCANFD_800U
        )
    }
    #[inline(always)]
    pub const fn is_linux_support(&self) -> bool {
        self.is_usbcan() || self.is_usbcan_4e_u() || self.is_usbcan_8e_u()
            || self.is_usbcanfd() || self.is_usbcanfd_800u()
    }
}

impl From<ZCanDeviceType> for u32 {
    fn from(value: ZCanDeviceType) -> Self {
        value as u32
    }
}

impl TryFrom<u32> for ZCanDeviceType {
    /// Attention!!!
    /// This method is unsafe if the value is too large
    type Error = CanError;
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        let layout = std::alloc::Layout::new::<ZCanDeviceType>();
        unsafe {
            let vk: *mut ZCanDeviceType = std::alloc::alloc(layout).cast();
            if vk.is_null() {
                Err(CanError::other_error("allocate memory failed"))
            }
            else {
                let ptr = vk as *mut u32;
                *ptr = value;

                Ok(*vk)
            }
        }
    }
}

impl std::fmt::Display for ZCanDeviceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", *self)
    }
}

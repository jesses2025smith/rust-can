use crate::{
    constants,
    native::{
        can::{ZCanChlCfg, ZCanChlError, ZCanChlStatus, ZCanChlType, ZCanFrame},
        device::{CmdPath, IProperty, ZCanDeviceType, ZDeviceInfo},
    },
};
use dlopen2::symbor::{SymBorApi, Symbol};
use rs_can::{CanError, ChannelConfig};
use std::ffi::{c_uchar, c_uint, c_void};

#[rustfmt::skip]
#[allow(non_snake_case)]
#[derive(Debug, Clone, SymBorApi)]
pub(crate) struct USBCANFD800UApi<'a> {
    /// DEVICE_HANDLE FUNC_CALL ZCAN_OpenDevice(UINT device_type, UINT device_index, UINT reserved);
    pub(crate) ZCAN_OpenDevice: Symbol<'a, unsafe extern "C" fn(dev_type: c_uint, dev_index: c_uint, reserved: c_uint) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_CloseDevice(DEVICE_HANDLE device_handle);
    pub(crate) ZCAN_CloseDevice: Symbol<'a, unsafe extern "C" fn(dev_hdl: c_uint) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_GetDeviceInf(DEVICE_HANDLE device_handle, ZCAN_DEVICE_INFO* pInfo);
    pub(crate) ZCAN_GetDeviceInf: Symbol<'a, unsafe extern "C" fn(dev_hdl: c_uint, info: *mut ZDeviceInfo) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_IsDeviceOnLine(DEVICE_HANDLE device_handle);
    //ZCAN_IsDeviceOnLine: Symbol<'a, unsafe extern "C" fn(dev_hdl: c_uint) -> c_uint>,

    /// CHANNEL_HANDLE FUNC_CALL ZCAN_InitCAN(DEVICE_HANDLE device_handle, UINT can_index, ZCAN_CHANNEL_INIT_CONFIG* pInitConfig);
    pub(crate) ZCAN_InitCAN: Symbol<'a, unsafe extern "C" fn(dev_hdl: c_uint, channel: c_uint, cfg: *const ZCanChlCfg) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_StartCAN(CHANNEL_HANDLE channel_handle);
    pub(crate) ZCAN_StartCAN: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_ResetCAN(CHANNEL_HANDLE channel_handle);
    pub(crate) ZCAN_ResetCAN: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_ClearBuffer(CHANNEL_HANDLE channel_handle);
    pub(crate) ZCAN_ClearBuffer: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_ReadChannelErrInfo(CHANNEL_HANDLE channel_handle, ZCAN_CHANNEL_ERR_INFO* pErrInfo);
    pub(crate) ZCAN_ReadChannelErrInfo: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, err: *mut ZCanChlError) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_ReadChannelStatus(CHANNEL_HANDLE channel_handle, ZCAN_CHANNEL_STATUS* pCANStatus);
    pub(crate) ZCAN_ReadChannelStatus: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, status: *mut ZCanChlStatus) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_GetReceiveNum(CHANNEL_HANDLE channel_handle, BYTE type);    //type:TYPE_CAN, TYPE_CANFD, TYPE_ALL_DATA
    pub(crate) ZCAN_GetReceiveNum: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, can_type: c_uchar) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_Transmit(CHANNEL_HANDLE channel_handle, ZCAN_Transmit_Data* pTransmit, UINT len);
    pub(crate) ZCAN_Transmit: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, frames: *const ZCanFrame, len: c_uint) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_Receive(CHANNEL_HANDLE channel_handle, ZCAN_Receive_Data* pReceive, UINT len, int wait_time DEF(-1));
    pub(crate) ZCAN_Receive: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, frames: *mut ZCanFrame, size: c_uint, timeout: c_uint) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_TransmitFD(CHANNEL_HANDLE channel_handle, ZCAN_TransmitFD_Data* pTransmit, UINT len);
    pub(crate) ZCAN_TransmitFD: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, frames: *const ZCanFrame, len: c_uint) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_ReceiveFD(CHANNEL_HANDLE channel_handle, ZCAN_ReceiveFD_Data* pReceive, UINT len, int wait_time DEF(-1));
    pub(crate) ZCAN_ReceiveFD: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, frames: *mut ZCanFrame, size: c_uint, timeout: c_uint) -> c_uint>,

    /// UINT FUNC_CALL ZCAN_TransmitData(DEVICE_HANDLE device_handle, ZCANDataObj* pTransmit, UINT len);
    // ZCAN_TransmitData: Symbol<'a, unsafe extern "C" fn(dev_hdl: c_uint, data: *const ZCANDataObj, len: c_uint) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_ReceiveData(DEVICE_HANDLE device_handle, ZCANDataObj* pReceive, UINT len, int wait_time DEF(-1));
    // ZCAN_ReceiveData: Symbol<'a, unsafe extern "C" fn(dev_hdl: c_uint, frames: *mut ZCANDataObj, size: c_uint, timeout: c_uint) -> c_uint>,

    /// UINT FUNC_CALL ZCAN_SetValue(DEVICE_HANDLE device_handle, const char* path, const void* value);
    // ZCAN_SetValue: Symbol<'a, unsafe extern "C" fn(dev_hdl: c_uint, path: *const c_char, value: *const c_void) -> c_uint>,
    /// const void* FUNC_CALL ZCAN_GetValue(DEVICE_HANDLE device_handle, const char* path);
    // ZCAN_GetValue: Symbol<'a, unsafe extern "C" fn(dev_hdl: c_uint, path: *const c_char) -> *const c_void>,
    /// IProperty* FUNC_CALL GetIProperty(DEVICE_HANDLE device_handle);
    pub(crate) GetIProperty: Symbol<'a, unsafe extern "C" fn(dev_hdl: c_uint) -> *const IProperty>,
    /// UINT FUNC_CALL ReleaseIProperty(IProperty * pIProperty);
    pub(crate) ReleaseIProperty: Symbol<'a, unsafe extern "C" fn(p: *const IProperty) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_GetReference(UINT DeviceType, UINT nDevIndex, UINT nChnlIndex, UINT nRefType, void* pData);
    pub(crate) ZCAN_GetReference: Symbol<'a, unsafe extern "C" fn(dev_type: c_uint, dev_idx: c_uint, chl: c_uint, cmd: c_uint, value: *mut c_void) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_SetReference(UINT DeviceType, UINT nDevIndex, UINT nChnlIndex, UINT nRefType, void* pData);
    pub(crate) ZCAN_SetReference: Symbol<'a, unsafe extern "C" fn(dev_type: c_uint, dev_idx: c_uint, chl: c_uint, cmd: c_uint, value: *const c_void) -> c_uint>,
}

#[allow(dead_code)]
impl USBCANFD800UApi<'_> {
    pub(crate) const INVALID_DEVICE_HANDLE: u32 = 0;
    pub(crate) const INVALID_CHANNEL_HANDLE: u32 = 0;
    pub(crate) const STATUS_OK: u32 = 1;
    // #define MAX_DEVICE_COUNT                        32  //支持的设备数量
    // #define DEVICE_CAN_CHNL_COUNT_MAX               8   //支持最大的CAN通道数量,实际通道数量可能小于此数值
    // #define DEVICE_LIN_CHNL_COUNT_MAX               4   //支持最大的LIN通道数量,实际通道数量可能小于此数值
    // #define DEVICE_TOTAL_CHNL_COUNT                 (DEVICE_CAN_CHNL_COUNT_MAX + DEVICE_LIN_CHNL_COUNT_MAX)
    // #define FILTER_RULE_COUNT_MAX                   64  //设备允许的过滤条数
    // #define DEV_AUTO_SEND_INDEX_MAX                 32  //定时发送索引最大值
    pub(crate) const REF_CONTROLLER_TYPE: u32 = 1; // pData 指向uint32_t, 0:CAN; 1：ISO CANFD; 2:Non-ISO CANFD, 需要在StartCAN之前设置
    pub(crate) const REF_ADD_FILTER: u32 = 2; // 添加通道过滤条目，pData Pointer to RefFilterItem(12 Bytes)
    pub(crate) const REF_APPLY_FILTER: u32 = 3; // 应用通道过滤
    pub(crate) const REF_CLEAR_FILTER: u32 = 4; // 清除通道过滤
    pub(crate) const REF_UPDATE_FIRMWARE: u32 = 5; // pData Pointer to FirmwareUpdateParam结构,指示固件路径
    pub(crate) const REF_GET_UPDATE_STATUS: u32 = 6; // pData Pointer to FirmwareUpdateStatus
    pub(crate) const REF_ADD_TIMER_SEND_CAN: u32 = 7; // pData Pointer to ZCAN_AUTO_TRANSMIT_OBJ
    pub(crate) const REF_ADD_TIMER_SEND_CANFD: u32 = 8; // pData Pointer to ZCANFD_AUTO_TRANSMIT_OBJ
    pub(crate) const REF_APPLY_TIMER_SEND: u32 = 9; // Start Timer Send
    pub(crate) const REF_APPLY_TIMER_SEND_FD: u32 = 10; // Stop Timer Send & Clear Send List
    pub(crate) const REF_INTERNAL_RESISTANCE: u32 = 11; // pData 指向uint32_t, 0:断开内置终端电阻；1：使用设备内部终端电阻, 需要在StartCAN之前设置
    pub(crate) const REF_SET_DEVICE_NAME: u32 = 12; // 设备设备名称，pData Pointer to char*
    pub(crate) const REF_GET_DEVICE_NAME: u32 = 13; // 设备设备名称，pData 指向用户申请内存，大小需要足够容纳设备名字
    pub(crate) const REF_CLEAR_DEVICE_LOG: u32 = 14; // 清除设备日志
    pub(crate) const REF_GET_DEVICE_LOG_SIZE: u32 = 15; // 获取设备日志大小，pData Pointer to uint32_t
    pub(crate) const REF_GET_DEVICE_LOG_DATA: u32 = 16; // 设备设备日志内容，pData 指向用户申请内存，大小需要足够容纳设备日志
    pub(crate) const REF_SET_DATA_RECV_MERGE: u32 = 17; // 设置合并接收数据，CAN/LIN/GPS以及不同通道的数据合并接收,pData Pointer to uint32_t, 0:关闭合并接收，1：开启合并接收
    pub(crate) const REF_GET_DATA_RECV_MERGE: u32 = 18; // 获取合并接收数据状态，pData Pointer to uint32_t, 0:合并接收关闭，1：合并接收处于开启状态
    pub(crate) const REF_INTERNAL_TEST: u32 = 19;
    pub(crate) const REF_VERIFY_DEVICE_BY_PASS: u32 = 20; // ZCANPRO验证设备，pData数据类型为指向VerifyDeviceData的指针
    pub(crate) const REF_ENABLE_BUS_USAGE: u32 = 21; // pData 指向uint32_t, 0:关闭总线利用率上报，1：开启总线利用率上报，需要在StartCAN之前设置
    pub(crate) const REF_SET_BUS_USAGE_PERIOD: u32 = 22; // pData 指向uint32_t, 表示设备上报周期，单位毫秒，范围20-2000ms, 需要在StartCAN之前设置
    pub(crate) const REF_GET_BUS_USAGE: u32 = 23; // /获取总线利用率, pData指向 BusUsage
    pub(crate) const REF_GET_DELAY_SEND_AVAILABLE_COUNT: u32 = 24; // 获取设备端延迟发送可用数量 pData Pointer to uint32_t
    pub(crate) const REF_CLEAR_DELAY_SEND_QUEUE: u32 = 25; // 如果队列发送中有数据因为时间未到未发送，取消设备当前的队列发送
    pub(crate) const REF_GET_LIN_TX_FIFO_TOTAL: u32 = 26; // 获取LIN发送缓冲区大小
    pub(crate) const REF_GET_LIN_TX_FIFO_AVAILABLE: u32 = 27; // 获取LIN发送缓冲区可用大小
    pub(crate) const REF_ADD_TIMER_SEND_CAN_DIRECT: u32 = 28;
    pub(crate) const REF_ADD_TIMER_SEND_CANFD_DIRECT: u32 = 29; //
    pub(crate) const REF_GET_DEV_CAN_AUTO_SEND_COUNT: u32 = 30; // 获取设备端定时发送CAN帧的数量，pData指向uint32_t,表示设备端定时发送CAN帧数量
    pub(crate) const REF_GET_DEV_CAN_AUTO_SEND_DATA: u32 = 31; // 获取设备端定时发送CAN帧的数据，用户根据查询到的CAN帧数量申请内存 sizeof(ZCAN_AUTO_TRANSMIT_OBJ) * N，将申请到的内存地址填入pData
    pub(crate) const REF_GET_DEV_CANFD_AUTO_SEND_COUNT: u32 = 32; // 获取设备端定时发送CANFD帧的数量，pData指向uint32_t,表示设备端定时发送CANFD帧数量
    pub(crate) const REF_GET_DEV_CANFD_AUTO_SEND_DATA: u32 = 33; // 获取设备端定时发送CANFD帧的数据，用户根据查询到的CAN帧数量申请内存 sizeof(ZCANFD_AUTO_TRANSMIT_OBJ) * N，将申请到的内存地址填入pData
    pub(crate) const REF_SET_TX_ECHO: u32 = 34; // 设置库强制发送回显,pData指向uint32_t，0表示不开启发送回显，1表示开启发送回显，开启后，普通发送也会设置发送回显请求标志
    pub(crate) const REF_GET_TX_ECHO: u32 = 35; // 查询是否设置了强制发送回显,pData指向uint32_t，0表示不开启发送回显，1表示开启发送回显
    pub(crate) const REF_SET_TX_RETRY_POLICY: u32 = 36; // 发送失败是否重传：0：发送失败不重传；1：发送失败重传，直到总线关闭。
    pub(crate) const REF_SET_TX_TIMEOUT: u32 = 37; // 发送超时时间，单位ms；设置后发送达到超时时间后，取消当前报文发送；取值范围0-2000ms。
    pub(crate) const REF_GET_TX_TIMEOUT: u32 = 38; // 获取发送超时时间

    #[inline]
    pub(crate) fn init_can_chl_ex(
        &self,
        dev_type: ZCanDeviceType,
        dev_idx: u32,
        channel: u8,
        cfg: &ChannelConfig,
    ) -> Result<(), CanError> {
        // set channel resistance status
        if dev_type.has_resistance() {
            let state = cfg.resistance().unwrap_or(true) as u32;
            let cmd_path = CmdPath::new_reference(USBCANFD800UApi::REF_INTERNAL_RESISTANCE);
            self.self_set_reference(
                dev_type,
                dev_idx,
                channel,
                cmd_path.get_reference(),
                &state as *const c_uint as *const c_void,
            )?;
        }
        // set channel protocol
        let can_type = cfg
            .get_other::<ZCanChlType>(constants::CHANNEL_TYPE)?
            .unwrap_or(ZCanChlType::CANFD_ISO) as u32;
        let cmd_path = CmdPath::new_reference(USBCANFD800UApi::REF_CONTROLLER_TYPE);
        self.self_set_reference(
            dev_type,
            dev_idx,
            channel,
            cmd_path.get_reference(),
            &can_type as *const c_uint as *const c_void,
        )
    }

    #[inline]
    pub(crate) fn self_set_reference(
        &self,
        dev_type: ZCanDeviceType,
        dev_idx: u32,
        channel: u8,
        cmd: c_uint,
        value: *const c_void,
    ) -> Result<(), CanError> {
        match unsafe {
            (self.ZCAN_SetReference)(dev_type as u32, dev_idx, channel as u32, cmd, value)
        } {
            Self::STATUS_OK => Ok(()),
            code => Err(CanError::OperationError(format!(
                "`ZCAN_SetReference` ret: {}",
                code
            ))),
        }
    }

    #[inline]
    pub(crate) fn self_get_reference(
        &self,
        dev_type: ZCanDeviceType,
        dev_idx: u32,
        channel: u8,
        cmd: c_uint,
        value: *mut c_void,
    ) -> Result<(), CanError> {
        match unsafe {
            (self.ZCAN_GetReference)(dev_type as u32, dev_idx, channel as u32, cmd, value)
        } {
            Self::STATUS_OK => Ok(()),
            code => Err(CanError::OperationError(format!(
                "`ZCAN_GetReference` ret: {}",
                code
            ))),
        }
    }
}

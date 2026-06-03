#ifndef _ZUDS_H_
#define _ZUDS_H_

typedef unsigned char      uint8;
typedef unsigned short     uint16;
typedef unsigned int       uint32;
typedef unsigned long long uint64;
typedef uint32 ZUDS_HANDLE;
#define ZUDS_INVALID_HANDLE ((ZUDS_HANDLE)-1)

// UDS服务标识
#define ZUDS_SI_DiagnosticSessionControl           0x10
#define ZUDS_SI_ECUReset                           0x11
#define ZUDS_SI_ClearDiagnosticInformation         0x14
#define ZUDS_SI_ReadDTCInformation                 0x19
#define ZUDS_SI_ReadDataByIdentifier               0x22
#define ZUDS_SI_ReadMemoryByAddress                0x23
#define ZUDS_SI_ReadScalingDataByIdentifier        0x24
#define ZUDS_SI_SecurityAccess                     0x27
#define ZUDS_SI_CommunicationControl               0x28
#define ZUDS_SI_ReadDataByPeriodicIdentifier       0x2A
#define ZUDS_SI_DynamicallyDefineDataIdentifier    0x2C
#define ZUDS_SI_WriteDataByIdentifier              0x2E
#define ZUDS_SI_InputOutputControlByIdentifier     0x2F
#define ZUDS_SI_RoutineControl                     0x31
#define ZUDS_SI_RequestDownload                    0x34
#define ZUDS_SI_RequestUpload                      0x35
#define ZUDS_SI_TransferData                       0x36
#define ZUDS_SI_RequestTransferExit                0x37
#define ZUDS_SI_WriteMemoryByAddress               0x3D
#define ZUDS_SI_TesterPresent                      0x3E
#define ZUDS_SI_AccessTimingParameter              0x83
#define ZUDS_SI_SecuredDataTransmission            0x84
#define ZUDS_SI_ControlDTCSetting                  0x85
#define ZUDS_SI_ResponseOnEvent                    0x86
#define ZUDS_SI_LinkControl                        0x87

/* 结构体按1字节对齐 */
#pragma pack(push,1)

typedef struct _ZUDS_REQUEST
{
    uint32 src_addr;             // 请求地址
    uint32 dst_addr;             // 响应地址
    uint8  suppress_response;    // 1:抑制响应
    uint8  sid;                  // 请求服务id
    uint16 reserved0;            // 保留
    uint8  *param;               // 参数数组
    uint32 param_len;            // 参数数组的长度
    uint32 reserved;             // 保留
}ZUDS_REQUEST;

//错误码
typedef uint8 UDS_STATUS;
#define ZUDS_ERROR_OK                   0    // 没错误
#define ZUDS_ERROR_TIMEOUT              1    // 响应超时
#define ZUDS_ERROR_TRANSPORT            2    // 发送数据失败
#define ZUDS_ERROR_CANCEL               3    // 取消请求
#define ZUDS_ERROR_SUPPRESS_RESPONSE    4    // 抑制响应
#define ZUDS_ERROR_OTHTER               100

typedef uint8 RESPONSE_TYPE;
#define RT_POSITIVE 1 // 积极响应
#define RT_NEGATIVE 0 // 消极响应

typedef struct _ZUDS_RESPONSE
{
    UDS_STATUS status;  // 错误码
    RESPONSE_TYPE type; // RT_POSITIVE, RT_NEGATIVE
    union
    {
        struct
        {
            uint8  sid;                // 响应服务id
            uint8  *param;             // 参数数组, 不用释放
            uint32 param_len;          // 参数数组的长度
        }positive;
        struct
        {
            uint8  neg_code;            // 0x7F
            uint8  sid;                 // 请求服务id
            uint8  error_code;          // 错误码
        }negative;
    };
    uint32 reserved;                    // 保留
}ZUDS_RESPONSE;

typedef struct _ZUDS_FRAME
{
    uint32 id;                // 帧id
    uint8  extend;            // 1:扩展帧 0:标准帧
    uint8  remote;            // 1:远程帧 0:数据帧
    uint8  data_len;          // 数据长度
    uint8  data[64];          // 数据数组
    uint32 reserved;          // 保留
}ZUDS_FRAME;

typedef uint8 PARAM_TYPE;
#define PARAM_TYPE_SESSION   0 // ZUDS_SESSION_PARAM
#define PARAM_TYPE_ISO15765  1 // ZUDS_ISO15765_PARAM
#define PARAM_TYPE_TRANSPORT 2

// 会话层设置参数
typedef struct _ZUDS_SERSSION_PARAM
{
    uint16 timeout;             // 响应超时时间(ms)。因PC定时器误差，建议设置不小于200ms
    uint16 enhanced_timeout;    // 收到消极响应错误码为0x78后的超时时间(ms)。因PC定时器误差，建议设置不小于200ms
    uint8 check_any_negative_response:1;  // 接收到非本次请求服务的消极响应时是否需要判定为响应错误
    uint8 wait_if_suppress_response:1;    // 抑制响应时是否需要等待消极响应，等待时长为响应超时时间
    uint8 flag:6;               // 保留
    uint8 reserved0[3];         // 保留
    uint32 reserved1;           // 保留
} ZUDS_SESSION_PARAM;

// 传输协议版本
#define VERSION_0 0             // ISO15765-2(2004版本)
#define VERSION_1 1             // ISO15765-2(2016版本)

//#define FILL_MODE_NONE     0    // 不填充
//#define FILL_MODE_SHORT    1    // 小于8字节填充至8字节，大于8字节时按DLC就近填充
#define FILL_MODE_MAX      2    // 填充至最大数据长度 (不建议)

// 传输层设置参数
typedef struct _ZUDS_ISO15765_PARAM
{
    uint8  version;           // 传输协议版本, VERSION_0, VERSION_1
    uint8  max_data_len;      // 单帧最大数据长度, can:8, canfd:64
    uint8  local_st_min;      // 本程序发送流控时用，连续帧之间的最小间隔, 0x00-0x7F(0ms~127ms), 0xF1-0xF9(100us~900us)
    uint8  block_size;        // 流控帧的块大小
    uint8  fill_byte;         // 无效字节的填充数据
    uint8  frame_type;        // 0:标准帧 1:扩展帧
    uint8  is_modify_ecu_st_min; // 是否忽略ECU返回流控的STmin，强制使用本程序设置的 remote_st_min
    uint8  remote_st_min;        // 发送多帧时用, is_ignore_ecu_st_min = 1 时有效, 0x00-0x7F(0ms~127ms), 0xF1-0xF9(100us~900us)
    uint16 fc_timeout;        // 接收流控超时时间(ms), 如发送首帧后需要等待回应流控帧，范围 [20, 10000]
    uint8  fill_mode;           // 数据长度填充模式, FILL_MODE_NONE, FILL_MODE_SHORT, FILL_MODE_MAX
    uint8  reserved[5];       // 保留
} ZUDS_ISO15765_PARAM;

// 会话保持的设置参数
typedef struct _ZUDS_TESTER_PRESENT_PARAM
{
    uint32 addr;                 // 请求地址
    uint16 cycle;                // 发送周期, 单位毫秒
    uint8  suppress_response;    // 1:抑制响应
    uint32 reserved;             // 保留
}ZUDS_TESTER_PRESENT_PARAM;

#pragma pack(pop)

#define TRANSPORT_OK    0
#define TRANSPORT_ERROR 1


#ifdef WIN32
#define STDCALL __stdcall
#else
#define STDCALL
#endif

typedef uint32 TP_TYPE; // 传输层协议
#define DoCAN 0



#ifdef __cplusplus
extern "C" {
#endif

   /**
    * @brief 初始化资源, 获取操作句柄
    * @param[in] type 目前只支持 DoCAN
    * @return 操作句柄, 供之后的函数使用; 初始化失败返回ZUDS_INVALID_HANDLE。
    */ 
    ZUDS_HANDLE STDCALL ZUDS_Init(TP_TYPE type);

    /**
     * @brief 执行服务请求, 阻塞函数, ECU有响应或响应超时返回。
     * @param[in] handle 操作句柄，ZUDS_Init的返回值, 下同。
     * @param[in] request 请求结构体, 详见 ZUDS_REQUEST。
     * @param[out] resposne 响应结构体, 详见ZUDS_RESPONSE。
     */
    void STDCALL ZUDS_Request(ZUDS_HANDLE handle, const ZUDS_REQUEST* request, ZUDS_RESPONSE *response);

    /** 
     * @brief 中止请求, ZUDS_Request返回
     */
    void STDCALL ZUDS_Stop(ZUDS_HANDLE handle);

    /** 
     * @brief 发送帧数据回调函数, 用户自行实现。
     * @param[in] ctx 通过ZUDS_SetTransmitHandler传入的参数, 回调函数中传出
     * @param[in] frame 帧数组
     * @param[in] count 帧数组元素个数
     * @return 返回值为 TRANSPORT_OK 或 TRANSPORT_ERROR
     */
    typedef uint32(STDCALL *OnUDSTransmit)(void* ctx, const ZUDS_FRAME* frame, uint32 count);

    /** 
     * @brief 设置发送回调函数。
     * @param[in] handle 操作句柄
     * @param[in] ctx 上下文参数, 在回调函数中传出, 库内部不会处理该参数。
     * @param[in] transmittor 回调函数, 一定要实现, 不然无法实现报文交互。
     */
    void STDCALL ZUDS_SetTransmitHandler(ZUDS_HANDLE handle, void* ctx, OnUDSTransmit transmittor);

    /** 
     * @brief 库外部接收到的帧数据通过该函数传入, 一定要调用, 不然无法实现报文交互
     * @param[in] handle 操作句柄
     * @param[in] frame 数据帧
     */
    void STDCALL ZUDS_OnReceive(ZUDS_HANDLE handle, const ZUDS_FRAME* frame);

    /**
     * @brief 设置参数
     *      会话层参数, type=PARAM_TYPE_SESSION, param:详见ZUDS_SESSION_PARAM
     *      传输层参数, type=PARAM_TYPE_ISO15765, param:详见ZUDS_ISO15765_PARAM
     * @param[in] handle 操作句柄
     * @param[in] type 类型
     * @param[in] param 参数结构
     */
    void STDCALL ZUDS_SetParam(ZUDS_HANDLE handle, PARAM_TYPE type, void* param);

    /** 
     * @brief 用于实现会话保持。
     * @param[in] handle 操作句柄
     * @param[in] enable 是否使能会话保持
     * @param[in] param 参数配置
     */
    void STDCALL ZUDS_SetTesterPresent(ZUDS_HANDLE handle, uint8 enable, const ZUDS_TESTER_PRESENT_PARAM* param);

    /**
     * @brief 启动独立的流控响应。当接收到响应地址的多帧数据时，按请求地址回应流控数据。
     *      由于ZUDS_Request请求过程中也会回应流控帧，本功能可能被ZUDS_Request修改响应地址而影响，所以不建议与请求同时使用。请单独申请个ZUDS_HANDLE做独立的流控响应。
     *      如果使用两个ZUDS_HANDLE，为了避免发送两次相同的流控响应，请使用ZUDS_Request前关闭本功能。
     * @param[in] handle 操作句柄
     * @param[in] src_addr 请求地址
     * @param[in] dst_addr 响应地址
     * @return 返回错误码
     */
    UDS_STATUS STDCALL ZUDS_StartAloneFlowControl(ZUDS_HANDLE handle, uint32 src_addr, uint32 dst_addr);

    /**
     * @brief 停止独立流控
     * @param[in] handle 操作句柄
     */
    void STDCALL ZUDS_StopAloneFlowControl(ZUDS_HANDLE handle);

    /**
     * @brief 释放资源, 与ZUDS_Init配对使用
     * @param[in] handle 操作句柄
     */
    void STDCALL ZUDS_Release(ZUDS_HANDLE handle);


#ifdef __cplusplus
}
#endif

#endif // _ZUDS_H_

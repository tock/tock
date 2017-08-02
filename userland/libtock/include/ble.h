#pragma once

#include "tock.h"

#define DRIVER_RADIO      33

/*---------------------SYSCALLS---------------------------------------------*/

/*--COMMAND CALLS-------------------------------------*/
#define BLE_ADV_START          0
#define BLE_ADV_STOP           1
#define BLE_CFG_TX_POWER       2
#define BLE_CFG_ADV_INTERVAL   3
#define BLE_ADV_CLEAR_DATA     4
#define BLE_SCAN               5
/*----END COMMAND CALLS-------------------------------*/

/*--ALLOW CALLS---------------------------------------*/

// AD Types
#define BLE_HS_ADV_TYPE_FLAGS                   0x01
#define BLE_HS_ADV_TYPE_INCOMP_UUIDS16          0x02
#define BLE_HS_ADV_TYPE_COMP_UUIDS16            0x03
#define BLE_HS_ADV_TYPE_INCOMP_UUIDS32          0x04
#define BLE_HS_ADV_TYPE_COMP_UUIDS32            0x05
#define BLE_HS_ADV_TYPE_INCOMP_UUIDS128         0x06
#define BLE_HS_ADV_TYPE_COMP_UUIDS128           0x07
#define BLE_HS_ADV_TYPE_INCOMP_NAME             0x08
#define BLE_HS_ADV_TYPE_COMP_NAME               0x09
#define BLE_HS_ADV_TYPE_TX_PWR_LVL              0x0a
#define BLE_HS_ADV_TYPE_SLAVE_ITVL_RANGE        0x12
#define BLE_HS_ADV_TYPE_SOL_UUIDS16             0x14
#define BLE_HS_ADV_TYPE_SOL_UUIDS128            0x15
#define BLE_HS_ADV_TYPE_SVC_DATA_UUID16         0x16
#define BLE_HS_ADV_TYPE_PUBLIC_TGT_ADDR         0x17
#define BLE_HS_ADV_TYPE_RANDOM_TGT_ADDR         0x18
#define BLE_HS_ADV_TYPE_APPEARANCE              0x19
#define BLE_HS_ADV_TYPE_ADV_ITVL                0x1a
#define BLE_HS_ADV_TYPE_SVC_DATA_UUID32         0x20
#define BLE_HS_ADV_TYPE_SVC_DATA_UUID128        0x21
#define BLE_HS_ADV_TYPE_URI                     0x24
#define BLE_HS_ADV_TYPE_MFG_DATA                0xff

// ETC
#define BLE_CFG_ADV_ADDR                        0x30
#define BLE_CFG_SCAN_BUF                        0x31 

/*-----END ALLOW CALLS---------------------------------*/

/*--- SUBSCRIBE CALLS----------------------------------*/
#define BLE_SCAN_CALLBACK                       0
/*----END COMMAND CALLS--------------------------------*/

// BLE MODES
// CONN_NON   - a device which only is advertising (broadcast) and not connectable
// FIXME: the others are not supported yet
typedef enum {
    CONN_NON     = 0x00,
    CONN_DIR     = 0x01,
    CONN_UND     = 0x02,
    SCAN_NON     = 0x03,
    SCAN_DIR     = 0x04,
    SCAN_UND     = 0x05,
} BLE_Gap_Mode_t;


// TX power
// FIXME: Not platform independent.
typedef enum {
    POS4_DBM     = 0x04,
    ODBM         = 0x00,
    NEG4_DBM     = 0xFC,
    NEG_8_DBM    = 0xF8,
    NEG_12_DBM   = 0xF4,
    NEG_16_DBM   = 0xF0,
    NEG_20_DBM   = 0xEC,
    NEG_30_DBM   = 0xD8
} BLE_TX_Power_t;

#ifdef __cplusplus
extern "C" {
#endif

// configure advertisement data
//
// type     - ad type
// data     - buffer with data
// len      - length of the data buffer
int ble_adv_data(uint8_t type, const unsigned char *data, uint8_t len);

// clear the advertisement data
int ble_adv_clear_data(void);

// configure tx power
//
// power    - tx power in DBm (-30 to +4)
int ble_adv_set_txpower(BLE_TX_Power_t power);


// configure advertisement interval
//
// interval   - advertisement interval in milliseconds (20ms - 10240ms)
int ble_adv_set_interval(uint16_t interval);

// start send BLE advertisement periodically according to the configured interval
// if no interval is connected it will use a pre-configured value
//
// mode     - advertising mode (non-connectable, connectable or active scanner)
// data     - only used for scanning to provide a buffer to user-space
// len      - length of the buffer of the scanning result (should be 39 bytes)
int ble_adv_start(BLE_Gap_Mode_t mode);

// Temporary function that sends all received advertisements to user-space
int ble_adv_scan(const unsigned char *data, uint8_t len, subscribe_cb callback);

// stop advertising
int ble_adv_stop(void);

// configure an address to be used for advertising
//
// data - buffer with the address
// len  - length of the buffer with the address (should be 6 bytes)
int ble_adv_set_address(const unsigned char *data, uint8_t len);

#ifdef __cplusplus
}
#endif

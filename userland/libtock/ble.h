#pragma once

#include <tock.h>

/*------- SYSCALLS------------*/
#define DRIVER_RADIO      33

// commands calls
#define BLE_ADV_START      0
#define BLE_ADV_STOP       1
#define CFG_TX_POWER       2
#define CFG_ADV_INTERVAL   3
#define BLE_ADV_CLEAR_DATA 4
/*----------------------------*/

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
} TX_Power_t;

#ifdef __cplusplus
extern "C" {
#endif

int ble_adv_data(uint8_t type, uint8_t len, const unsigned char *data);
int ble_adv_clear_data(void);
int ble_adv_set_txpower(TX_Power_t power);
int ble_adv_set_interval(uint16_t);
int ble_adv_start(void);
int ble_adv_stop(void);

#ifdef __cplusplus
}
#endif

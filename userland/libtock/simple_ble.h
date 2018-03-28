#pragma once

#include "tock.h"

#ifdef __cplusplus
extern "C" {
#endif

/*******************************************************************************
 *   DRIVER DEFINITIONS   -- corresponds to different system calls
 *
 *    *_CMD   - command call
 *    *_ALLOW - allow call
 *    *_SUB   - subscribe call
 *
 *    All enumerations in GapAdvertisementData_t corresponds to allow calls
 *    in the BluetoothAdvertisingDriver
 ******************************************************************************/

#define BLE_DRIVER_NUMBER 0x30000
#define BLE_ADV_START_CMD 0
#define BLE_ADV_STOP_CMD 1
#define BLE_CFG_TX_POWER_CMD 2
#define BLE_CFG_ADV_ITV_CMD 3
#define BLE_ADV_CLEAR_DATA_CMD 4
#define BLE_SCAN_CMD 5
#define BLE_REQ_ADV_ADDR 6
#define BLE_SCAN_SUB 0
#define BLE_CFG_GAP_BUF_ALLOW 0
#define BLE_CFG_SCAN_BUF_ALLOW 1
#define BLE_CFG_ADV_BUF_ALLOW 2

typedef enum {
  GAP_FLAGS = 0x01, /* Flags, see enum below */
  GAP_INCOMPLETE_LIST_16BIT_SERVICE_IDS =
      0x02, /* Incomplete list of 16-bit Service IDs. */
  GAP_COMPLETE_LIST_16BIT_SERVICE_IDS =
      0x03, /* Complete list of 16-bit Service IDs. */
  GAP_INCOMPLETE_LIST_32BIT_SERVICE_IDS =
      0x04, /* Incomplete list of 32-bit Service IDs (not relevant for
               Bluetooth 4.0). */
  GAP_COMPLETE_LIST_32BIT_SERVICE_IDS =
      0x05, /* Complete list of 32-bit Service IDs (not relevant for
               Bluetooth 4.0). */
  GAP_INCOMPLETE_LIST_128BIT_SERVICE_IDS =
      0x06, /* Incomplete list of 128-bit Service IDs. */
  GAP_COMPLETE_LIST_128BIT_SERVICE_IDS =
      0x07,                        /* Complete list of 128-bit Service IDs. */
  GAP_SHORTENED_LOCAL_NAME = 0x08, /* Shortened Local Name. */
  GAP_COMPLETE_LOCAL_NAME = 0x09,  /* Complete Local Name. */
  GAP_TX_POWER_LEVEL = 0x0A,       /* TX Power Level (in dBm). */
  GAP_DEVICE_ID = 0x10,            /* Device ID. */
  GAP_SLAVE_CONNECTION_INTERVAL_RANGE =
      0x12, /* Slave Connection Interval Range. */
  GAP_LIST_128BIT_SOLICITATION_IDS =
      0x15, /* List of 128 bit service UUIDs the device is looking for. */
  GAP_SERVICE_DATA = 0x16,              /* Service Data. */
  GAP_APPEARANCE = 0x19,                /* Appearance */
  GAP_ADVERTISING_INTERVAL = 0x1A,      /* Advertising Interval. */
  GAP_MANUFACTURER_SPECIFIC_DATA = 0xFF /* Manufacturer Specific Data. */
} GapAdvertisementData_t;

enum {
  LE_LIMITED_DISCOVERABLE = 0x01, /* Peripheral device is discoverable for a
                                     limited period of time. */
  LE_GENERAL_DISCOVERABLE =
      0x02, /* Peripheral device is discoverable at any moment. */
  BREDR_NOT_SUPPORTED = 0x04,     /* Peripheral device is LE only. */
  SIMULTANEOUS_LE_BREDR_C = 0x08, /* Not relevant - central mode only. */
  SIMULTANEOUS_LE_BREDR_H = 0x10  /* Not relevant - central mode only. */
} GapFlags_t;

typedef enum {
  POSITIVE_10_DBM = 10,
  POSITIVE_9_DBM = 9,
  POSITIVE_8_DBM = 8,
  POSITIVE_7_DBM = 7,
  POSITIVE_6_DBM = 6,
  POSITIVE_5_DBM = 5,
  POSITIVE_4_DBM = 4,
  POSITIVE_3_DBM = 3,
  POSITIVE_2_DBM = 2,
  POSITIVE_1_DBM = 1,
  ZERO_DBM = 0,
  NEGATIVE_1_DBM = 0xff,
  NEGATIVE_2_DBM = 0xfe,
  NEGATIVE_3_DBM = 0xfd,
  NEGATIVE_4_DBM = 0xfc,
  NEGATIVE_5_DBM = 0xfb,
  NEGATIVE_6_DBM = 0xfa,
  NEGATIVE_7_DBM = 0xf9,
  NEGATIVE_8_DBM = 0xf8,
  NEGATIVE_9_DBM = 0xf7,
  NEGATIVE_10_DBM = 0xf6,
  NEGATIVE_11_DBM = 0xf5,
  NEGATIVE_12_DBM = 0xf4,
  NEGATIVE_13_DBM = 0xf3,
  NEGATIVE_14_DBM = 0xf2,
  NEGATIVE_15_DBM = 0xf1,
  NEGATIVE_16_DBM = 0xf0,
  NEGATIVE_17_DBM = 0xef,
  NEGATIVE_18_DBM = 0xee,
  NEGATIVE_19_DBM = 0xed,
  NEGATIVE_20_DBM = 0xec,
} TxPower_t;

/*******************************************************************************
 *   User-space API
 ******************************************************************************/

// initialize advertisement (should be used before invoked
// ble_start_advertising)
//
// advertising_iterval_ms  - advertisement interval in milliseconds
// discoverable            - if the device should be discoverable or not
int ble_initialize(uint16_t advertising_interval_ms, bool discoverable);

// start advertising
int ble_start_advertising(void);

// stop advertising but don't change anything in the packet configuration
int ble_stop_advertising(void);

// reset the entire advertisement data packet
// excluding flags and advertisement address
int ble_reset_advertisement(void);

// configure advertisement name
//
// device_name         - device named to be used in the advertisement
// size_b              - size of device in bytes
int ble_advertise_name(uint8_t *device_name, uint8_t size_b);

// configure list of 16 bit uuids
//
// uuid16               - array of 16 bit uuids
// size_b               - size of uuid16 in bytes
int ble_advertise_uuid16(uint16_t *uuid16, uint8_t size_b);

// configure service data
//
// uuid16               - 16 bit uuid to be associated with the data
// data                 - array of data in bytes
// size_b               - size of data in bytes
int ble_advertise_service_data(uint16_t uuid16, uint8_t *data, uint8_t size_b);

// configure manufacturer specific data
//
// data                 - array of data in bytes
// size_b               - size of data in bytes
int ble_advertise_manufacturer_specific_data(uint8_t *data, uint8_t size_b);

// passive scanning of advertisements
//
// data                 - array of bytes to write the received advertisment to
// len                  - max_size (39 bytes)
// callback             - callback handler to call when an advertisement is
//                        received
//
// type signature of the callback handler:
// static void callback(int result,
//                      int len,
//                      __attribute__((unused)) int unused2,
//                      __attribute__((unused)) void* ud);
//
// The kernel will fill the array of bytes with the received advertisement
// it's then up to user-space application to determine what to do in the
// callback handler.
//
// result               - kernel indicates whether the radio rx was successful
//                        or not
// len                  - the number of bytes received via the radio
//
int ble_start_passive_scan(uint8_t *data, uint8_t len, subscribe_cb callback);

// stop passive scanning
int ble_stop_passive_scan(void);

// configure tx_power
//
// power_level          - transmitting power in dBM of the radio
//                        according to Bluetooth 4.2 (-20 dBm to 10 dBm)
//
int ble_set_tx_power(TxPower_t power_level);

// configure advertisment interval
//
// advertising_iterval_ms - advertisment interval in milliseconds
//
int ble_set_advertisement_interval(uint16_t advertising_itv_ms);

#ifdef __cplusplus
}
#endif

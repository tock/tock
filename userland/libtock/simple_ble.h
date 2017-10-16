#pragma once

#include "tock.h"


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

#define BLE_DRIVER_NUMBER         0x30000
#define BLE_ADV_START_CMD         0
#define BLE_ADV_STOP_CMD          1
#define BLE_CFG_TX_POWER_CMD      2
#define BLE_CFG_ADV_ITV_CMD       3
#define BLE_ADV_CLEAR_DATA_CMD    4
#define BLE_SCAN_CMD              5
#define BLE_SCAN_SUB              0
#define BLE_CFG_ADV_ADDR_ALLOW    0x30
#define BLE_CFG_SCAN_BUF_ALLOW    0x31


typedef enum {
  GAP_FLAGS                              = 0x01, /* Flags, see enum below */
  GAP_INCOMPLETE_LIST_16BIT_SERVICE_IDS  = 0x02, /* Incomplete list of 16-bit Service IDs. */
  GAP_COMPLETE_LIST_16BIT_SERVICE_IDS    = 0x03, /* Complete list of 16-bit Service IDs. */
  GAP_INCOMPLETE_LIST_32BIT_SERVICE_IDS  = 0x04, /* Incomplete list of 32-bit Service IDs (not relevant for Bluetooth 4.0). */
  GAP_COMPLETE_LIST_32BIT_SERVICE_IDS    = 0x05, /* Complete list of 32-bit Service IDs (not relevant for Bluetooth 4.0). */
  GAP_INCOMPLETE_LIST_128BIT_SERVICE_IDS = 0x06, /* Incomplete list of 128-bit Service IDs. */
  GAP_COMPLETE_LIST_128BIT_SERVICE_IDS   = 0x07, /* Complete list of 128-bit Service IDs. */
  GAP_SHORTENED_LOCAL_NAME               = 0x08, /* Shortened Local Name. */
  GAP_COMPLETE_LOCAL_NAME                = 0x09, /* Complete Local Name. */
  GAP_TX_POWER_LEVEL                     = 0x0A, /* TX Power Level (in dBm). */
  GAP_DEVICE_ID                          = 0x10, /* Device ID. */
  GAP_SLAVE_CONNECTION_INTERVAL_RANGE    = 0x12, /* Slave Connection Interval Range. */
  GAP_LIST_128BIT_SOLICITATION_IDS       = 0x15, /* List of 128 bit service UUIDs the device is looking for. */
  GAP_SERVICE_DATA                       = 0x16, /* Service Data. */
  GAP_APPEARANCE                         = 0x19, /* Appearance */
  GAP_ADVERTISING_INTERVAL               = 0x1A, /* Advertising Interval. */
  GAP_MANUFACTURER_SPECIFIC_DATA         = 0xFF  /* Manufacturer Specific Data. */
} GapAdvertisementData_t;

enum {
  LE_LIMITED_DISCOVERABLE = 0x01, /* Peripheral device is discoverable for a limited period of time. */
  LE_GENERAL_DISCOVERABLE = 0x02, /* Peripheral device is discoverable at any moment. */
  BREDR_NOT_SUPPORTED     = 0x04, /* Peripheral device is LE only. */
  SIMULTANEOUS_LE_BREDR_C = 0x08, /* Not relevant - central mode only. */
  SIMULTANEOUS_LE_BREDR_H = 0x10  /* Not relevant - central mode only. */
} GapFlags_t;




/*******************************************************************************
 *   User-space API
 ******************************************************************************/

 // initialize advertisement (should be used before invoked ble_start_advertising)
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
// len                  - max_size (31 bytes)
// callback             - callback handler to call when an advertisement is received
//
// type signature of the callback handler:
// static void callback(__attribute__((unused)) int unused0,
//                      __attribute__((unused)) int unused1,
//                      __attribute__((unused)) int unused2,
//                      __attribute__((unused)) void* ud);
//
// The kernel will fill the array of bytes with the received advertisement
// it's then up to user-space application to determine what to do in the
// callback handler.
//
// Currently the number of received bytes in the advertisement is not
// passed back to user-space thus userspace must loop through the entire
// buffer.
//
// TODO: add functionlity in kernel to pass back the size of the received
// advertisement
//
int ble_start_passive_scan(uint8_t *data, uint8_t len, subscribe_cb callback);


#ifdef __cplusplus
}
#endif

#pragma once

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define ADV_DATA_MAX_SIZE 31
#define ADV_SIZE 39
#define ADV_A_SIZE 6

#define ADV_IND  0x00
#define ADV_DIRECT_IND  0x01
#define ADV_NON_CONN_IND  0x02
#define ADV_SCAN_IND  0x06 

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

typedef enum {
  LE_LIMITED_DISCOVERABLE = 0x01, /* Peripheral device is discoverable for a
                                     limited period of time. */
  LE_GENERAL_DISCOVERABLE =
      0x02, /* Peripheral device is discoverable at any moment. */
  BREDR_NOT_SUPPORTED = 0x04,     /* Peripheral device is LE only. */
  SIMULTANEOUS_LE_BREDR_C = 0x08, /* Not relevant - central mode only. */
  SIMULTANEOUS_LE_BREDR_H = 0x10  /* Not relevant - central mode only. */
} GapFlags_t;

typedef struct AdvData {
  uint8_t *buf;
  int offset;
  int capacity;
} AdvData_t;

AdvData_t gap_adv_data_new(uint8_t* buf, int len);

int gap_add_flags(AdvData_t *adv_data, uint8_t flags);
int gap_add_adv_data_field(AdvData_t *adv_data, GapAdvertisementData_t type,
                                    const uint8_t *data, uint8_t data_len);

// configure advertisement name
//
// device_name         - device named to be used in the advertisement
// size_b              - size of device in bytes
int gap_add_device_name(AdvData_t *adv_data, const uint8_t *device_name, uint8_t len);

// configure list of 16 bit uuids
//
// uuid16               - array of 16 bit uuids
// size_b               - size of uuid16 in bytes
int gap_add_service_uuid16(AdvData_t *adv_data, const uint16_t *uuid16, uint8_t len);

// configure service data
//
// uuid16               - 16 bit uuid to be associated with the data
// data                 - array of data in bytes
// size_b               - size of data in bytes
int gap_add_service_data(AdvData_t *adv_data, uint16_t uuid16, uint8_t *data, uint8_t data_len);

// configure manufacturer specific data
//
// data                 - array of data in bytes
// size_b               - size of data in bytes
int gap_add_manufacturer_specific_data(AdvData_t *adv_data, uint8_t *data, uint8_t size_b);

#ifdef __cplusplus
}
#endif

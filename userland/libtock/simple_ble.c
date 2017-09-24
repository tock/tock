/*
 * BLE setup functions
 */

#include "simple_ble.h"
#include "tock.h"
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>

#define MAX_SIZE 31


/*******************************************************************************
 *   INTERNAL BLE HELPER FUNCTION Prototypes
 *
 *   s_   - static (file scope)
 ******************************************************************************/

// internal helper function to configure flags in the advertisement
//
// flags     - a byte of flags to use in the advertisement
static int s_ble_configure_flags (uint8_t flags) {
  return allow(BLE_DRIVER_NUMBER, GAP_FLAGS, &flags, 1);
}

// internal helper to configure advertisement interval
//
// advertising_iterval_ms - advertisment intervall in millisecons
static int s_ble_configure_advertisement_interval (uint16_t advertising_itv_ms) {
  return command(BLE_DRIVER_NUMBER, BLE_CFG_ADV_ITV_CMD, advertising_itv_ms, 0);
}

// internal helper to configure gap data in the advertisement
//
// header - gap data header
// data   - buffer of data configure in the advertisement
// len    - length of data buffer
static int s_ble_configure_gap_data (GapAdvertisementData_t header, uint8_t *data, uint8_t data_len) {
  return allow(BLE_DRIVER_NUMBER, header, (void*)data, data_len);
}

/*******************************************************************************
 *   USER-SPACE BLE API
 ******************************************************************************/

int ble_initialize(uint16_t advertising_itv_ms, bool discoverable) {
  int err;

  // configure advertisement interval
  // if the interval is less than 20 or bigger than 10240 to kernel
  // will use default value
  err = s_ble_configure_advertisement_interval(advertising_itv_ms);
  if (err < TOCK_SUCCESS) return err;

  uint8_t flags = BREDR_NOT_SUPPORTED;

  // configure advertisement flags in the packet
  if (discoverable) {
    flags |= LE_GENERAL_DISCOVERABLE;
  }

  err = s_ble_configure_flags(flags);
  if (err < TOCK_SUCCESS) return err;

  return err;
}

int ble_start_advertising(void) {
  return command(BLE_DRIVER_NUMBER, BLE_ADV_START_CMD, 0, 0);
}

int ble_stop_advertising(void) {
  return command(BLE_DRIVER_NUMBER, BLE_ADV_STOP_CMD, 1, 0);
}

int ble_reset_advertisement(void) {
  return command(BLE_DRIVER_NUMBER, BLE_ADV_CLEAR_DATA_CMD, 1, 0);
}

int ble_advertise_name(uint8_t *device_name, uint8_t size_b) {
  if (device_name == NULL) {
    return TOCK_FAIL;
  }else {
    return s_ble_configure_gap_data(GAP_COMPLETE_LOCAL_NAME, device_name, size_b);
  }
}

int ble_advertise_uuid16(uint16_t *uuid16, uint8_t size_b) {
  if (uuid16 == NULL) {
    return TOCK_FAIL;
  }else {
    return s_ble_configure_gap_data(GAP_COMPLETE_LIST_16BIT_SERVICE_IDS, (uint8_t*)uuid16, size_b);
  }
}

int ble_advertise_service_data(uint16_t uuid16, uint8_t *data, uint8_t size_b) {
  // potential buffer overflow in libtock generate error
  if (size_b + 2 > MAX_SIZE || data == NULL) {
    return TOCK_FAIL;
  }else {
    uint8_t s_gap[MAX_SIZE];
    memset(s_gap, 0, MAX_SIZE);
    memcpy(s_gap, &uuid16, 2);
    memcpy(s_gap + 2, data, size_b);
    return s_ble_configure_gap_data(GAP_SERVICE_DATA, s_gap, size_b + 2);
  }
}

int ble_advertise_manufacturer_specific_data(uint8_t *data, uint8_t size_b) {
  if (data == NULL) {
    return TOCK_FAIL;
  }else {
    return s_ble_configure_gap_data(GAP_MANUFACTURER_SPECIFIC_DATA, data, size_b);
  }
}

int ble_start_passive_scan(uint8_t *data, uint8_t max_len, subscribe_cb callback) {
  if (data == NULL || callback == NULL) {
    return TOCK_FAIL;
  }else {
    int err;

    err = subscribe(BLE_DRIVER_NUMBER, BLE_SCAN_SUB, callback, NULL);
    if (err < TOCK_SUCCESS) return err;

    err = allow(BLE_DRIVER_NUMBER, BLE_CFG_SCAN_BUF_ALLOW, (void*)data, max_len);
    if (err < TOCK_SUCCESS) return err;

    return command(BLE_DRIVER_NUMBER, BLE_SCAN_CMD, 1, 0);
  }
}

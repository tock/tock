/*
 * BLE setup functions
 */

#include "simple_ble.h"
#include "tock.h"
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>

#define ADV_DATA_MAX_SIZE 31
#define ADV_SIZE 39

// Entire Advertisement Buffer
static unsigned char advertisement_buf[ADV_SIZE];
// AdvData buffer - to be used by `ADV_IND`, `ADV_NONCONN_IND` and `ADV_SCAN_IND`
static unsigned char adv_data[ADV_DATA_MAX_SIZE];
// Index in the AdvData buffer
static uint8_t adv_data_idx = 0;

/*******************************************************************************
 *   INTERNAL BLE HELPER FUNCTION Prototypes
 *
 *   s_   - static (file scope)
 ******************************************************************************/

// internal helper function to configure flags in the advertisement
// flags     - a byte of flags to use in the advertisement
static int s_ble_configure_flags(uint8_t flags) {
  return allow(BLE_DRIVER_NUMBER, GAP_FLAGS, &flags, 1);
}

// internal helper to configure gap data in the advertisement
//
// header - gap data header
// data   - buffer of data configure in the advertisement
// len    - length of data buffer
static int s_ble_configure_adv_data(GapAdvertisementData_t type,
                                    uint8_t *data, uint8_t data_len) {
  // make room for gap data header: length and gap_type
  uint8_t new_length = 2 + data_len + adv_data_idx;
  if (new_length > ADV_DATA_MAX_SIZE) {
    return TOCK_FAIL;
  } else {
    adv_data[adv_data_idx]     = data_len + 1;
    adv_data[adv_data_idx + 1] = type;
    memcpy(&adv_data[adv_data_idx + 2], data, data_len);
    adv_data_idx = new_length;
    return TOCK_SUCCESS;
  }
}

// internal helper to request the kernel to generate a random advertisemen address
static int s_request_advertisement_address(void) {
  return command(BLE_DRIVER_NUMBER, BLE_REQ_ADV_ADDR, 0, 0);
}

static int s_initialize_advertisement_buffer(void) {
  return allow(BLE_DRIVER_NUMBER, BLE_CFG_ADV_BUF_ALLOW,
               (void *)advertisement_buf, ADV_SIZE);
}

/*******************************************************************************
 *   USER-SPACE BLE API
 ******************************************************************************/

int ble_initialize(uint16_t advertising_itv_ms, bool discoverable) {
  int err;

  adv_data_idx = 0;
  memset(adv_data, 0, ADV_DATA_MAX_SIZE);

  err = s_initialize_advertisement_buffer();

  err = s_request_advertisement_address();
  if (err < TOCK_SUCCESS)
    return err;

  // configure advertisement interval
  // if the interval is less than 20 or bigger than 10240 to kernel
  // will use default value
  err = ble_set_advertisement_interval(advertising_itv_ms);
  if (err < TOCK_SUCCESS)
    return err;

  uint8_t flags = BREDR_NOT_SUPPORTED;

  // configure advertisement flags in the packet
  if (discoverable) {
    flags |= LE_GENERAL_DISCOVERABLE;
  }

  return s_ble_configure_flags(flags);
}

int ble_start_advertising(void) {
  int err = allow(BLE_DRIVER_NUMBER, BLE_CFG_GAP_BUF_ALLOW, (void *)adv_data, adv_data_idx);
  if (err < TOCK_SUCCESS)
    return err;

  return command(BLE_DRIVER_NUMBER, BLE_ADV_START_CMD, 0, 0);

}

int ble_stop_advertising(void) {
  return command(BLE_DRIVER_NUMBER, BLE_ADV_STOP_CMD, 1, 0);
}

int ble_reset_advertisement(void) {
  int err = command(BLE_DRIVER_NUMBER, BLE_ADV_CLEAR_DATA_CMD, 1, 0);
  if (err < TOCK_SUCCESS)
    return err;
  else {
    adv_data_idx = 0;
    memset(adv_data, 0, ADV_DATA_MAX_SIZE);
    return TOCK_SUCCESS;
  }
}

int ble_advertise_name(uint8_t *device_name, uint8_t len) {
  if (device_name == NULL) {
    return TOCK_FAIL;
  } else {
    return s_ble_configure_adv_data(GAP_COMPLETE_LOCAL_NAME, device_name,
                                    len);
  }
}

int ble_advertise_uuid16(uint16_t *uuid16, uint8_t len) {
  if (uuid16 == NULL) {
    return TOCK_FAIL;
  } else {
    return s_ble_configure_adv_data(GAP_COMPLETE_LIST_16BIT_SERVICE_IDS,
                                    (uint8_t *)uuid16, len);
  }
}

int ble_advertise_service_data(uint16_t uuid16, uint8_t *data, uint8_t data_len) {
  uint8_t pdu_size = data_len + 2;
  // potential buffer overflow in libtock generate error
  if (pdu_size > ADV_DATA_MAX_SIZE || data == NULL) {
    return TOCK_FAIL;
  } else {
    uint8_t pdu[ADV_DATA_MAX_SIZE];
    memcpy(pdu, &uuid16, 2);
    memcpy(pdu + 2, data, data_len);
    return s_ble_configure_adv_data(GAP_SERVICE_DATA, pdu, pdu_size);
  }
}

int ble_advertise_manufacturer_specific_data(uint8_t *data, uint8_t size_b) {
  if (data == NULL) {
    return TOCK_FAIL;
  } else {
    return s_ble_configure_adv_data(GAP_MANUFACTURER_SPECIFIC_DATA, data,
                                    size_b);
  }
}

int ble_start_passive_scan(uint8_t *data, uint8_t max_len,
                           subscribe_cb callback) {
  if (data == NULL || callback == NULL) {
    return TOCK_FAIL;
  } else {
    int err;

    err = subscribe(BLE_DRIVER_NUMBER, BLE_SCAN_SUB, callback, NULL);
    if (err < TOCK_SUCCESS)
      return err;

    err =
      allow(BLE_DRIVER_NUMBER, BLE_CFG_SCAN_BUF_ALLOW, (void *)data, max_len);
    if (err < TOCK_SUCCESS)
      return err;

    return command(BLE_DRIVER_NUMBER, BLE_SCAN_CMD, 1, 0);
  }
}

int ble_stop_passive_scan(void) {
  return command(BLE_DRIVER_NUMBER, BLE_ADV_STOP_CMD, 1, 0);
}

int ble_set_tx_power(TxPower_t power_level) {
  return command(BLE_DRIVER_NUMBER, BLE_CFG_TX_POWER_CMD, power_level, 0);
}

int ble_set_advertisement_interval(uint16_t advertising_itv_ms) {
  return command(BLE_DRIVER_NUMBER, BLE_CFG_ADV_ITV_CMD, advertising_itv_ms, 0);
}

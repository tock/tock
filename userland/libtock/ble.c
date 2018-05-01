/*
 * BLE setup functions
 */

#include "ble.h"
#include "tock.h"
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>

int ble_start_advertising(int pdu_type, uint8_t* advd, int len, uint16_t interval) {
  int err = allow(BLE_DRIVER_NUMBER, 0, advd, len);
  if (err < TOCK_SUCCESS)
    return err;

  return command(BLE_DRIVER_NUMBER, BLE_ADV_START_CMD, pdu_type, interval);
}

int ble_stop_advertising(void) {
  return command(BLE_DRIVER_NUMBER, BLE_ADV_STOP_CMD, 1, 0);
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

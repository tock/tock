#include "ble.h"
#include <stdio.h>
#include <string.h>

int ble_adv_set_txpower(BLE_TX_Power_t power) {
  return command(DRIVER_RADIO, BLE_CFG_TX_POWER, power, 0);
}

int ble_adv_set_interval(uint16_t interval) {
  return command(DRIVER_RADIO, BLE_CFG_ADV_INTERVAL, interval, 0);
}

int ble_adv_data(uint8_t type, const unsigned char *data, uint8_t len) {
  return allow(DRIVER_RADIO, type, (void*)data, len);
}

int ble_adv_clear_data(void){
  return command(DRIVER_RADIO, BLE_ADV_CLEAR_DATA, 1, 0);
}

int ble_adv_start(BLE_Gap_Mode_t mode) {
  return command(DRIVER_RADIO, BLE_ADV_START, mode, 0);
}

int ble_adv_scan(const unsigned char *data, uint8_t len, subscribe_cb callback) {
  int err;

  err = subscribe(DRIVER_RADIO, BLE_SCAN_CALLBACK, callback, NULL);
  if (err < TOCK_SUCCESS) return err;

  err = allow(DRIVER_RADIO, BLE_CFG_SCAN_BUF, (void*)data, len);
  if (err < TOCK_SUCCESS) return err;

  return command(DRIVER_RADIO, BLE_SCAN, 1, 0);
}

int ble_adv_stop(void) {
  return command(DRIVER_RADIO, BLE_ADV_STOP, 1, 0);
}

int ble_adv_set_address(const unsigned char *data, uint8_t len) {
  return allow(DRIVER_RADIO, BLE_CFG_ADV_ADDR, (void*)data, len);
}

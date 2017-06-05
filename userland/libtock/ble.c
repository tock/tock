#include "ble.h"
#include <stdio.h>
#include <string.h>

int ble_adv_set_txpower(TX_Power_t power) {
  return command(DRIVER_RADIO, CFG_TX_POWER, power);
}

int ble_adv_set_interval(uint16_t interval) {
  return command(DRIVER_RADIO, CFG_ADV_INTERVAL, interval);
}

int ble_adv_data(uint8_t type, uint8_t len, const unsigned char *data) {
  return allow(DRIVER_RADIO, type, (void*)data, len);
}

int ble_adv_clear_data(void){
  return command(DRIVER_RADIO,BLE_ADV_CLEAR_DATA, 1);
}

int ble_adv_start(void){
  return command(DRIVER_RADIO, BLE_ADV_START, 1);
}

int ble_adv_stop(void) {
  return command(DRIVER_RADIO, BLE_ADV_STOP, 1);
}

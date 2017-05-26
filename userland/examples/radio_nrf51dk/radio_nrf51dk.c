#include "radio_nrf51dk.h"
#include <stdio.h>
#include <string.h>

int subscribe_rx(subscribe_cb callback, void *ud) {
  return subscribe(DRIVER_RADIO, RX, callback, ud);
}

int subscribe_tx(subscribe_cb callback, void *ud) {
  return subscribe(DRIVER_RADIO, TX, callback, ud);
}

int tx_data(const char* data, unsigned char len) {
  int err = allow(DRIVER_RADIO, TX, (void*)data, len);
  if (err < 0)  {
    return err;
  }
  return command(DRIVER_RADIO, TX, 16);
}

int start_ble_advertisement(const char* name, const char *data){
  // empty string used pre-configured name
  int err = allow(DRIVER_RADIO, SET_NAME, (void*)name, strlen(name));
  if (err < 0){
    printf("Warning invalid name kernel configures default name");
  }
  
  err = allow(DRIVER_RADIO, SET_DATA, (void*)data, strlen(data));
  if (err < 0){
    printf("Warning invalid data kernel do not use data");
  }
  // len not used in command i.e. 1
  return command(DRIVER_RADIO, BLE_ADV_START, 1);
}

int stop_ble_advertisement(void){
  return command(DRIVER_RADIO,BLE_ADV_STOP, 1);
}

int rx_data(const char* data, unsigned char len) {
  int err = allow(DRIVER_RADIO, RX, (void*)data, len);
  if (err < 0)  {
    return err;
  }
  return command(DRIVER_RADIO, RX, 16);
}

int read_data(const char* packet, subscribe_cb callback, unsigned char len) {
  int err = allow(DRIVER_RADIO, RX, (void*)packet, len);
  if ( err < 0)  {
    return err;
  }
  err = subscribe(DRIVER_RADIO, RX, callback, NULL);
  if (err < 0 ){
    return err;
  }
  return command(DRIVER_RADIO, RX, 16);
}

int set_channel(int ch_num) {
  return command(DRIVER_RADIO, CH, ch_num);
}

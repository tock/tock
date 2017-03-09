#include "radio_nrf51dk.h"

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
  subscribe(DRIVER_RADIO, RX, callback, NULL);
  command(DRIVER_RADIO, RX, 16);
}

int set_channel(int ch_num) {
  return command(DRIVER_RADIO, CH, ch_num);
}

#include "nrf51_serialization.h"

int nrf51_serialization_subscribe (subscribe_cb cb) {
  // get some callback love
  return subscribe(5, 0, cb, NULL);
}

int nrf51_serialization_setup_rx_buffer (char* rx, int rx_len) {
  // Pass the RX buffer for the UART module to use.
  return allow(5, 0, rx, rx_len);
}

int nrf51_serialization_write (char* tx, int tx_len) {
  int ret;

  // Pass in the TX buffer.
  ret = allow(5, 1, tx, tx_len);
  if (ret < 0) return ret;

  // Do the write!!!!!
  ret = command(5, 1, 0);
  return ret;
}

int nrf51_wakeup (void) {
  return command(5, 9001, 0);
}


#pragma once

#include "tock.h"

#ifdef __cplusplus
extern "C" {
#endif

#define DRIVER_NUM_NRF_SERIALIZATION 0x80004

// Give the BLE Serialization / UART layer a callback to call when
// a packet is received and when a TX is finished.
__attribute__ ((warn_unused_result))
int nrf51_serialization_subscribe (subscribe_cb cb);

// Pass a buffer for the driver to write received UART bytes to.
__attribute__ ((warn_unused_result))
int nrf51_serialization_setup_rx_buffer (char* rx, int rx_len);

// Write a packet to the BLE Serialization connectivity processor.
__attribute__ ((warn_unused_result))
int nrf51_serialization_write (char* tx, int tx_len);

#ifdef __cplusplus
}
#endif

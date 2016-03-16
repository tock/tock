#ifndef _FIRESTORM_H
#define _FIRESTORM_H

#include <unistd.h>
#include "tock.h"
#include "gpio.h"

// Pin definitions
#define LED_0 PC10

#ifdef __cplusplus
extern "C" {
#endif

enum firestorm_cb_type {
  PUTSTR,
  READTMP,
  ASYNC,
  SPIBUF,
  GPIO,
  READLIGHT,
  DELAY,
};

void putstr(const char* str);
void putnstr(const char* str, size_t len);
void putnstr_async(const char* str, size_t len, subscribe_cb cb, void* userdata);

int timer_subscribe(subscribe_cb cb, void *userdata);
int timer_start_repeating(uint32_t interval_ms);
int timer_oneshot(uint32_t interval_ms);

void delay_ms(uint32_t ms);

int spi_read_write(const char* write, char* read, size_t  len, subscribe_cb cb);

// Output pins on Firestorm
// From https://github.com/SoftwareDefinedBuildings/storm/blob/master/docs/_posts/2014-10-02-pins.md
//  combined with the eagle files for Firestorm https://github.com/helena-project/firestorm
enum GPIO_Pin_enum{
  PC10=0,
  PA16,
  PA12,
  PC09,
  PA10,
  PA11,
  PA19,
  PA13,
  PA17,
};
#define LED_0     PC10
#define P2        PA16
#define P3        PA12
#define P5        PA10
#define P6        PA11
#define P7        PA19
#define P8        PA13
#define STORM_INT PA17

// Give the BLE Serialization / UART layer a callback to call when
// a packet is received and when a TX is finished.
void nrf51822_serialization_subscribe (subscribe_cb cb);

// Pass a buffer for the driver to write received UART bytes to.
void nrf51822_serialization_setup_rx_buffer (char* rx, int rx_len);

// Write a packet to the BLE Serialization connectivity processor.
void nrf51822_serialization_write (char* tx, int tx_len);

#ifdef __cplusplus
}
#endif

#endif // _FIRESTORM_H

#ifndef _FIRESTORM_H
#define _FIRESTORM_H

#include <stdbool.h>
#include <unistd.h>
#include "tock.h"
#include "gpio.h"

// Pin definitions
#define LED_0 PC10

#ifdef __cplusplus
extern "C" {
#endif

/*
 * Sets the callback for timers
 *
 * Side-effects: cancels any existing/outstanding timers
 */
int timer_subscribe(subscribe_cb cb, void *userdata);

/*
 * Starts a repeating timer
 *
 * interval_ms - the interval for the timer in milliseconds
 *
 * Side-effects: cancels any existing/outstanding timers
 */
int timer_start_repeating(uint32_t interval_ms);

/*
 * Starts a oneshot timer
 *
 * interval_ms - the interval for the timer in milliseconds
 *
 * Side-effects: cancels any existing/outstanding timers
 */
int timer_oneshot(uint32_t interval_ms);

int timer_stop();

/*
 * Blocks for the given amount of time in millisecond.
 *
 * This is a wrapper around the `timer` interface, so calling this will cancel
 * any outstanding timers as well as replace the timer callback.
 */
void delay_ms(uint32_t ms);

/* SPI system calls */
int spi_init();
/* All SPI operations depend on which peripheral is
 * active, determined by set_chip_select. Configuration
 * of a peripheral is persistent; e.g., setting the rate R
 * for peripheral 3, then switching to peripheral 2,
 * peripheral 2 will not necessarily have rate R. Then
 * back to peripheral 3, it still has rate R.*/
int spi_set_chip_select(unsigned char cs);
int spi_get_chip_select();
int spi_set_rate(int rate);
int spi_get_rate();
int spi_set_phase(bool phase);
int spi_get_phase();
int spi_set_polarity(bool pol);
int spi_get_polarity();
int spi_hold_low();
int spi_release_low();
int spi_write_byte(unsigned char byte);
int spi_write(const char* write, size_t len, subscribe_cb cb, bool* cond);
int spi_read_write(const char* write, char* read, size_t len, subscribe_cb cb, bool* cond);

int spi_write_sync(const char* write, size_t len);
int spi_read_write_sync(const char* write, char* read, size_t len);

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
  PC14,
  PC15,
  PA20,
};
#define LED_0     PC10
#define P2        PA16
#define P3        PA12
#define P5        PA10
#define P6        PA11
#define P7        PA19
#define P8        PA13
#define STORM_INT PA17
#define RADIO_SLP PC14
#define RADIO_RST PC15
#define RADIO_IRQ PA20

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

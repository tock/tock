#include <stdbool.h>

#include "led.h"
#include "spi_slave.h"
#include "gpio.h"

#define GPIO_PIN 31

#define BUF_SIZE 200
char rbuf[BUF_SIZE];
char wbuf[BUF_SIZE];
bool toggle = true;
bool error = false;

static void write_cb(__attribute__ ((unused)) int arg0,
              __attribute__ ((unused)) int arg2,
              __attribute__ ((unused)) int arg3,
              __attribute__ ((unused)) void* userdata) {
    led_toggle(0);
    if (toggle) {
        spi_slave_read_write(rbuf, wbuf, BUF_SIZE, write_cb, NULL);
    } else {
        spi_slave_read_write(wbuf, rbuf, BUF_SIZE, write_cb, NULL);
    }
    toggle = !toggle;
}

static void selected_cb(__attribute__ ((unused)) int arg0,
              __attribute__ ((unused)) int arg2,
              __attribute__ ((unused)) int arg3,
              __attribute__ ((unused)) void* userdata) {
    if (toggle) {
      led_on(0);
      gpio_clear(GPIO_PIN);
    } else {
      led_off(0);
      gpio_set(GPIO_PIN);
    }
    toggle = !toggle;
}

// This function can operate in one of two modes. Either
// a periodic timer triggers an SPI operation, or SPI
// operations are performed back-to-back (callback issues
// the next one.) The periodic one writes 6 byte messages,
// the back-to-back writes a 10 byte message, followed by
// 6 byte ones.
//
// In both cases, the calls alternate on which of two
// buffers is used as the write buffer. The first call
// uses the buffer initialized to 0..199. The
// 2n calls use the buffer initialized to 0.
//
// If you use back-to-back operations, the calls
// both read and write. Periodic operations only
// write. Therefore, if you set SPI to loopback
// and use back-to-back // loopback, then the read buffer
// on the first call will read in the data written.  As a
// result, you can check if reads work properly: all writes
// will be 0..n rather than all 0s.

int main(void) {
  gpio_enable_output(GPIO_PIN);

  int i;
  for (i = 0; i < 200; i++) {
    wbuf[i] = i;
  }
  spi_slave_init();
  // All of these calls should fail (return ENOSUPPORT)
  if (spi_slave_set_chip_select(1) >= 0) {
    error = true;
  }
  if (spi_slave_get_chip_select() != 0) {
    error = true;
  }
  if (spi_slave_set_rate(10000) >= 0) {
    error = true;
  }
  if (spi_slave_get_rate() >= 0) {
    error = true;
  }
  if (spi_slave_hold_low() >= 0) {
    error = true;
  }
  if (spi_slave_release_low() >= 0) {
    error = true;
  }

  // If any of the calls succeeded, return -1 and
  // set the LED and GPIO pin
  if (error == true) {
    led_on(0);
    gpio_set(GPIO_PIN);
    return -1;
  }

  spi_slave_set_polarity(false);
  spi_slave_set_phase(false);
  spi_slave_read_write(wbuf, rbuf, BUF_SIZE, write_cb, NULL);
  spi_slave_chip_selected(selected_cb);
}

#include <stdbool.h>
#include <stdio.h>

#include <led.h>
#include <spi.h>

#include <multispi.h>
#include <timer.h>

#define BUF_SIZE 200
char rbuf[BUF_SIZE];
char wbuf[BUF_SIZE];
const char *msg = "Hello World!\r\n";
bool toggle = true;

static void write_cb(__attribute__ ((unused)) int arg0,
                     __attribute__ ((unused)) int arg2,
                     __attribute__ ((unused)) int arg3,
                     __attribute__ ((unused)) void* userdata) {
  led_toggle(0);
  delay_ms(200);
  select_spi_bus(0);
  spi_read_write_sync(wbuf, rbuf, BUF_SIZE);

  select_spi_bus(1);
  spi_read_write(wbuf, rbuf, BUF_SIZE, write_cb, NULL);
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
  int i;
  for (i = 0; i < 200; i++) {
    wbuf[i] = msg[i % 15];
  }
  select_spi_bus(0);
  spi_set_chip_select(0);
  spi_set_rate(10e6);
  spi_set_polarity(true);
  spi_set_phase(true);

  spi_read_write_sync(wbuf, rbuf, BUF_SIZE);

  select_spi_bus(1);
  spi_set_chip_select(0);
  spi_set_rate(20e6);
  spi_set_polarity(false);
  spi_set_phase(false);

  spi_read_write(wbuf, rbuf, BUF_SIZE, write_cb, NULL);
}

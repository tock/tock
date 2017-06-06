#include <stdbool.h>
#include <string.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

#include "led.h"
#include "spi_slave.h"

#define BUF_SIZE 16
char rbuf[BUF_SIZE];
char wbuf[BUF_SIZE];
char zbuf[BUF_SIZE];
char ibuf[BUF_SIZE];
bool toggle = true;

// Check for buffer equality, set the LED
// if the buffers are *not* equal.
static void buffer_eq (char *buf1, char *buf2) {
  int i;
  for (i = 0; i < BUF_SIZE; i++) {
    if (buf1[i] != buf2[i]) {
      led_on(0);
      return;
    }
  }
}

// Note that this assumes the behavior of the master; that it passes us
// a buffer with increasing i values, and on the next operation, will
// pass us back the buffer we sent it. This is implemented in the
// spi_master_transfer example.
static void write_cb(__attribute__ ((unused)) int arg0,
              __attribute__ ((unused)) int arg2,
              __attribute__ ((unused)) int arg3,
              __attribute__ ((unused)) void* userdata) {
  printf("In write callback\n");
  if (toggle) {
      // The transfer before the one that just completed (either the
      // first transfer or a subsequent transfer), the master sent us
      // the buffer with increasing numbers.
      buffer_eq (rbuf, ibuf);
      spi_slave_read_write(rbuf, wbuf, BUF_SIZE, write_cb, NULL);
  } else {
      // The transfer before this one, we should have passed the master
      // the zero buffer back.
      buffer_eq (wbuf, zbuf);
      spi_slave_read_write(wbuf, rbuf, BUF_SIZE, write_cb, NULL);
  }
  toggle = !toggle;
  printf("In write callback, before return\n");
}

static void selected_cb(__attribute__ ((unused)) int arg0,
              __attribute__ ((unused)) int arg2,
              __attribute__ ((unused)) int arg3,
              __attribute__ ((unused)) void* userdata) {
  printf("In subscribe callback\n");
}

// This function first initializes the write buffer to all zeroes. We
// then wait until the master begins the transfer, and we then switch
// buffers, so that the data the master sends is passed between the 
// master and the slave. Further, after we receive the buffer with data, we
// check to make sure we received the correct values. If not, we enable the LED
// and never disable it.
int main(void) {
  int i;
  for (i = 0; i < BUF_SIZE; i++) {
    wbuf[i] = 0;
    zbuf[i] = 0;
    ibuf[i] = i;
  }

  printf("Before init!\n");
  spi_slave_init();
  printf("After init!\n");


  spi_slave_set_polarity(false);
  spi_slave_set_phase(false);
  printf("After init pt 2!\n");
  // We write wbuf, read rbuf here
  printf("Asynch call\n");
  int err = spi_slave_read_write(wbuf, rbuf, BUF_SIZE, write_cb, NULL);
  printf("After asynch call: %d\n", err);

  spi_slave_chip_selected(selected_cb, NULL);
}

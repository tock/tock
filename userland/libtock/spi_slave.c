#include <spi_slave.h>

#define SPI_SLAVE 25

int spi_slave_get_chip_select(void) {
  return command(SPI_SLAVE, 2, 0, 0);
}
int spi_slave_set_phase(bool phase) {
  return command(SPI_SLAVE, 3, (unsigned char)phase, 0);
}
int spi_slave_get_phase(void) {
  return command(SPI_SLAVE, 4, 0, 0);
}
int spi_slave_set_polarity(bool pol) {
  return command(SPI_SLAVE, 5, (unsigned char)pol, 0);
}
int spi_slave_get_polarity(void) {
  return command(SPI_SLAVE, 6, 0, 0);
}

/* This registers a callback for when the slave is selected. */
int spi_slave_chip_selected(subscribe_cb cb, bool* cond) {
  return subscribe(SPI_SLAVE, 1, cb, cond);
}

int spi_slave_read_buf(const char* str, size_t len) {
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wcast-qual"
  // in lieu of RO allow
  void* buf = (void*) str;
#pragma GCC diagnostic pop
  return allow(SPI_SLAVE, 0, buf, len);
}

static void spi_slave_cb(__attribute__ ((unused)) int unused0,
                         __attribute__ ((unused)) int unused1,
                         __attribute__ ((unused)) int unused2,
                         __attribute__ ((unused)) void* ud) {
  *((bool*)ud) = true;
}

int spi_slave_write(const char* str,
                    size_t len,
                    subscribe_cb cb, bool* cond) {
  int err;
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wcast-qual"
  // in lieu of RO allow
  void* buf = (void*) str;
#pragma GCC diagnostic pop
  err = allow(SPI_SLAVE, 1, buf, len);
  if (err < 0 ) {
    return err;
  }
  err = subscribe(SPI_SLAVE, 0, cb, cond);
  if (err < 0 ) {
    return err;
  }
  return command(SPI_SLAVE, 1, len, 0);
}

int spi_slave_read_write(const char* write,
                         char* read,
                         size_t len,
                         subscribe_cb cb, bool* cond) {

  int err = allow(SPI_SLAVE, 0, (void*)read, len);
  if (err < 0) {
    return err;
  }
  return spi_slave_write(write, len, cb, cond);
}

int spi_slave_write_sync(const char* write,
                         size_t len) {
  bool cond = false;
  spi_slave_write(write, len, spi_slave_cb, &cond);
  yield_for(&cond);
  return 0;
}

int spi_slave_read_write_sync(const char* write,
                              char* read,
                              size_t len) {
  bool cond = false;
  int err   = spi_slave_read_write(write, read, len, spi_slave_cb, &cond);
  if (err < 0) {
    return err;
  }
  yield_for(&cond);
  return 0;
}

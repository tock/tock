#include "spi.h"

__attribute__((const)) int spi_init(void) {return 0;}
int spi_set_chip_select(unsigned char cs) {return command(4, 3, cs);}
int spi_get_chip_select(void)             {return command(4, 4, 0);}
int spi_set_rate(int rate)                {return command(4, 5, rate);}
int spi_get_rate(void)                    {return command(4, 6, 0);}
int spi_set_phase(bool phase)             {return command(4, 7, (unsigned char)phase);}
int spi_get_phase(void)                   {return command(4, 8, 0);}
int spi_set_polarity(bool pol)            {return command(4, 9, (unsigned char)pol);}
int spi_get_polarity(void)                {return command(4, 10, 0);}
int spi_hold_low(void)                    {return command(4, 11, 0);}
int spi_release_low(void)                 {return command(4, 12, 0);}

int spi_write_byte(unsigned char byte) {
  return command(4, 1, byte);
}

int spi_read_buf(const char* str, size_t len) {
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wcast-qual"
  // in lieu of RO allow
  void* buf = (void*) str;
#pragma GCC diagnostic pop
  return allow(4, 0, buf, len);
}

static void spi_cb( __attribute__ ((unused)) int unused0,
                    __attribute__ ((unused)) int unused1,
                    __attribute__ ((unused)) int unused2,
                    __attribute__ ((unused)) void* ud) {
  *((bool*)ud) = true;
}

int spi_write(const char* str,
              size_t len,
              subscribe_cb cb, bool* cond) {
  int err;
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wcast-qual"
  // in lieu of RO allow
  void* buf = (void*) str;
#pragma GCC diagnostic pop
  err = allow(4, 1, buf, len);
  if (err < 0 ) {
    return err;
  }
  err = subscribe(4, 0, cb, cond);
  if (err < 0 ) {
    return err;
  }
  return command(4, 2, len);
}

int spi_read_write(const char* write,
                   char* read,
                   size_t  len,
                   subscribe_cb cb, bool* cond) {

  int err = allow(4, 0, (void*)read, len);
  if (err < 0) {
    return err;
  }
  return spi_write(write, len, cb, cond);
}

int spi_write_sync(const char* write,
                   size_t  len) {
  bool cond = false;
  spi_write(write, len, spi_cb, &cond);
  yield_for(&cond);
  return 0;
}

int spi_read_write_sync(const char* write,
                        char* read,
                        size_t  len) {
  bool cond = false;
  int err = spi_read_write(write, read, len, spi_cb, &cond);
  if (err < 0) {
    return err;
  }
  yield_for(&cond);
  return 0;
}

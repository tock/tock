#pragma once

#include <tock.h>

#ifdef __cplusplus
extern "C" {
#endif

/* SPI system calls */
int spi_slave_init(void);
/* Set chip select is not supported in slave mode. */
int spi_slave_set_chip_select(unsigned char cs);
/* Get chip select always returns 0 in slave mode. */
int spi_slave_get_chip_select(void);

/* Set rate and get rate are not supported in slave mode. */
int spi_slave_set_rate(int rate);
int spi_slave_get_rate(void);

  /* false means sample on a leading (low to high) clock edge
   * true means sample on a trailing (high to low) clock edge */
int spi_slave_set_phase(bool phase);
int spi_slave_get_phase(void);

  /* false means an idle clock is low
   * true means an idle clock is high. */
int spi_slave_set_polarity(bool pol);
int spi_slave_get_polarity(void);

/* Hold low and release low are not supported in slave mode. */
int spi_slave_hold_low(void);
int spi_slave_release_low(void);

/* Write byte is no longer supported. */
int spi_slave_write_byte(unsigned char byte);

/* This registers a callback for when the slave is selected. */
int spi_slave_chip_selected(subscribe_cb cb);

int spi_slave_read_buf(const char* str, size_t len);
int spi_slave_write(const char* str, size_t len, subscribe_cb cb, bool* cond);
int spi_slave_read_write(const char* write, char* read, size_t len, subscribe_cb cb, bool* cond);

int spi_slave_write_sync(const char* write, size_t len);
int spi_slave_read_write_sync(const char* write, char* read, size_t len);

#ifdef __cplusplus
}
#endif

#pragma once

#include "tock.h"

#ifdef __cplusplus
extern "C" {
#endif

/* SPI system calls */
/* Get chip select always returns 0 in slave mode. */
int spi_slave_get_chip_select(void);

  /* false means sample on a leading (low to high) clock edge
   * true means sample on a trailing (high to low) clock edge */
int spi_slave_set_phase(bool phase);
int spi_slave_get_phase(void);

  /* false means an idle clock is low
   * true means an idle clock is high. */
int spi_slave_set_polarity(bool pol);
int spi_slave_get_polarity(void);

/* This registers a callback for when the slave is selected. */
int spi_slave_chip_selected(subscribe_cb cb, bool* cond);

int spi_slave_read_buf(const char* str, size_t len);
int spi_slave_write(const char* str, size_t len, subscribe_cb cb, bool* cond);
int spi_slave_read_write(const char* write, char* read, size_t len, subscribe_cb cb, bool* cond);

int spi_slave_write_sync(const char* write, size_t len);
int spi_slave_read_write_sync(const char* write, char* read, size_t len);

#ifdef __cplusplus
}
#endif

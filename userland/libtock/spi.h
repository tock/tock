#ifndef _SPI_H
#define _SPI_H

#include <tock.h>

#ifdef __cplusplus
extern "C" {
#endif

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
int spi_write(const char* str, size_t len, subscribe_cb cb, bool* cond);
int spi_read_write(const char* write, char* read, size_t len, subscribe_cb cb, bool* cond);

int spi_write_sync(const char* write, size_t len);
int spi_read_write_sync(const char* write, char* read, size_t len);

#ifdef __cplusplus
}
#endif

#endif // _SPI_H

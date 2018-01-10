#pragma once

#ifdef __cplusplus
extern "C" {
#endif

/**
 * Use the given SPI module.
 */
int select_spi_bus(int spi_num);

#ifdef __cplusplus
}
#endif

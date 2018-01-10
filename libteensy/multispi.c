#include "tock.h"
#include "spi.h"
#include "multispi.h"

int select_spi_bus(int spi_num) {
    return command(DRIVER_NUM_SPI, 11, spi_num, 0);
}

#include <string.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <stdint.h>
#include <stdbool.h>

#include <tock.h>
#include <console.h>
#include <tsl2561.h>

int main (void) {
  printf("[TSL2561] Test\n");

  // Start a light measurement
  int lux = tsl2561_get_lux_sync();

  // Print the lux value
  printf("\tValue(%d lux) [0x%X]\n\n", lux, lux);
}

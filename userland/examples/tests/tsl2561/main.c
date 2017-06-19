#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include <console.h>
#include <tock.h>
#include <tsl2561.h>

int main (void) {
  printf("[TSL2561] Test\n");

  // Start a light measurement
  int lux = tsl2561_get_lux_sync();

  // Print the lux value
  printf("\tValue(%d lux) [0x%X]\n\n", lux, lux);
}

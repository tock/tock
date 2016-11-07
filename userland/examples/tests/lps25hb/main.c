#include <string.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <stdint.h>
#include <stdbool.h>

#include "tock.h"
#include "console.h"
#include "lps25hb.h"

int main () {
  printf("[LPS25HB] Test\n");

  // Start a pressure measurement
  int pressure = lps25hb_get_pressure_sync();

  // Print the pressure value
  printf("\tValue(%d ubar) [0x%X]\n\n", pressure, pressure);
}

#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include <console.h>
#include <si7021.h>
#include <tock.h>

int main (void) {
  printf("[SI7021] Test App\n");

  // Start a measurement
  int temp;
  unsigned humi;
  si7021_get_temperature_humidity_sync(&temp, &humi);

  // Print the value
  printf("\tTemp(%d 1/100 degrees C) [0x%X]\n", temp, (unsigned) temp);
  printf("\tHumi(%u 0.01%%) [0x%X]\n\n", humi, humi);
}

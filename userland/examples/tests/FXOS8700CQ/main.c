#include <string.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <stdint.h>
#include <stdbool.h>
#include <math.h>
#include <timer.h>

#include <tock.h>
#include <console.h>

#include "FXOS8700CQ.h"

const double g = -9.8;

// Step counter app
// TODO get sqrt working
int main(void) {
  printf("Step counter init\n");
  unsigned num_measurements = 100;
  double accel_mags[num_measurements];

  for (unsigned ii = 0; ii < num_measurements; ii++) {
    unsigned accel_mag = FXOS8700CQ_read_accel_mag();
    printf("accel square = %u\n", accel_mag);
    printf("********************\n");
    accel_mags[ii] = accel_mag + g;
    delay_ms(2000);
  }

  unsigned steps = 0;
  for (unsigned ii = 0; ii < num_measurements - 1; ii++) {
    if (accel_mags[ii] < 0 && accel_mags[ii + 1] > 0) {
      // step occurred
      steps ++;
    }
  }

  printf("%u steps occurred.\n", steps);
}

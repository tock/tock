#include <stdio.h>
#include <math.h>

#include <led.h>
#include <FXOS8700CQ.h>

int main(void) {
  int x, y, z;

  // Choose the LED to use. We want green (which is usually
  // second in RGB), but will take anything.
  int led = 0;
  int num_leds = led_count();
  if (num_leds > 1) led = 1;

  while(1) {
    FXOS8700CQ_read_magenetometer_sync(&x, &y, &z);
    printf("x: %d, y: %d, z: %d\n", x, y, z);

    // Compute the X-Y angle of the board.
    double angle = atan2((double) y, (double) x);
    if (y > 0) {
      angle = 90 - angle * (180 / M_PI);
    } else {
      angle = 270 - angle * (180 / M_PI);
    }

    // Turn the LED on if the board is pointing in a certain range.
    if (angle > 50 && angle < 310) {
      led_off(led);
    } else {
      led_on(led);
    }
  }

  return 0;
}

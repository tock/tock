#include <stdio.h>
#include <limits.h>

#include <led.h>
#include <FXOS8700CQ.h>

int main(void) {
  printf("[App] Accelerometer -> LEDs\n");

  while (1) {
    int x, y, z;
    FXOS8700CQ_read_acceleration_sync(&x, &y, &z);

    // abs()
    if (x < 0) x *= -1;
    if (y < 0) y *= -1;
    if (z < 0) z *= -1;

    // Set LEDs based on acceleration.
    int largest = INT_MIN;
    if (x > largest) largest = x;
    if (y > largest) largest = y;
    if (z > largest) largest = z;

    if (x == largest) led_on(0); else led_off(0);
    if (y == largest) led_on(1); else led_off(1);
    if (z == largest) led_on(2); else led_off(2);
  }

  return 0;
}

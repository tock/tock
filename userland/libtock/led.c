#include "led.h"

int led_count(void) {
  return command(DRIVER_NUM_LEDS, 0, 0);
}

int led_on(int led_num) {
  return command(DRIVER_NUM_LEDS, 1, led_num);
}

int led_off(int led_num) {
  return command(DRIVER_NUM_LEDS, 2, led_num);
}

int led_toggle(int led_num) {
  return command(DRIVER_NUM_LEDS, 3, led_num);
}

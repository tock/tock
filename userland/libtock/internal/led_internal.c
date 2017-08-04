#include "led.h"

int led_internal_count(void) {
  return command(DRIVER_NUM_LEDS, 0, 0);
}

int led_internal_on(int led_num) {
  return command(DRIVER_NUM_LEDS, 1, led_num);
}

int led_internal_off(int led_num) {
  return command(DRIVER_NUM_LEDS, 2, led_num);
}

int led_internal_toggle(int led_num) {
  return command(DRIVER_NUM_LEDS, 3, led_num);
}

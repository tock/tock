#include "internal/led.h"

// For LEDs, these simply pass through to the syscall wrappers (the functions
// in the internal folder).

int led_on(int led_num) {
  return led_internal_on(led_num);
}

int led_off(int led_num) {
  return led_internal_off(led_num);
}

int led_toggle(int led_num) {
  return led_internal_toggle(led_num);
}

int led_count(void) {
  return led_internal_count();
}

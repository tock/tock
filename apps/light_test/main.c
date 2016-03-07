#include <firestorm.h>
#include <isl29035.h>
#include <stdio.h>

CB_TYPE nop(int x, int y, int z, void *ud) { return ASYNC; }

void print_intensity(int intensity) {
  static char msg[24];
  int len = snprintf(msg, sizeof(msg), "Intensity: %d\n", intensity);
  putnstr_async(msg, len, nop, NULL);
}

CB_TYPE intensity_cb(int intensity, int unused1, int unused2, void* ud) {
  print_intensity(intensity);
  return ASYNC;
}

CB_TYPE timer_fired(int arg0, int arg1, int arg2, void* ud) {
  gpio_toggle(LED_0);
  isl29035_start_intensity_reading();
  return ASYNC;
}

int main() {
  gpio_enable_output(LED_0);

  isl29035_subscribe(intensity_cb, NULL);

  // Setup periodic timer
  timer_repeating_subscribe(timer_fired, NULL);

  return 0;
}

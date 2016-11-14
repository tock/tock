#include <stdio.h>
#include <stdbool.h>

#include <timer.h>
#include <isl29035.h>
#include <tmp006.h>

void print_intensity(int intensity) {
  printf("Intensity: %d\n", intensity);
}

void intensity_cb(int intensity,
                  __attribute__ ((unused)) int unused1,
                  __attribute__ ((unused)) int unused2,
                  __attribute__ ((unused)) void* ud) {
  print_intensity(intensity);
}

void temp_callback(int temp_value,
                   int err,
                   __attribute__ ((unused)) int unused,
                   __attribute__ ((unused)) void* ud) {
  printf("Current Temp (%d) [0x%X]\n", temp_value, err);
}

void timer_fired(__attribute__ ((unused)) int arg0,
                 __attribute__ ((unused)) int arg1,
                 __attribute__ ((unused)) int arg2,
                 __attribute__ ((unused)) void* ud) {
  isl29035_start_intensity_reading();
}

int main() {
  printf("Hello\n");

  isl29035_subscribe(intensity_cb, NULL);
  // Setup periodic timer
  timer_subscribe(timer_fired, NULL);
  timer_start_repeating(1000);

  //tmp006_start_sampling(0x2, temp_callback, NULL);

  return 0;
}

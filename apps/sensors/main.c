#include <firestorm.h>
#include <isl29035.h>
#include <stdio.h>
#include <stdbool.h>

void print_intensity(int intensity) {
  printf("Intensity: %d\n", intensity);
}

void intensity_cb(int intensity, int unused1, int unused2, void* ud) {
  print_intensity(intensity);
}

void temp_callback(int temp_value, int err, int unused, void* ud) {
  printf("Current Temp (%d) [0x%X]\n", temp_value, err);
}

void timer_fired(int arg0, int arg1, int arg2, void* ud) {
  isl29035_start_intensity_reading();
}

int main() {
  printf("Hello\n");

  isl29035_subscribe(intensity_cb, NULL);
  // Setup periodic timer
  timer_subscribe(timer_fired, NULL);
  timer_start_repeating(1000);

  tmp006_start_sampling(0x2, temp_callback, NULL);

  return 0;
}

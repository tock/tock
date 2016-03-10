#include <firestorm.h>
#include <isl29035.h>
#include <stdio.h>
#include <stdbool.h>

bool inflight = false;
const char *buf = NULL;


CB_TYPE putstr_cb(int x, int y, int z, void *ud) {
  if (buf == NULL) {
    inflight = false;
  } else {
    putnstr_async(buf, strlen(buf), putstr_cb, buf);
    buf = NULL;
  }
  return ASYNC;
}

void putsa(const char* str) {
  if (inflight) {
    buf = str;
  } else {
    inflight = true;
    putnstr_async(str, strlen(str), putstr_cb, str);
  }
}

void print_intensity(int intensity) {
  static char msg[24];
  int len = snprintf(msg, sizeof(msg), "Intensity: %d\n", intensity);
  putsa(msg);
}

CB_TYPE intensity_cb(int intensity, int unused1, int unused2, void* ud) {
  print_intensity(intensity);
  return ASYNC;
}

CB_TYPE temp_callback(int temp_value, int err, int unused, void* ud) {
  gpio_toggle(LED_0);
  static char buf[64];
  snprintf(buf, 64, "Current Temp (%d) [0x%X]\n", temp_value, err);
  putsa(buf);
  return ASYNC;
}

CB_TYPE timer_fired(int arg0, int arg1, int arg2, void* ud) {
  isl29035_start_intensity_reading();
  return ASYNC;
}

int main() {
  static char hello[] = "Hello\n";
  putsa(hello);
  gpio_enable_output(LED_0);

  isl29035_subscribe(intensity_cb, NULL);
  // Setup periodic timer
  timer_repeating_subscribe(timer_fired, NULL);

  tmp006_start_sampling(0x2, temp_callback, NULL);

  return 0;
}

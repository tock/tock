#include "internal/alarm.h"

int alarm_internal_subscribe(subscribe_cb cb, void *userdata) {
  return subscribe(3, 0, cb, userdata);
}

int alarm_internal_oneshot(uint32_t interval_ms) {
  return command(3, 1, (int)interval_ms);
}

int alarm_internal_start_repeating(uint32_t interval_ms) {
  return command(3, 2, (int)interval_ms);
}

int alarm_internal_absolute(uint32_t tics) {
  return command(3, 5, (int)tics);
}

int alarm_internal_stop(void) {
  return command(3, 3, 0);
}

unsigned int alarm_internal_frequency(void) {
  return (unsigned int) command(3, 6, 0);
}

unsigned int alarm_internal_read(void) {
  return (unsigned int) command(3, 4, 0);
}

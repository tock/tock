#include "internal/alarm.h"

int alarm_internal_subscribe(subscribe_cb cb, void *userdata) {
  return subscribe(DRIVER_NUM_ALARM, 0, cb, userdata);
}

int alarm_internal_oneshot(uint32_t interval_ms) {
  return command(DRIVER_NUM_ALARM, 1, (int)interval_ms);
}

int alarm_internal_start_repeating(uint32_t interval_ms) {
  return command(DRIVER_NUM_ALARM, 2, (int)interval_ms);
}

int alarm_internal_absolute(uint32_t tics) {
  return command(DRIVER_NUM_ALARM, 5, (int)tics);
}

int alarm_internal_stop(void) {
  return command(DRIVER_NUM_ALARM, 3, 0);
}

unsigned int alarm_internal_frequency(void) {
  return (unsigned int) command(DRIVER_NUM_ALARM, 6, 0);
}

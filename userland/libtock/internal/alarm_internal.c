#include "internal/alarm.h"

int alarm_internal_subscribe(subscribe_cb cb, void *userdata) {
  return subscribe(DRIVER_NUM_ALARM, 0, cb, userdata);
}

int alarm_internal_set(uint32_t tics) {
  return command(DRIVER_NUM_ALARM, 4, (int)tics);
}

int alarm_internal_stop(void) {
  return command(DRIVER_NUM_ALARM, 3, 0);
}

unsigned int alarm_internal_frequency(void) {
  return (unsigned int) command(DRIVER_NUM_ALARM, 1, 0);
}

#include "temperature.h"

int temperature_init(subscribe_cb callback, __attribute__ ((unused)) void *ud) {
  return subscribe(DRIVER_TEMP, 0, callback, NULL);
}

int temperature_measure(void) {
  return command(DRIVER_TEMP, 0, 0);
}

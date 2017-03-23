#include "temperature.h"

int temperature_init(subscribe_cb callback, void *ud) {
  return subscribe(DRIVER_TEMP, 0, callback, NULL);
}

int temperature_measure() {
  return command(DRIVER_TEMP, 0, 0);
}

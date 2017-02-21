#include "temperature.h"

int temp_init(subscribe_cb callback, void *ud) {
  return subscribe(DRIVER_TEMP, 0, callback, NULL);
}

int temp_measure() {
  return command(DRIVER_TEMP, 0, 0);
}

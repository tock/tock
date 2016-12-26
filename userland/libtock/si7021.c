#include "tock.h"
#include "si7021.h"

struct data {
  bool fired;
  int temp;
  int humi;
};

static struct data result = { .fired = false };

// Internal callback for faking synchronous reads
static void cb(int temp,
               int humidity,
               __attribute__ ((unused)) int unused,
               void* ud) {
  struct data* result = (struct data*) ud;
  result->temp = temp;
  result->humi = humidity;
  result->fired = true;
}

int si7021_set_callback (subscribe_cb callback, void* callback_args) {
    return subscribe(DRIVER_NUM_SI7021, 0, callback, callback_args);
}

int si7021_get_temperature_humidity () {
    return command(DRIVER_NUM_SI7021, 1, 0);
}

int si7021_get_temperature_humidity_sync (int* temperature, int* humidity) {
    int err;
    result.fired = false;

    err = si7021_set_callback(cb, (void*) &result);
    if (err < 0) return err;

    err = si7021_get_temperature_humidity();
    if (err < 0) return err;

    // Wait for the callback.
    yield_for(&result.fired);

    *temperature = result.temp;
    *humidity = result.humi;

    return 0;
}

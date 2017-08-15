#include "temperature.h"
#include "tock.h"

struct data {
  bool fired;
  int temp;
};

static struct data result = { .fired = false };

// Internal callback for faking synchronous reads
static void cb(int temp,
               __attribute__ ((unused)) int unused,
               __attribute__ ((unused)) int unused1,
               void* ud) {
  struct data* data = (struct data*) ud;
  data->temp  = temp;
  data->fired = true;
}

int temperature_set_callback(subscribe_cb callback, void* callback_args) {
  return subscribe(DRIVER_NUM_TEMPERATURE, 0, callback, callback_args);
}

int temperature_read(void) {
  return command(DRIVER_NUM_TEMPERATURE, 1, 0);
}

int temperature_read_sync(int* temperature) {
  int err;
  result.fired = false;

  err = temperature_set_callback(cb, (void*) &result);
  if (err < 0) return err;

  err = temperature_read();
  if (err < 0) return err;

  // Wait for the callback.
  yield_for(&result.fired);

  *temperature = result.temp;

  return 0;
}


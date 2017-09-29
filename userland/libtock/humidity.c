#include "humidity.h"
#include "tock.h"

struct data {
  bool fired;
  int humidity;
};

static struct data result = { .fired = false };

// Internal callback for faking synchronous reads
static void cb(int humidity,
               __attribute__ ((unused)) int unused,
               __attribute__ ((unused)) int unused1,
               void* ud) {
  struct data* data = (struct data*) ud;
  data->humidity = humidity;
  data->fired    = true;
}

int humidity_set_callback(subscribe_cb callback, void* callback_args) {
  return subscribe(DRIVER_NUM_HUMIDITY, 0, callback, callback_args);
}

int humidity_read(void) {
  return command(DRIVER_NUM_HUMIDITY, 1, 0, 0);
}

int humidity_read_sync(unsigned* humidity) {
  int err;
  result.fired = false;

  err = humidity_set_callback(cb, (void*) &result);
  if (err < 0) return err;

  err = humidity_read();
  if (err < 0) return err;

  // Wait for the callback.
  yield_for(&result.fired);

  *humidity = result.humidity;

  return 0;
}

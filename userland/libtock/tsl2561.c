#include "tock.h"
#include "tsl2561.h"

struct tsl2561_data {
  bool fired;
  int value;
};

static struct tsl2561_data result = { .fired = false };

// Internal callback for faking synchronous reads
static void tsl2561_cb(__attribute__ ((unused)) int callback_type,
                       int value,
                       __attribute__ ((unused)) int unused2,
                       void* ud) {
  struct tsl2561_data* data = (struct tsl2561_data*) ud;
  data->value = value;
  data->fired = true;
}

int tsl2561_set_callback (subscribe_cb callback, void* callback_args) {
  return subscribe(DRIVER_NUM_TSL2561, 0, callback, callback_args);
}

int tsl2561_get_lux (void) {
  return command(DRIVER_NUM_TSL2561, 1, 0, 0);
}

int tsl2561_get_lux_sync (void) {
  int err;
  result.fired = false;

  err = tsl2561_set_callback(tsl2561_cb, (void*) &result);
  if (err < 0) return err;

  err = tsl2561_get_lux();
  if (err < 0) return err;

  // Wait for the callback.
  yield_for(&result.fired);

  return result.value;
}

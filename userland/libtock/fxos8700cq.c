#include <stdio.h>
#include "fxos8700cq.h"
#include "math.h"

struct fx0_data {
  int x;
  int y;
  int z;
  bool fired;
};

static struct fx0_data res = { .fired = false };

// internal callback for faking synchronous reads
static void fxos8700cq_cb(int x, int y, int z, void* ud) {
  struct fx0_data* result = (struct fx0_data*) ud;
  result->x = x;
  result->y = y;
  result->z = z;
  result->fired = true;
}

double fxos8700cq_read_accel_mag(void) {
  struct fx0_data result = { .fired = false };
  int err;

  err = fxos8700cq_subscribe(fxos8700cq_cb, (void*)(&result));
  if (err < 0) {
    return err;
  }

  err = fxos8700cq_start_accel_reading();
  if (err < 0) {
    return err;
  }

  yield_for(&result.fired);

  return sqrt(result.x * result.x + result.y * result.y + result.z * result.z);
}

int fxos8700cq_subscribe(subscribe_cb callback, void* userdata) {
  return subscribe(11, 0, callback, userdata);
}

int fxos8700cq_start_accel_reading(void) {
  return command(11, 1, 0);
}
int fxos8700cq_start_magnetometer_reading(void) {
  return command(11, 2, 0);
}

int fxos8700cq_read_acceleration_sync(int* x, int* y, int* z) {
    int err;
    res.fired = false;

    err = fxos8700cq_subscribe(fxos8700cq_cb, (void*) &res);
    if (err < 0) return err;

    err = fxos8700cq_start_accel_reading();
    if (err < 0) return err;

    // Wait for the callback.
    yield_for(&res.fired);

    *x = res.x;
    *y = res.y;
    *z = res.z;

    return 0;
}

int fxos8700cq_read_magenetometer_sync(int* x, int* y, int* z) {
    int err;
    res.fired = false;

    err = fxos8700cq_subscribe(fxos8700cq_cb, (void*) &res);
    if (err < 0) return err;

    err = fxos8700cq_start_magnetometer_reading();
    if (err < 0) return err;

    // Wait for the callback.
    yield_for(&res.fired);

    *x = res.x;
    *y = res.y;
    *z = res.z;

    return 0;
}

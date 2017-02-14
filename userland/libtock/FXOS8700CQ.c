#include <stdio.h>
#include "FXOS8700CQ.h"
#include "math.h"

struct fx0_data {
  int x;
  int y;
  int z;
  bool fired;
};

static struct fx0_data res = { .fired = false };

// internal callback for faking synchronous reads
static void FXOS8700CQ_cb(int x, int y, int z, void* ud) {
  struct fx0_data* result = (struct fx0_data*) ud;
  result->x = x;
  result->y = y;
  result->z = z;
  result->fired = true;
}

double FXOS8700CQ_read_accel_mag(void) {
  struct fx0_data result = { .fired = false };
  int err;

  err = FXOS8700CQ_subscribe(FXOS8700CQ_cb, (void*)(&result));
  if (err < 0) {
    return err;
  }

  err = FXOS8700CQ_start_accel_reading();
  if (err < 0) {
    return err;
  }

  yield_for(&result.fired);

  return sqrt(result.x * result.x + result.y * result.y + result.z * result.z);
}

int FXOS8700CQ_subscribe(subscribe_cb callback, void* userdata) {
  return subscribe(11, 0, callback, userdata);
}

int FXOS8700CQ_start_accel_reading(void) {
  return command(11, 1, 0);
}
int FXOS8700CQ_start_magnetometer_reading(void) {
  return command(11, 2, 0);
}

int FXOS8700CQ_read_acceleration_sync(int* x, int* y, int* z) {
    int err;
    res.fired = false;

    err = FXOS8700CQ_subscribe(FXOS8700CQ_cb, (void*) &res);
    if (err < 0) return err;

    err = FXOS8700CQ_start_accel_reading();
    if (err < 0) return err;

    // Wait for the callback.
    yield_for(&res.fired);

    *x = res.x;
    *y = res.y;
    *z = res.z;

    return 0;
}

int FXOS8700CQ_read_magenetometer_sync(int* x, int* y, int* z) {
    int err;
    res.fired = false;

    err = FXOS8700CQ_subscribe(FXOS8700CQ_cb, (void*) &res);
    if (err < 0) return err;

    err = FXOS8700CQ_start_magnetometer_reading();
    if (err < 0) return err;

    // Wait for the callback.
    yield_for(&res.fired);

    *x = res.x;
    *y = res.y;
    *z = res.z;

    return 0;
}

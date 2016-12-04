#include <stdio.h>
#include "FXOS8700CQ.h"
#include "math.h"

struct fx0_data {
  int x;
  int y;
  int z;
  bool fired;
};

// internal callback for faking synchronous reads
static void FXOS8700CQ_cb(int x, int y, int z, void* ud) {
  struct fx0_data* result = (struct fx0_data*) ud;
  result->x = x;
  result->y = y; 
  result->z = z; 
  result->fired = true;
}

double FXOS8700CQ_read_accel_mag() {
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

  // TODO add sqrt of accel magnitude once software floating point supported in userspace
  // adding sqrt(...) currently causes crash 
  return (result.x * result.x + result.y * result.y + result.z * result.z);
}

int FXOS8700CQ_subscribe(subscribe_cb callback, void* userdata) {
  return subscribe(11, 0, callback, userdata);
}

int FXOS8700CQ_start_accel_reading() {
  return command(11, 0, 0);
}


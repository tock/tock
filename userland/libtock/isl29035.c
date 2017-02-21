#include "isl29035.h"

struct isl_data {
  int intensity;
  bool fired;
};

// internal callback for faking synchronous reads
static void isl29035_cb(int intensity,
                           __attribute__ ((unused)) int unused1,
                           __attribute__ ((unused)) int unused2, void* ud) {
  struct isl_data* result = (struct isl_data*)ud;
  result->intensity = intensity;
  result->fired = true;
}

int isl29035_read_light_intensity(void) {
  struct isl_data result = { .fired = false };
  int err;

  err = isl29035_subscribe(isl29035_cb, (void*)(&result));
  if (err < 0) {
    return err;
  }

  err = isl29035_start_intensity_reading();
  if (err < 0) {
    return err;
  }

  yield_for(&result.fired);

  return result.intensity;
}

int isl29035_subscribe(subscribe_cb callback, void* userdata) {
  return subscribe(DRIVER_NUM_ISL29035, 0, callback, userdata);
}

int isl29035_start_intensity_reading(void) {
  return command(DRIVER_NUM_ISL29035, 1, 0);
}


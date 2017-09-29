#include "ambient_light.h"

struct ambient_light_data {
  int intensity;
  bool fired;
};

// internal callback for faking synchronous reads
static void ambient_light_cb(int intensity,
                             __attribute__ ((unused)) int unused1,
                             __attribute__ ((unused)) int unused2, void* ud) {
  struct ambient_light_data* result = (struct ambient_light_data*)ud;
  result->intensity = intensity;
  result->fired     = true;
}

int ambient_light_read_intensity(void) {
  struct ambient_light_data result = { .fired = false };
  int err;

  err = ambient_light_subscribe(ambient_light_cb, (void*)(&result));
  if (err < 0) {
    return err;
  }

  err = ambient_light_start_intensity_reading();
  if (err < 0) {
    return err;
  }

  yield_for(&result.fired);

  return result.intensity;
}

int ambient_light_subscribe(subscribe_cb callback, void* userdata) {
  return subscribe(DRIVER_NUM_AMBIENT_LIGHT, 0, callback, userdata);
}

int ambient_light_start_intensity_reading(void) {
  return command(DRIVER_NUM_AMBIENT_LIGHT, 1, 0, 0);
}


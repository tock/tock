#include "ambient_light.h"
#include "tock.h"

typedef struct {
  int intensity;
  bool fired;
} ambient_light_data_t;

// internal callback for faking synchronous reads
static void ambient_light_cb(int intensity,
                             __attribute__ ((unused)) int unused1,
                             __attribute__ ((unused)) int unused2, void* ud) {
  ambient_light_data_t* result = (ambient_light_data_t*)ud;
  result->intensity = intensity;
  result->fired     = true;
}

int ambient_light_read_intensity_sync(int* lux_value) {
  int err;
  ambient_light_data_t result = {0};
  result.fired = false;

  err = ambient_light_subscribe(ambient_light_cb, (void*)(&result));
  if (err < TOCK_SUCCESS) {
    return err;
  }

  err = ambient_light_start_intensity_reading();
  if (err < TOCK_SUCCESS) {
    return err;
  }

  yield_for(&result.fired);

  *lux_value = result.intensity;

  return TOCK_SUCCESS;
}

int ambient_light_subscribe(subscribe_cb callback, void* userdata) {
  return subscribe(DRIVER_NUM_AMBIENT_LIGHT, 0, callback, userdata);
}

int ambient_light_start_intensity_reading(void) {
  return command(DRIVER_NUM_AMBIENT_LIGHT, 1, 0, 0);
}


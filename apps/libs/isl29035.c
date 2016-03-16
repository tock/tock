#include <firestorm.h>
#include <tock.h>
#include <isl29035.h>

// internal callback for faking synchronous reads
static CB_TYPE isl29035_cb(int intensity,
                           __attribute__ ((unused)) int unused1,
                           __attribute__ ((unused)) int unused2, void* ud) {
  int* result = (int*)ud;
  *result = intensity;
  return READLIGHT;
}

int isl29035_read_light_intensity() {
  int result;
  int err;

  err = isl29035_subscribe(isl29035_cb, (void*)(&result));
  if (err < 0) {
    return err;
  }

  err = isl29035_start_intensity_reading();
  if (err < 0) {
    return err;
  }

  wait_for(READLIGHT);

  return result;
}

int isl29035_subscribe(subscribe_cb callback, void* userdata) {
  return subscribe(6, 0, callback, userdata);
}

int isl29035_start_intensity_reading() {
  return command(6, 0, 0);
}


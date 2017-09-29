#include "tmp006.h"

static bool readtemp;

// internal callback for faking synchronous reads
static void tmp006_cb(int temp_value,
                      int error_code,
                      int unused __attribute__ ((unused)),
                      void* callback_args) {
  readtemp = true;

  // return data to user
  int32_t* callback_vals = (int32_t*)callback_args;
  callback_vals[0] = temp_value;
  callback_vals[1] = error_code;
}

// enable TMP006, take a single reading, disable TMP006, return value to user
int tmp006_read_sync(int16_t* temp_reading) {
  // store temperature value and error code
  int32_t callback_vals[2] = {0};

  // request a single sample
  uint32_t err_code = subscribe(DRIVER_NUM_TMP006, 0, tmp006_cb, callback_vals);
  if (err_code != ERR_NONE) {
    return err_code;
  }

  // wait for result
  readtemp = false;
  yield_for(&readtemp);

  // write value for user
  *temp_reading = (int16_t)callback_vals[0];

  // return error code to user
  return callback_vals[1];
}

// enable TMP006, take a single reading, disable TMP006, callback with value
int tmp006_read_async(subscribe_cb callback, void* callback_args) {

  // subscribe to a single temp value callback
  //  also enables the temperature sensor for the duration of one sample
  return subscribe(DRIVER_NUM_TMP006, 0, callback, callback_args);
}

// enable TMP006, configure periodic sampling with interrupts, callback with value on interrupt
int tmp006_start_sampling(uint8_t period, subscribe_cb callback, void* callback_args) {
  // set period for periodic temp readings
  uint32_t err_code = command(DRIVER_NUM_TMP006, 1, period, 0);
  if (err_code != ERR_NONE) {
    return err_code;
  }

  // subscribe to periodic temp value callbacks
  //  also enables the temperature sensor
  return subscribe(DRIVER_NUM_TMP006, 1, callback, callback_args);
}

// disable TMP006
int tmp006_stop_sampling(void) {
  // unsubscribe from periodic temp value callbacks
  //  also disables the temperature sensor
  return command(DRIVER_NUM_TMP006, 2, 0, 0);
}


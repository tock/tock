#pragma once

#include <tock.h>

#ifdef __cplusplus
extern "C" {
#endif

#define DRIVER_NUM_FXO 11

// Proivide a callback function for acceleration readings
int FXOS8700CQ_subscribe(subscribe_cb callback, void* userdata);
// Read acceleration and relay to callback function
int FXOS8700CQ_start_accel_reading(void);
// Read magnetometer and relay to callback function
int FXOS8700CQ_start_magnetometer_reading(void);
// Read square of magnitude of acceleration (blocking)
double FXOS8700CQ_read_accel_mag(void);

// Get the magnitude of acceleration in the X,Y,Z directions. Blocking.
int FXOS8700CQ_read_acceleration_sync(int* x, int* y, int* z);

// Get a reading from the magnetometer. Blocking.
int FXOS8700CQ_read_magenetometer_sync(int* x, int* y, int* z);

#ifdef __cplusplus
}
#endif

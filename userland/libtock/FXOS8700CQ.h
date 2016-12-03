#pragma once

#include <tock.h>

#ifdef __cplusplus
extern "C" {
#endif

#define DRIVER_NUM_FXO 11

// Proivide a callback function for acceleration readings 
int FXOS8700CQ_subscribe(subscribe_cb callback, void* userdata);
// Read acceleration and relay to callback function 
int FXOS8700CQ_start_accel_reading();
// Read square of magnitude of acceleration (blocking)
double FXOS8700CQ_read_accel_mag();

#ifdef __cplusplus
}
#endif


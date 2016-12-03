#ifndef _FXOS8700CQ_H
#define _FXOS8700CQ_H

#include <tock.h>

#ifdef __cplusplus
extern "C" {
#endif

#define DRIVER_NUM_FXO 11

int FXOS8700CQ_subscribe(subscribe_cb callback, void* userdata);
int FXOS8700CQ_start_accel_reading();

double FXOS8700CQ_read_accel_mag();

#ifdef __cplusplus
}
#endif

#endif // _FXOS8700CQ_H

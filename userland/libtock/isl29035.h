#ifndef _ISL29035_H
#define _ISL29035_H

#include "tock.h"

#ifdef __cplusplus
extern "C" {
#endif

int isl29035_subscribe(subscribe_cb callback, void* callback_args);
int isl29035_start_intensity_reading();

int isl29035_read_light_intensity();

#ifdef __cplusplus
}
#endif

#endif // _ISL29035_H

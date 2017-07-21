#pragma once

#include "tock.h"

#ifdef __cplusplus
extern "C" {
#endif

#define DRIVER_NUM_TEMPERATURE 10

int temperature_set_callback (subscribe_cb callback, void* callback_args);

int temperature_ambient_get(void);
int temperature_ambient_get_sync (int* temperature);

int temperature_cpu_get(void);
int temperature_cpu_get_sync(int* temperature);

#ifdef __cplusplus
}
#endif

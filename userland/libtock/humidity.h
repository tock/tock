#pragma once

#include "tock.h"

#ifdef __cplusplus
extern "C" {
#endif

#define DRIVER_NUM_HUMIDITY 35

int humidity_set_callback (subscribe_cb callback, void* callback_args);
int humidity_get(void);

int humidity_get_sync (unsigned* humi);

#ifdef __cplusplus
}
#endif

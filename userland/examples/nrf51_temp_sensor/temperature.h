#pragma once

#include <tock.h>

#ifdef __cplusplus
extern "C" {
#endif

#define DRIVER_TEMP 36

int temperature_init(subscribe_cb callback, void *ud);
int temperature_measure();

#ifdef __cplusplus
}
#endif


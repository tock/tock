#pragma once

#include <tock.h>

#define DRIVER_TEMP 36

#ifdef __cplusplus
extern "C" {
#endif

int temp_init(subscribe_cb callback, void *ud);
int temp_measure();

#ifdef __cplusplus
}
#endif


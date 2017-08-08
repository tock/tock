#pragma once

#include "tock.h"

#ifdef __cplusplus
extern "C" {
#endif

#define ERR_NONE 0
#define DRIVER_NUM_TMP006 0x70001

int tmp006_read_sync(int16_t* temp_reading);
int tmp006_read_async(subscribe_cb callback, void* callback_args);
int tmp006_start_sampling(uint8_t period, subscribe_cb callback, void* callback_args);
int tmp006_stop_sampling(void);

#ifdef __cplusplus
}
#endif

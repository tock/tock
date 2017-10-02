#pragma once

#include "tock.h"

#ifdef __cplusplus
extern "C" {
#endif

#define DRIVER_NUM_LPS25HB 0x70004

int lps25hb_set_callback (subscribe_cb callback, void* callback_args);
int lps25hb_get_pressure (void);

int lps25hb_get_pressure_sync (void);

#ifdef __cplusplus
}
#endif

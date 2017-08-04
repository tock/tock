#pragma once

#include "tock.h"

#ifdef __cplusplus
extern "C" {
#endif

int led_on(int led_num);
int led_off(int led_num);
int led_toggle(int led_num);

// Returns the number of LEDs on the host platform.
int led_count(void);

#ifdef __cplusplus
}
#endif

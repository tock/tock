#pragma once

#include "tock.h"

#ifdef __cplusplus
extern "C" {
#endif

#define DRIVER_NUM_LEDS 0x00000002

/*
 * Returns the number of LEDs on the host platform.
 */
int led_internal_count(void);

/*
 * Turn an LED on.
 *
 * led_num - Index of the LED starting at 0.
 */
int led_internal_on(int led_num);

/*
 * Turn an LED off.
 *
 * led_num - Index of the LED starting at 0.
 */
int led_internal_off(int led_num);

/*
 * Toggle the state of an LED.
 *
 * led_num - Index of the LED starting at 0.
 */
int led_internal_toggle(int led_num);

#ifdef __cplusplus
}
#endif

#pragma once

#include "tock.h"

#ifdef __cplusplus
extern "C" {
#endif

#define DRIVER_NUM_ALARM 0x0

/*
 * Sets the callback for timers
 *
 * When invoked, the callback's first argument will be the timer value at which
 * the timer was fired.
 *
 * Side-effects: cancels any existing/outstanding timers
 */
int alarm_internal_subscribe(subscribe_cb cb, void *userdata);

/*
 * Starts a repeating timer
 *
 * interval_ms - the interval for the timer in milliseconds
 *
 * Side-effects: cancels any existing/outstanding timers
 */
int alarm_internal_start_repeating(uint32_t interval_ms);

/*
 * Starts a oneshot timer
 *
 * interval_ms - the interval for the timer in milliseconds
 *
 * Side-effects: cancels any existing/outstanding timers
 */
int alarm_internal_oneshot(uint32_t interval_ms);

/*
 * Starts a oneshot alarm
 *
 * expiration - absolute expiration value in clock tics
 *
 * Side-effects: cancels any existing/outstanding timers
 */
int alarm_internal_absolute(uint32_t tics);


/*
 * Stops any outstanding hardware alarm.
 *
 * Side-effects: cancels any existing/outstanding timers
 */
int alarm_internal_stop(void);

/*
 * Get the the timer frequency in Hz.
 */
unsigned int alarm_internal_frequency(void);

#ifdef __cplusplus
}
#endif

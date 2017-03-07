#pragma once

#ifdef __cplusplus
extern "C" {
#endif

#include <tock.h>

/*
 * Sets the callback for timers
 *
 * Side-effects: cancels any existing/outstanding timers
 */
int timer_subscribe(subscribe_cb cb, void *userdata);

/*
 * Starts a repeating timer
 *
 * interval_ms - the interval for the timer in milliseconds
 *
 * Side-effects: cancels any existing/outstanding timers
 */
int timer_start_repeating(uint32_t interval_ms);

/*
 * Starts a oneshot timer
 *
 * interval_ms - the interval for the timer in milliseconds
 *
 * Side-effects: cancels any existing/outstanding timers
 */
int timer_oneshot(uint32_t interval_ms);

int timer_stop(void);

/*
 * Get the current counter value of the timer.
 */
unsigned int timer_read(void);

/*
 * Blocks for the given amount of time in millisecond.
 *
 * This is a wrapper around the `timer` interface, so calling this will cancel
 * any outstanding timers as well as replace the timer callback.
 */
void delay_ms(uint32_t ms);

#ifdef __cplusplus
}
#endif

#pragma once

#ifdef __cplusplus
extern "C" {
#endif

#include <tock.h>

typedef struct alarm alarm_t;

alarm_t *alarm_start(uint32_t expiration, subscribe_cb, void*);

alarm_t *alarm_in(uint32_t ms, subscribe_cb, void*);

typedef struct alarm_repeating alarm_repeating_t;

alarm_repeating_t* alarm_every(uint32_t ms, subscribe_cb, void*);

void alarm_cancel(alarm_t*);

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

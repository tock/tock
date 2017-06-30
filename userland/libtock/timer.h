/** @file timer.h
 * @brief Timer function prototypes
 *
 * The timer module allows the client to receive callbacks when single-shot or
 * periodic timers expire. Timers are measured at millisecond granularity,
 * regardless of the hardware clock's native frequency. In addition, the
 * `delay_ms` function is a blocking call that returns after the given number
 * of milliseconds.
 *
 * # Structures
 *
 * `timer_repeating_t` represents a handle to a continuous, repeating alarm. In
 * addition, the single-shot timer uses the `alarm_t` structure defined in
 * alarm.h.
 *
 * ## Example
 *
 *     static void callback(int now, int interval, int arg2, char* str) {
 *       printf("%s\n", str);
 *     }
 *
 *     timer_in(1000, callback, (void*)"1 second elapsed");
 *     timer_repeating(2000, callback, (void*)"Another 2 seconds elapsed");
 *
 */

#pragma once

#ifdef __cplusplus
extern "C" {
#endif

#include <tock.h>

#include "alarm.h"

/** \brief Opaque handle to a repeating alarm.
 *
 * An opaque handle to a repeating alarm created by `alarm_every`.
 */
typedef struct timer_repeating timer_repeating_t;

/** \brief Create a new alarm to fire in `ms` milliseconds.
 *
 * \param ms the number of milliseconds to fire the alarm after.
 * \param callback a callback to be invoked when the alarm expires.
 * \param userdata passed to the callback.
 * \return A handle to the alarm that was created.
 */
alarm_t *timer_in(uint32_t ms, subscribe_cb, void*);

/** \brief Create a new repeating alarm to fire every `ms` milliseconds.
 *
 * \param ms the interval to fire the alarm at in milliseconds.
 * \param callback a callback to be invoked when the alarm expires.
 * \param userdata passed to the callback.
 * \return A handle to the repeating alarm that was created.
 */
timer_repeating_t* timer_every(uint32_t ms, subscribe_cb, void*);

/** \brief Cancels an existing alarm.
 *
 * \param alarm
 */
void timer_cancel(timer_repeating_t*);

/** \brief Blocks for the given amount of time in millisecond.
 *
 * This is a blocking version of `alarm_in`. Instead of calling a user
 * specified callback, it blocks the current call-stack.
 *
 * \param ms the number of milliseconds to delay for.
 */
void delay_ms(uint32_t ms);


#ifdef __cplusplus
}
#endif

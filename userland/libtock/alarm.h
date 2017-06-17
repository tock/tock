/** @file alarm.h
 * @brief Alarm and timer function prototypes
 *
 * The alarm and timer module allows the client to initiate alarms and receive
 * callbacks when those alarms have expired. Clients can set one-shot alarms to
 * fire at particular clock values (`alarm_at`), with some number of
 * milliseconds, or repeating every severl milliseconds. In addition, the
 * `delay_ms` function is a blocking call that returns after the given number
 * of milliseconds.
 *
 * # Usage details
 *
 * There are two main, opaque, data-structures:
 *
 *   1. `alarm_t` represents a handle to a single-shot alarm.
 *
 *   2. `alarm_repeating_t` represents a handle to a continuous, repeating alarm.
 *
 * They are different types because the alarm implementation may keep track of
 * different metadata.
 *
 * The client should not assume anything about the underlying clock used by an
 * implementation other than that it is running at sufficient frequency to
 * deliver at least millisecond granularity and that it is a 32-bit clock (i.e.
 * it will wrap at 2^32 clock ticks).
 *
 * ## Example
 *
 *     static void callback(int now, int interval, int arg2, char* str) {
 *       printf("%s\n", str);
 *     }
 *
 *     alarm_in(1000, callback, (void*)"1 second elapsed");
 *     alarm_repeating(2000, callback, (void*)"Another 2 seconds elapsed");
 *
 */

#pragma once

#ifdef __cplusplus
extern "C" {
#endif

#include <tock.h>

/** \brief Opaque handle to a single-shot alarm.
 *
 * An opaque handle to an alarm created by `alarm_at` or `alarm_in`. Memory
 * management is handled by the underlying implementation.
 *
 * \bug Memory mangaement shouldn't be handled by the underlying
 * implementation. This makes it pretty dangerous to every use `alarm_cancel`.
 */
typedef struct alarm alarm_t;

/** \brief Opaque handle to a repeating alarm.
 *
 * An opaque handle to a repeating alarm created by `alarm_every`.
 */
typedef struct alarm_repeating alarm_repeating_t;

/** \brief Create a new alarm to fire at a particular clock value.
 *
 * \param expiration the clock value to schedule the alarm for.
 * \param callback a callback to be invoked when the alarm expires.
 * \param userdata passed to the callback.
 * \return A handle to the alarm that was created.
 */
alarm_t *alarm_at(uint32_t expiration, subscribe_cb, void*);

/** \brief Create a new alarm to fire in `ms` milliseconds.
 *
 * \param ms the number of milliseconds to fire the alarm after.
 * \param callback a callback to be invoked when the alarm expires.
 * \param userdata passed to the callback.
 * \return A handle to the alarm that was created.
 */
alarm_t *alarm_in(uint32_t ms, subscribe_cb, void*);

/** \brief Create a new repeating alarm to fire every `ms` milliseconds.
 *
 * \param ms the interval to fire the alarm at in milliseconds.
 * \param callback a callback to be invoked when the alarm expires.
 * \param userdata passed to the callback.
 * \return A handle to the repeating alarm that was created.
 */
alarm_repeating_t* alarm_every(uint32_t ms, subscribe_cb, void*);

/** \brief Cancels an existing alarm.
 *
 * \param alarm
 */
void alarm_cancel(alarm_t*);

/** \brief Get the current counter value of the timer.
 * \return The current value of the underlying clock.
 */
uint32_t alarm_read(void);

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

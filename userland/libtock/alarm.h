/** @file alarm.h
 * @brief Alarm function prototypes
 *
 * The alarm module allows the client to initiate alarms and receive
 * callbacks when those alarms have expired. Clients can set one-shot alarms to
 * fire at particular clock values (`alarm_at`)
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
 *     uint32_t frequency = alarm_frequency();
 *     uint32_t now = alarm_now();
 *     alarm_at(now + frequency, callback, (void*)"1 second elapsed");
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
 * implementation. This makes it pretty dangerous to ever use `alarm_cancel`.
 */
typedef struct alarm alarm_t;

/** \brief Create a new alarm to fire at a particular clock value.
 *
 * \param expiration the clock value to schedule the alarm for.
 * \param callback a callback to be invoked when the alarm expires.
 * \param userdata passed to the callback.
 * \return A handle to the alarm that was created.
 */
alarm_t *alarm_at(uint32_t expiration, subscribe_cb, void*);

/** \brief Cancels an existing alarm.
 *
 * \param alarm
 */
void alarm_cancel(alarm_t*);

/** \brief Get the current counter value of the timer.
 * \return The current value of the underlying clock.
 */
uint32_t alarm_read(void);


#ifdef __cplusplus
}
#endif

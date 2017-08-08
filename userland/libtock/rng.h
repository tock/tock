#pragma once

#include "tock.h"

#ifdef __cplusplus
extern "C" {
#endif

#define DRIVER_NUM_RNG 0x40001

/*  rng_async
 *  Asynchronous RNG request. Registers callback and buffer and starts RNG.
 *    callback: user defined callback.
 *    buf: user defined buffer.
 *    len: length of buffer.
 *    num: number of random bytes requested.
 *  returns 0 on success, negative on failure.
 */
int rng_async(subscribe_cb callback, uint8_t* buf, uint32_t len, uint32_t num);

/*  rng_sync
 *  Synchronous RNG request.
 *    buf: user defined buffer.
 *    len: length of buffer.
 *    num: number of random bytes requested.
 *  returns number of random bytes acquired on success, negative on failure.
 */
int rng_sync(uint8_t* buf, uint32_t len, uint32_t num);

/*  rng_set_callback()
 *  Registers a callback function that is called when requested randomness is
 *  obtained or provided buffer is full. Call this before rng_get_random().
 *    callback: user defined callback function of the form:
 *      void user_callback(int callback_type, int received, int unused, void* return);
 *      where receieved is the number of random bytes actually returned by the rng.
 *    callback_args: unused.
 *  returns 0 on success, negative on failure.
 */
int rng_set_callback(subscribe_cb callback, void* callback_args);

/*  rng_set_buffer()
 *  Registers buffer to hold received randomness. Call before rng_get_random().
 *    buffer: pointer to uint8_t array to store randomness
 *    len: length of buffer.
 *  returns 0 on success, negative on failure.
 */
int rng_set_buffer(uint8_t* buf, uint32_t len);

/*  rng_get_random
 *  Starts random number generator. Call after rng_set_callback and
 *  rng_set_buffer.
 *    num_bytes: number of random bytes requested.
 *  returns 0 on success, negative on failure.
 */
int rng_get_random(int num_bytes);

#ifdef __cplusplus
}
#endif

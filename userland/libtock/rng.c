#include <rng.h>
#include <tock.h>

struct rng_data {
  bool fired;
  int received;
};

static struct rng_data result = { .fired = false, .received = 0};

// Internal callback for faking synchronous reads
static void rng_cb(__attribute__ ((unused)) int callback_type,
                   int received,
                   __attribute__ ((unused)) int val2,
                   void* ud) {
  struct rng_data* data = (struct rng_data*) ud;
  data->fired    = true;
  data->received = received;
}

int rng_set_buffer(uint8_t* buf, uint32_t len) {
  return allow(DRIVER_NUM_RNG, 0, (void*) buf, len);
}

int rng_set_callback(subscribe_cb callback, void* callback_args) {
  return subscribe(DRIVER_NUM_RNG, 0, callback, callback_args);
}

int rng_get_random(int num_bytes) {
  return command(DRIVER_NUM_RNG, 1, num_bytes, 0);
}

int rng_async(subscribe_cb callback, uint8_t* buf, uint32_t len, uint32_t num) {
  int err;

  err = rng_set_callback(callback, NULL);
  if (err < 0) return err;

  err = rng_set_buffer(buf, len);
  if (err < 0) return err;

  return rng_get_random(num);
}

int rng_sync(uint8_t* buf, uint32_t len, uint32_t num) {
  int err;

  err = rng_set_buffer(buf, len);
  if (err < 0) return err;

  err = rng_set_callback(rng_cb, (void*) &result);
  if (err < 0) return err;

  result.fired = false;
  err = rng_get_random(num);
  if (err < 0) return err;

  yield_for(&result.fired);

  return result.received;
}

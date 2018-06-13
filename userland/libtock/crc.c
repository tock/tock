#include "crc.h"

int crc_exists(void) {
  return command(DRIVER_NUM_CRC, 0, 0, 0) >= 0;
}

int crc_request(enum crc_alg alg) {
  return command(DRIVER_NUM_CRC, 2, alg, 0);
}

int crc_subscribe(subscribe_cb callback, void *ud) {
  return subscribe(DRIVER_NUM_CRC, 0, callback, ud);
}

int crc_set_buffer(const void* buf, size_t len) {
  return allow(DRIVER_NUM_CRC, 0, (void*) buf, len);
}

struct data {
  bool fired;
  int status;
  uint32_t result;
};

static void callback(int status, int v1, __attribute__((unused)) int v2, void *data)
{
  struct data *d = data;

  d->fired  = true;
  d->status = status;
  d->result = v1;
}

int crc_compute(const void *buf, size_t buflen, enum crc_alg alg, uint32_t *result)
{
  struct data d = { .fired = false };

  crc_set_buffer(buf, buflen);
  crc_subscribe(callback, (void *) &d);
  crc_request(alg);
  yield_for(&d.fired);

  if (d.status == TOCK_SUCCESS)
    *result = d.result;

  return d.status;
}

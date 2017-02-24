#include "crc.h"

int crc_exists(void) {
  return command(DRIVER_NUM_CRC, 0, 0) >= 0;
}

uint32_t crc_version(void) {
  return command(DRIVER_NUM_CRC, 1, 0);
}

int crc_compute(enum crc_polynomial poly) {
  return command(DRIVER_NUM_CRC, 2, poly);
}

int crc_subscribe(subscribe_cb callback, void *ud) {
  return subscribe(DRIVER_NUM_CRC, 0, callback, ud);
}

int crc_set_buffer(const void* buf, size_t len) {
  return allow(DRIVER_NUM_CRC, 0, (void*) buf, len);
}

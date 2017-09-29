#include "internal/nonvolatile_storage.h"

int nonvolatile_storage_internal_read_done_subscribe(subscribe_cb cb, void *userdata) {
  return subscribe(DRIVER_NUM_NONVOLATILE_STORAGE, 0, cb, userdata);
}

int nonvolatile_storage_internal_write_done_subscribe(subscribe_cb cb, void *userdata) {
  return subscribe(DRIVER_NUM_NONVOLATILE_STORAGE, 1, cb, userdata);
}

int nonvolatile_storage_internal_read_buffer(uint8_t* buffer, uint32_t len) {
  return allow(DRIVER_NUM_NONVOLATILE_STORAGE, 0, (void*) buffer, len);
}

int nonvolatile_storage_internal_write_buffer(uint8_t* buffer, uint32_t len) {
  return allow(DRIVER_NUM_NONVOLATILE_STORAGE, 1, (void*) buffer, len);
}

int nonvolatile_storage_internal_get_number_bytes(void) {
  return command(DRIVER_NUM_NONVOLATILE_STORAGE, 1, 0, 0);
}

int nonvolatile_storage_internal_read(uint32_t offset, uint32_t length) {
  uint32_t arg0 = (length << 8) | 2;
  return command(DRIVER_NUM_NONVOLATILE_STORAGE, (int) arg0, (int) offset, 0);
}

int nonvolatile_storage_internal_write(uint32_t offset, uint32_t length) {
  uint32_t arg0 = (length << 8) | 3;
  return command(DRIVER_NUM_NONVOLATILE_STORAGE, (int) arg0, (int) offset, 0);
}

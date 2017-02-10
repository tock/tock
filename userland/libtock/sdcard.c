// SD card interface

#include "tock.h"
#include "sdcard.h"

struct sdcard_data {
  bool fired;
  uint32_t block_size;
  uint32_t size_in_kB;
  int32_t error;
};

static struct sdcard_data result = {
  .fired = false,
  .block_size = 0,
  .size_in_kB = 0,
  .error = 0,
};

// Internal callback for faking synchronous reads
static void sdcard_cb (int callback_type, int arg1, int arg2, void* ud) {

  struct sdcard_data* result = (struct sdcard_data*) ud;
  switch (callback_type) {
    case 0:
      // card_detection_changed
      result->error = -4; // EOFF
      break;

    case 1:
      // init_done
      result->block_size = arg1;
      result->size_in_kB = arg2;
      result->error = 0;
      break;

    case 2:
      // read_done
      result->error = 0;
      break;

    case 3:
      // write_done
      result->error = 0;
      break;

    case 4:
      // error
      result->error = arg1;
      break;
  }

  result->fired = true;
}

int sdcard_set_callback (subscribe_cb callback, void* callback_args) {
  return subscribe(DRIVER_NUM_SDCARD, 0, callback, callback_args);
}

int sdcard_set_read_buffer (uint8_t* buffer, uint32_t len) {
  return allow(DRIVER_NUM_SDCARD, 0, (void*) buffer, len);
}

int sdcard_set_write_buffer (uint8_t* buffer, uint32_t len) {
  return allow(DRIVER_NUM_SDCARD, 1, (void*) buffer, len);
}

int sdcard_is_installed (void) {
  return command(DRIVER_NUM_SDCARD, 1, 0);
}

int sdcard_initialize (void) {
  return command(DRIVER_NUM_SDCARD, 2, 0);
}

int sdcard_read_block (uint32_t sector) {
  return command(DRIVER_NUM_SDCARD, 3, sector);
}

int sdcard_write_block (uint32_t sector) {
  return command(DRIVER_NUM_SDCARD, 4, sector);
}

int sdcard_initialize_sync (uint32_t* block_size, uint32_t* size_in_kB) {
  int err;
  result.fired = false;
  result.error = 0;

  err = sdcard_set_callback(sdcard_cb, (void*) &result);
  if (err < 0) return err;

  err = sdcard_initialize();
  if (err < 0) return err;

  // wait for callback
  yield_for(&result.fired);

  // copy args
  *block_size = result.block_size;
  *size_in_kB = result.size_in_kB;

  return result.error;
}

int sdcard_read_block_sync (uint32_t sector) {
  int err;
  result.fired = false;
  result.error = 0;

  err = sdcard_set_callback(sdcard_cb, (void*) &result);
  if (err < 0) return err;

  err = sdcard_read_block(sector);
  if (err < 0) return err;

  // wait for callback
  yield_for(&result.fired);

  return result.error;
}

int sdcard_write_block_sync (uint32_t sector) {
  int err;
  result.fired = false;
  result.error = 0;

  err = sdcard_set_callback(sdcard_cb, (void*) &result);
  if (err < 0) return err;

  err = sdcard_write_block(sector);
  if (err < 0) return err;

  // wait for callback
  yield_for(&result.fired);

  return result.error;
}


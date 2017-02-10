#pragma once

#define DRIVER_NUM_SDCARD 15

int sdcard_set_callback (subscribe_cb callback, void* callback_args);
int sdcard_set_read_buffer (uint8_t* buffer, uint32_t len);
int sdcard_set_write_buffer (uint8_t* buffer, uint32_t len);

int sdcard_is_installed (void);
int sdcard_initialize_sync (uint32_t* block_size, uint32_t* size_in_kB);
int sdcard_read_block_sync (uint32_t sector);
int sdcard_write_block_sync (uint32_t sector);


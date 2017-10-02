#pragma once

#include "tock.h"

#ifdef __cplusplus
extern "C" {
#endif

#define DRIVER_NUM_PCA9544A 0x80002

int pca9544a_set_callback(subscribe_cb callback, void* callback_args);

// Set which of the I2C selector's channels are active.
// channels is an 8 bit bitmask
int pca9544a_select_channels(uint32_t channels);

// Disable all channels on the I2C selector.
int pca9544a_disable_all_channels(void);

// Get which channels are asserting interrupts.
int pca9544a_read_interrupts(void);

// Get which channels are currently selected.
int pca9544a_read_selected(void);


//
// Synchronous Versions
//
int pca9544a_select_channels_sync(uint32_t channels);
int pca9544a_disable_all_channels_sync(void);
int pca9544a_read_interrupts_sync(void);
int pca9544a_read_selected_sync(void);

#ifdef __cplusplus
}
#endif

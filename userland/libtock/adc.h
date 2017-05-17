#pragma once

#include <stdint.h>

#include "tock.h"

#ifdef __cplusplus
extern "C" {
#endif

#define DRIVER_NUM_ADC 7

int adc_set_callback(subscribe_cb callback, void* callback_args);
int adc_set_interval_callback(subscribe_cb callback, void* callback_args);
int adc_initialize(void);
int adc_single_sample(uint8_t channel);

// Due to the 32-bit limit of the data parameter to the
// `command()' system call, only the lower 24 bits of
// FREQUENCY are used, leaving 8 bits for CHANNEL.
int adc_cont_sample(uint8_t channel, uint32_t frequency);
int adc_cancel_sampling(void);

int adc_compute_interval(uint32_t interval);

// Synchronous function to read a single ADC sample.
int adc_read_single_sample(uint8_t channel);

// Asynchronous function to read samples at the given FREQUENCY,
// with units of Hz.
// Due to the 32-bit limit of the data parameter to the
// `command()' system call, only the lower 24 bits of
// FREQUENCY are used, leaving 8 bits for CHANNEL.
int adc_read_cont_sample(uint8_t channel, uint32_t frequency, void (*cb)(int));
uint32_t adc_nearest_interval(uint32_t interval);

#ifdef __cplusplus
}
#endif

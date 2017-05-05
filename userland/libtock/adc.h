#pragma once

#include <stdint.h>

#include "tock.h"

#ifdef __cplusplus
extern "C" {
#endif

#define DRIVER_NUM_ADC 7

int adc_set_callback(subscribe_cb callback, void* callback_args);
int adc_initialize(void);
int adc_single_sample(uint8_t channel);
int adc_cont_sample(uint8_t channel, uint32_t interval);

// Synchronous function to read a single ADC sample.
int adc_read_single_sample(uint8_t channel);

// Asynchronous function to read samples at the given interval
int adc_read_cont_sample(uint8_t channel, uint32_t interval, void (*cb)(int));

#ifdef __cplusplus
}
#endif

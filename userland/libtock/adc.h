#pragma once

#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>

#include "tock.h"

#define DRIVER_NUM_ADC 7

int adc_set_callback(subscribe_cb callback, void* callback_args);
int adc_initialize();
int adc_single_sample(uint8_t channel);

// Synchronous function to read a single ADC sample.
int adc_read_single_sample(uint8_t channel);

#ifdef __cplusplus
}
#endif

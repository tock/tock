#include <stdint.h>
#include <stdio.h>

#include "tock.h"
#include "adc.h"

struct adc_data {
  int reading;
  bool fired;
};

struct adc_interval {
  int value;
  bool computed;
};

static struct adc_data result = { .fired = false };
static void(*cont_cb)(int);

// Internal callback for faking synchronous reads
static void adc_cb(__attribute__ ((unused)) int callback_type,
                   __attribute__ ((unused)) int channel,
                   int reading,
                   void* ud) {
  struct adc_data* data = (struct adc_data*) ud;
  data->reading = reading;
  data->fired = true;

  // In continuous mode
  if (cont_cb)
      cont_cb(reading);
}

static struct adc_interval interval_result = { .computed = false };

// Internal callback for determining closest sampling interval
// to that requested by user.
static void adc_interval_cb(__attribute__ ((unused)) int callback_type,
                            __attribute__ ((unused)) int channel,
                            int value,
                            void* ud) {
  struct adc_interval *interval = (struct adc_data*) ud;
  interval->value = value;
  interval->computed = true;
}

int adc_set_callback(subscribe_cb callback, void* callback_args) {
    return subscribe(DRIVER_NUM_ADC, 0, callback, callback_args);
}

int adc_set_interval_callback(subscribe_cb callback, void* callback_args) {
  return subscribe(DRIVER_NUM_ADC, 1, callback, callback_args);
}

int adc_initialize(void) {
    return command(DRIVER_NUM_ADC, 1, 0);
}

int adc_single_sample(uint8_t channel) {
    return command(DRIVER_NUM_ADC, 2, channel);
}

int adc_cont_sample(uint8_t channel, uint32_t frequency) {
  uint32_t chan_freq = (frequency << 8) | (channel);
  return command(DRIVER_NUM_ADC, 3, chan_freq);
}

int adc_cancel_sampling(void) {
    return command(DRIVER_NUM_ADC, 4, 0);
}

int adc_compute_interval(uint32_t interval) {
  return command(DRIVER_NUM_ADC, 5, interval);
}

int adc_read_single_sample(uint8_t channel) {
  int err;

  cont_cb = NULL;
  result.fired = false;
  err = adc_set_callback(adc_cb, (void*) &result);
  if (err < 0) return err;

  err = adc_single_sample(channel);
  if (err < 0) return err;

  // Wait for the ADC callback.
  yield_for(&result.fired);

  return result.reading;
}

int adc_read_cont_sample(uint8_t channel, uint32_t frequency, void (*cb)(int)) {
  int err;

  cont_cb = cb;
  err = adc_set_callback(adc_cb, (void*) &result);
  if (err < 0) return err;

  err = adc_cont_sample(channel, frequency);

  return err;
}

uint32_t adc_nearest_interval(uint32_t interval) {
  int err;

  interval_result.computed = false;
  // Callback used as a mechanism for retrieving the value
  // of the nearest achievable sampling interval.
  err = adc_set_interval_callback(adc_interval_cb, (void*) &interval_result);
  if (err < 0) return err;

  err = adc_compute_interval(interval);
  if (err < 0) return err;

  // Wait for callback.
  yield_for(&interval_result.computed);

  return interval_result.value;
}

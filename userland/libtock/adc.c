#include <stdint.h>

#include "tock.h"
#include "adc.h"

struct adc_data {
  int reading;
  bool fired;
};

struct adc_data result = { .fired = false };

// Internal callback for faking synchronous reads
static void adc_cb(__attribute__ ((unused)) int callback_type,
                   __attribute__ ((unused)) int channel,
                   int reading,
                   void* ud) {
  struct adc_data* result = (struct adc_data*) ud;
  result->reading = reading;
  result->fired = true;
}

int adc_set_callback(subscribe_cb callback, void* callback_args) {
    return subscribe(DRIVER_NUM_ADC, 0, callback, callback_args);
}

int adc_initialize() {
    return command(DRIVER_NUM_ADC, 0, 0);
}

int adc_single_sample(uint8_t channel) {
    return command(DRIVER_NUM_ADC, 1, channel);
}

int adc_read_single_sample(uint8_t channel) {
  int err;

  err = adc_set_callback(adc_cb, (void*) &result);
  if (err < 0) return err;

  err = adc_single_sample(channel);
  if (err < 0) return err;

  // Wait for the ADC callback.
  yield_for(&result.fired);

  return result.reading;
}

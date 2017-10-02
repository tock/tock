// ADC interface

#include <stdint.h>
#include <stdio.h>

#include "adc.h"
#include "tock.h"

// used for creating synchronous versions of functions
//
// fired - set when the callback has been called
// channel - channel that the collected sample corresponds to
// sample - collected sample value, valid if single sample operation
// length - number of collected sample values, valid if multiple sample
//          operation
// buffer - pointer to buffer filled with samples, valid if multiple sample
//          operation
// error - set to FAIL if an invalid callback type is detected
typedef struct {
  bool fired;
  uint8_t channel;
  uint16_t sample;
  uint32_t length;
  uint16_t* buffer;
  int error;
} adc_data_t;

// Internal callback for creating synchronous functions
//
// callback_type - number indicating which type of callback occurred
// arg1, arg2 - meaning varies based on callback_type
// callback_args - user data passed into the set_callback function
//
// Possible callbacks
// SingleSample: single sample operation is complete
//      arg1 - channel number that collected sample corresponds to
//      arg2 - sample value
// MultipleSample: sampling a buffer worth of data is complete
//      arg1 - channel in lower 8 bits,
//             number of samples collected in upper 24 bits
//      arg2 - pointer to buffer filled with samples
// ContinuousSample: a buffer of sample data is ready
//      arg1 - channel in lower 8 bits,
//             number of samples collected in upper 24 bits
//      arg2 - pointer to buffer filled with samples
static void adc_cb(int callback_type,
                   int arg1,
                   int arg2,
                   void* callback_args) {

  adc_data_t* result = (adc_data_t*)callback_args;

  switch (callback_type) {
    case SingleSample:
      result->error   = TOCK_SUCCESS;
      result->channel = arg1;
      result->sample  = arg2;
      break;

    case ContinuousSample:
      result->error   = TOCK_SUCCESS;
      result->channel = arg1;
      result->sample  = arg2;
      break;

    case SingleBuffer:
      result->error   = TOCK_SUCCESS;
      result->channel = (arg1 & 0xFF);
      result->length  = ((arg1 >> 8) & 0xFFFFFF);
      result->buffer  = (uint16_t*)arg2;
      break;

    case ContinuousBuffer:
      result->error   = TOCK_SUCCESS;
      result->channel = (arg1 & 0xFF);
      result->length  = ((arg1 >> 8) & 0xFFFFFF);
      result->buffer  = (uint16_t*)arg2;
      break;

    default:
      result->error = TOCK_FAIL;
      break;
  }

  result->fired = true;
}

// function pointers used for wrapping adc callbacks with the `adc_routing_cb`
// below
static void (*single_sample_callback)(uint8_t, uint16_t, void*) = NULL;
static void (*continuous_sample_callback)(uint8_t, uint16_t, void*) = NULL;
static void (*buffered_sample_callback)(uint8_t, uint32_t, uint16_t*, void*) = NULL;
static void (*continuous_buffered_sample_callback)(uint8_t, uint32_t, uint16_t*, void*) = NULL;

// Internal callback for routing to operation-specific callbacks
//
// callback_type - number indicating which type of callback occurred
// arg1, arg2 - meaning varies based on callback_type
// callback_args - user data passed into the set_callback function
//
// Possible callbacks
// SingleSample: single sample operation is complete
//      arg1 - channel number that collected sample corresponds to
//      arg2 - sample value
// MultipleSample: sampling a buffer worth of data is complete
//      arg1 - channel in lower 8 bits,
//             number of samples collected in upper 24 bits
//      arg2 - pointer to buffer filled with samples
// ContinuousSample: a buffer of sample data is ready
//      arg1 - channel in lower 8 bits,
//             number of samples collected in upper 24 bits
//      arg2 - pointer to buffer filled with samples
static void adc_routing_cb(int callback_type,
                           int arg1,
                           int arg2,
                           void* callback_args) {

  switch (callback_type) {
    case SingleSample:
      if (single_sample_callback) {
        uint8_t channel = (uint8_t)arg1;
        uint16_t sample = (uint16_t)arg2;
        single_sample_callback(channel, sample, callback_args);
      }
      break;

    case ContinuousSample:
      if (continuous_sample_callback) {
        uint8_t channel = (uint8_t)arg1;
        uint16_t sample = (uint16_t)arg2;
        continuous_sample_callback(channel, sample, callback_args);
      }
      break;

    case SingleBuffer:
      if (buffered_sample_callback) {
        uint8_t channel  = (uint8_t)(arg1 & 0xFF);
        uint32_t length  = ((arg1 >> 8) & 0xFFFFFF);
        uint16_t* buffer = (uint16_t*)arg2;
        buffered_sample_callback(channel, length, buffer, callback_args);
      }
      break;

    case ContinuousBuffer:
      if (continuous_buffered_sample_callback) {
        uint8_t channel  = (uint8_t)(arg1 & 0xFF);
        uint32_t length  = ((arg1 >> 8) & 0xFFFFFF);
        uint16_t* buffer = (uint16_t*)arg2;
        continuous_buffered_sample_callback(channel, length, buffer, callback_args);
      }
      break;
  }
}


// ***** System Call Interface *****

int adc_set_callback(subscribe_cb callback, void* callback_args) {
  return subscribe(DRIVER_NUM_ADC, 0, callback, callback_args);
}

int adc_set_buffer(uint16_t* buffer, uint32_t len) {
  // we "allow" byte arrays, so this is actually twice as long
  return allow(DRIVER_NUM_ADC, 0, (void*)buffer, len * 2);
}

int adc_set_double_buffer(uint16_t* buffer, uint32_t len) {
  // we "allow" byte arrays, so this is actually twice as long
  return allow(DRIVER_NUM_ADC, 1, (void*)buffer, len * 2);
}

bool adc_is_present(void) {
  return command(DRIVER_NUM_ADC, 0, 0, 0) >= 0;
}

int adc_channel_count(void) {
  return command(DRIVER_NUM_ADC, 0, 0, 0);
}

int adc_single_sample(uint8_t channel) {
  return command(DRIVER_NUM_ADC, 1, channel, 0);
}

int adc_continuous_sample(uint8_t channel, uint32_t frequency) {
  return command(DRIVER_NUM_ADC, 2, channel, frequency);
}

int adc_buffered_sample(uint8_t channel, uint32_t frequency) {
  return command(DRIVER_NUM_ADC, 3, channel, frequency);
}

int adc_continuous_buffered_sample(uint8_t channel, uint32_t frequency) {
  return command(DRIVER_NUM_ADC, 4, channel, frequency);
}

int adc_stop_sampling(void) {
  return command(DRIVER_NUM_ADC, 5, 0, 0);
}


// ***** Callback Wrappers *****

int adc_set_single_sample_callback(void (*callback)(uint8_t, uint16_t, void*),
                                   void* callback_args) {
  single_sample_callback = callback;
  return adc_set_callback(adc_routing_cb, callback_args);
}

int adc_set_continuous_sample_callback(void (*callback)(uint8_t, uint16_t, void*),
                                       void* callback_args) {
  continuous_sample_callback = callback;
  return adc_set_callback(adc_routing_cb, callback_args);
}

int adc_set_buffered_sample_callback(void (*callback)(uint8_t, uint32_t, uint16_t*, void*),
                                     void* callback_args) {
  buffered_sample_callback = callback;
  return adc_set_callback(adc_routing_cb, callback_args);
}

int adc_set_continuous_buffered_sample_callback(void (*callback)(uint8_t, uint32_t, uint16_t*, void*),
                                                void* callback_args){
  continuous_buffered_sample_callback = callback;
  return adc_set_callback(adc_routing_cb, callback_args);
}


// ***** Synchronous Calls *****

int adc_sample_sync(uint8_t channel, uint16_t* sample) {
  int err;
  adc_data_t result = {0};
  result.fired = false;
  result.error = TOCK_SUCCESS;

  err = adc_set_callback(adc_cb, (void*) &result);
  if (err < TOCK_SUCCESS) return err;

  err = adc_single_sample(channel);
  if (err < TOCK_SUCCESS) return err;

  // wait for callback
  yield_for(&result.fired);

  // copy over result
  *sample = result.sample;

  return result.error;
}

int adc_sample_buffer_sync(uint8_t channel, uint32_t frequency, uint16_t* buffer, uint32_t length) {
  int err;
  adc_data_t result = {0};
  result.fired = false;
  result.error = TOCK_SUCCESS;

  err = adc_set_callback(adc_cb, (void*) &result);
  if (err < TOCK_SUCCESS) return err;

  err = adc_set_buffer(buffer, length);
  if (err < TOCK_SUCCESS) return err;

  err = adc_buffered_sample(channel, frequency);
  if (err < TOCK_SUCCESS) return err;

  // wait for callback
  yield_for(&result.fired);

  // copy over result
  if (result.buffer != buffer) {
    return TOCK_FAIL;
  }

  return result.error;
}


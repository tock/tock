#include <string.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

#include <tock.h>
#include <console.h>
#include <timer.h>
#include <adc.h>

#include <inttypes.h>

void cb(int value);

// Race conditions possible 
static int total = 0;
static int last_sample = 0;
static int num_samples = 0;

void cb(int value) {
  // 12 bit, reference = VCC/2, gain = 0.5
  // millivolts = ((reading * 2) / (2^12 - 1)) * (3.3 V / 2) * 1000
  int millivolts = (value * 3300) / 4095;
  total += millivolts;
  ++num_samples;
  last_sample = millivolts;
}

int main(void) {
  putstr("[Tock] ADC Continuous Test\n");

  // 10000 microsecond sampling interval.
  // 100 Hz sampling frequency
  uint32_t interval = 400;
  uint32_t actual_interval = adc_nearest_interval(interval);
  // Read this asynchronously
  // Sample channel 1 at 100 Hz. This is pin A1.
  adc_read_cont_sample(1, interval, cb);


  printf("Requested sampling interval %" PRIu32 " microseconds. Nearest "
         "supported interval is %" PRIu32 " microseconds.\n",
         interval, actual_interval);

  while (1) {
    // sample for 5 seconds and then stop. XXX fix
    delay_ms(1000);
   // adc_cancel_sampling();
    printf("Measured average of %d over %d samples.\nLast sample is %i mV\n",
            total/num_samples, num_samples, last_sample);
    total = 0;
    num_samples = 0;

  }

  return 0;
}

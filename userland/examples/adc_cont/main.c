#include <string.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

#include <tock.h>
#include <console.h>
#include <timer.h>
#include <adc.h>


// RACE CONDITIONS!
static int total = 0;
static int last_sample = 0;
static int num_samples = 0;

void cb(int value) {
  total += value;
  ++num_samples;
  last_sample = value;
}

int main(void) {
  putstr("[Tock] ADC Continuous Test\n");

  // Setup the ADC
  adc_initialize();
  delay_ms(1000);
  
  // Read this asynchronously
  // Sample channel 1. This is pin A1.
  adc_read_cont_sample(1, 0, cb);

  while (1) {
    delay_ms(1000);
    printf("Measured average of %d over %d samples. Last sample is %d\n",
            total/num_samples, num_samples, last_sample);
    num_samples = 0;
    total = 0;

  }

  return 0;
}

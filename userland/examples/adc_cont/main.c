#include <string.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

#include <tock.h>
#include <console.h>
#include <timer.h>
#include <adc.h>

void cb(int value);

// Race conditions possible 
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

  // Setup the ADC. TODO no, don't do that!
  // Unless you make init common for both?
 // adc_initialize();
 // delay_ms(1000);
  
  // Read this asynchronously
  // Sample channel 1. This is pin A1.
  adc_read_cont_sample(1, 1001, cb);

  while (1) {
    delay_ms(1000);
    printf("Measured average of %d over %d samples. Last sample is %d\n",
            total/num_samples, num_samples, last_sample);
    total = 0;
    num_samples = 0;

  }

  return 0;
}

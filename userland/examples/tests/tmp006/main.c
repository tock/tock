/* vim: set sw=2 expandtab tw=80: */

#include <string.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <stdint.h>
#include <stdbool.h>

#include <tock.h>
#include <console.h>
#include <tmp006.h>


//********************************************************************************
// Example of synchronously reading the temperature sensor
//********************************************************************************

// Repeatedly read from the temperature sensor
static void read_sync (void) {
  int err;
  int16_t temperature;

  while (1) {
    err = tmp006_read_sync(&temperature);
    printf("\tError(%d) [0x%X]\n", err, err);
    printf("\tValue(%d) [0x%X]\n\n", temperature, temperature);
  }
}


//********************************************************************************
// Example of asynchronously reading the temperature sensor with callbacks
//********************************************************************************

int16_t temp_reading;
int32_t error_val;

// Callback to receive asynchronous data
static void temp_callback(int temp_value,
                   int error_code,
                   __attribute__ ((unused)) int unused,
                   __attribute__ ((unused)) void* callback_args) {
  temp_reading = (int16_t) temp_value;
  error_val = error_code;
}

// Start periodic temperature sampling, then print data, sleeping in between
//  samples. Note that you MUST yield() or else callbacks will never be serviced
static void read_periodic (void) {
  int err;

  // start sampling at 1 sample per second (0x2)
  putstr("Start Subscribe.\n");
  err = tmp006_start_sampling(0x2, temp_callback, NULL);
  if (error_val != 0) {
    printf("\tError(%d) [0x%X]\n\n", err, err);
  }

  while (1) {
    // yield for callbacks
    putstr("Sleeping...\n");
    yield();

    // print new temp reading
    printf("\tValue(%d) [0x%X]\n\n", temp_reading, (unsigned) temp_reading);
    if (error_val != 0) {
      printf("\tError(%lu) [0x%X]\n\n", error_val, (uint16_t) error_val);
    }

    // reset values
    temp_reading = 0xDEAD;
    error_val = 0;
  }
}


//********************************************************************************
// Demonstration code for the TMP006 temperature sensor
//********************************************************************************

// Demonstrate both synchronous and asynchronous reading from a driver
int main(void) {
  putstr("Welcome to Tock in C (with libc)\nReading temperature...\n");

  // uncomment whichever example you want
  read_sync();
  //read_periodic();
}

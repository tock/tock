/* vim: set sw=2 expandtab tw=80: */

#include <string.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <stdint.h>
#include <stdbool.h>

#include <tock.h>
#include <firestorm.h>
#include <tmp006.h>


//********************************************************************************
// example of synchronously reading the temperature sensor
//********************************************************************************

// repeatedily read from the temperature sensor
void read_sync (void) {
  int err;
  int16_t temperature;

  while (1) {
    err = tmp006_read_sync(&temperature);
    {
      char buf[64];
      snprintf(buf, 64, "\tError(%d) [0x%X]\n", err, err);
      putstr(buf);
    }

    {
      char buf[64];
      sprintf(buf, "\tValue(%d) [0x%X]\n", temperature, temperature);
      putstr(buf);
    }

    putstr("\n");
  }
}


//********************************************************************************
// example of asynchronously reading the temperature sensor with callbacks
//********************************************************************************

int16_t temp_reading;
int32_t error_val;

// callback to receive asynchronous data
CB_TYPE temp_callback(int temp_value, int error_code, int unused, void* callback_args) {
  temp_reading = (int16_t)temp_value;
  error_val = error_code;
}

// start periodic temperature sampling, then print data, sleeping in between
//  samples. Note that you MUST wait() or else callbacks will never be serviced
void read_periodic (void) {
  int err;

  // start sampling at 1 sample per second (0x2)
  putstr("Start Subscribe.\n");
  err = tmp006_start_sampling(0x2, temp_callback, NULL);
  if (error_val != 0) {
    char buf[64];
    snprintf(buf, 64, "\tError(%d) [0x%X]\n\n", err, err);
    putstr(buf);
  }

  while (1) {
    // yield for callbacks
    putstr("Sleeping...\n");
    wait();

    // print new temp reading
    {
      char buf[64];
      sprintf(buf, "\tValue(%d) [0x%X]\n\n", temp_reading, temp_reading);
      putstr(buf);
    }
    if (error_val != 0) {
      char buf[64];
      snprintf(buf, 64, "\tError(%d) [0x%X]\n\n", error_val, error_val);
      putstr(buf);
    }

    // reset values
    temp_reading = 0xDEAD;
    error_val = 0;
  }
}


//********************************************************************************
// Demonstration code for the TMP006 temperature sensor
//********************************************************************************

// demonstrate both synchronous and asynchronous reading from a driver
void main() {
  putstr("Welcome to Tock in C (with libc)\nReading temperature...\n");

  // uncomment whichever example you want
  read_sync();
  //read_periodic();
}


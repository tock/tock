/* vim: set sw=2 expandtab tw=80: */

#include <stdio.h>
#include <stdbool.h>
#include <stdint.h>

#include "tock.h"
#include "console.h"
#include "gpio.h"
#include "led.h"

typedef struct {
  uint8_t pir;
  uint8_t reed_switch;
} SensorData_t;

static SensorData_t sensor_data = {
  .pir = 0,
  .reed_switch = 0,
};

// callback for gpio interrupts
static void gpio_cb (int pin_num,
              int pin_val,
              __attribute__ ((unused)) int unused,
              __attribute__ ((unused)) void* userdata) {

  // save sensor data
  if (pin_num == 1) {
    // interrupt from pir sensor
    sensor_data.pir = pin_val;

  } else if (pin_num == 2) {
    // interrupt from reed switch
    sensor_data.reed_switch = pin_val;
  }
}

// This application reads from multiple sources:
//  * GPIO input from PIR sensor (motion)
//  * GPIO input from Hall-effect sensor (door open/close)
//  * Accelerometer (movement)
//  and makes that available over RF communication
int main(void) {
  putstr("*********************\n");
  putstr("Security Application\n");

  // configure pins
  gpio_interrupt_callback(gpio_cb, NULL);
  gpio_enable_interrupt(0, PullNone, Change);
  gpio_enable_interrupt(1, PullUp, Change);

  // configure accelerometer
  //TODO

  // configure radio
  //TODO

  while (1) {
    yield();
    led_toggle(0);

    {
      char buf[64];
      sprintf(buf, "\tPIR:\t\t%d\n\tReed Switch:\t%d\n\n",
          sensor_data.pir, sensor_data.reed_switch);
      putstr(buf);
    }
  }

  return 0;
}


#include "si7021.h"

#include <isl29035.h>
#include <stdio.h>
#include <stdbool.h>
#include <timer.h>

static int intensity = 0;
static int temperature = 0;
static int humidity = 0;

static int8_t status = 0b00; // bit 1 for intensity, 2 for temp + humidity

static void print_measurements(void) {
  printf("Intensity: %d; ", intensity);
  printf("Temperature: %d.%d; ", temperature / 100, temperature % 100);
  printf("Humidity: %d.%d\n", humidity / 100, humidity % 100);
}

static void intensity_cb(
    int intensity_t,
    int unused1 __attribute__((unused)),
    int unused2 __attribute__((unused)),
    void* ud __attribute__((unused)) ) {
  intensity = intensity_t;
  status |= 0b01;
  if (status == 0b11) {
    print_measurements();
  }
}

static void temp_cb(
    int temperature_t,
    int humidity_t,
    int unused2 __attribute__((unused)),
    void* ud __attribute__((unused)) ) {
  status |= 0b10;
  temperature = temperature_t;
  humidity = humidity_t;
  if (status == 0b11) {
    print_measurements();
  }
}

static void timer_fired(
    int arg0 __attribute__((unused)),
    int arg1 __attribute__((unused)),
    int arg2 __attribute__((unused)),
    void* ud __attribute__((unused)) ) {
  status = 0;
  isl29035_start_intensity_reading();
  si7021_get_temperature_humidity();
}

int main(void) {
  isl29035_subscribe(intensity_cb, NULL);
  si7021_set_callback(temp_cb, NULL);
  status = 0;
  isl29035_start_intensity_reading();
  si7021_get_temperature_humidity();

  // Setup periodic timer
  timer_subscribe(timer_fired, NULL);
  timer_start_repeating(1000);

  return 0;
}

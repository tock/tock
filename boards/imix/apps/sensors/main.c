#include "si7021.h"

#include <isl29035.h>
#include <stdio.h>
#include <stdbool.h>
#include <timer.h>

static int intensity = 0;
static int temperature = 0;
static int humidity = 0;

static int8_t status = 0b00; // bit 1 for intensity, 2 for temp + humidity

void print_measurements() {
  printf("Intensity: %d; ", intensity);
  printf("Temperature: %d.%d; ", temperature / 100, temperature % 100);
  printf("Humidity: %d.%d\n", humidity / 100, humidity % 100);
}

void intensity_cb(int intensity_t, int unused1, int unused2, void* ud) {
  intensity = intensity_t;
  status |= 0b01;
  if (status == 0b11) {
    print_measurements();
  }
}

void temp_cb(int temperature_t, int humidity_t, int unused2, void* ud) {
  status |= 0b10;
  temperature = temperature_t;
  humidity = humidity_t;
  if (status == 0b11) {
    print_measurements();
  }
}

void timer_fired(int arg0, int arg1, int arg2, void* ud) {
  status = 0;
  isl29035_start_intensity_reading();
  si7021_get_temperature_humidity();
}

int main() {
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

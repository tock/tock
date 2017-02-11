#include <stdio.h>
#include <stdbool.h>

#include <timer.h>
#include <isl29035.h>
#include <si7021.h>
#include <FXOS8700CQ.h>
#include <button.h>
#include <led.h>
#include <adc.h>
#include <gpio.h>

// Callback for button presses.
//   btn_num: The index of the button associated with the callback
//   val: 0 if pressed, 1 if depressed
static void button_callback(__attribute__ ((unused)) int btn_num,
                            int val,
                            __attribute__ ((unused)) int arg2,
                            __attribute__ ((unused)) void *ud) {
  if (val == 0) {
    led_on(1); // green
  } else {
    led_off(1);
  }
}

static void timer_fired(__attribute__ ((unused)) int arg0,
                 __attribute__ ((unused)) int arg1,
                 __attribute__ ((unused)) int arg2,
                 __attribute__ ((unused)) void* ud) {
  int temp;
  unsigned humi;
  uint32_t accel_mag;
  int light;
  int a0, a1, a2, a3, a4, a5;
  int d0, d1, d6, d7;

  si7021_get_temperature_humidity_sync(&temp, &humi);
  accel_mag = FXOS8700CQ_read_accel_mag();
  light = isl29035_read_light_intensity();

  // A0-A5
  a0 = (adc_read_single_sample(0) * 3300) / 4095;
  a1 = (adc_read_single_sample(1) * 3300) / 4095;
  a2 = (adc_read_single_sample(3) * 3300) / 4095;
  a3 = (adc_read_single_sample(4) * 3300) / 4095;
  a4 = (adc_read_single_sample(5) * 3300) / 4095;
  a5 = (adc_read_single_sample(6) * 3300) / 4095;

  // D0, D1, D6, D7
  d0 = gpio_read(0);
  d1 = gpio_read(1);
  d6 = gpio_read(2);
  d7 = gpio_read(3);

  printf("[Hail Sensor Reading]\n");
  printf("  Temperature:  %d 1/100 degrees C\n", temp);
  printf("  Humidity:     %u 0.01%%\n", humi);
  printf("  Light:        %d\n", light);
  printf("  Acceleration: %lu\n", accel_mag);
  printf("  A0:           %d mV\n", a0);
  printf("  A1:           %d mV\n", a1);
  printf("  A2:           %d mV\n", a2);
  printf("  A3:           %d mV\n", a3);
  printf("  A4:           %d mV\n", a4);
  printf("  A5:           %d mV\n", a5);
  printf("  D0:           %d\n", d0);
  printf("  D1:           %d\n", d1);
  printf("  D6:           %d\n", d6);
  printf("  D7:           %d\n", d7);
  printf("\n");
}

int main(void) {
  printf("[Hail] Test App!\n");
  printf("[Hail] Samples all sensors and transmits over BLE.\n");
  printf("[Hail] Button controls LED.\n");

  // Setup periodic timer
  timer_subscribe(timer_fired, NULL);
  timer_start_repeating(1000);

  // Enable button callbacks
  button_subscribe(button_callback, NULL);
  button_enable_interrupt(0);

  // Setup the ADC
  adc_initialize();

  // Setup D0, D1, D6, D7
  gpio_enable_input(0, PullDown); // D0
  gpio_enable_input(1, PullDown); // D1
  gpio_enable_input(2, PullDown); // D6
  gpio_enable_input(3, PullDown); // D7

  return 0;
}

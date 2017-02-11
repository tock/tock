/* vim: set sw=2 expandtab tw=80: */

#include <string.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

#include <tock.h>
#include <console.h>
#include <gpio.h>
#include <timer.h>
#include <led.h>

// callback for timers
static void timer_cb (__attribute__ ((unused)) int arg0,
               __attribute__ ((unused)) int arg1,
               __attribute__ ((unused)) int arg2,
               __attribute__ ((unused)) void* userdata) {
}

//**************************************************
// GPIO output example
//**************************************************
static void gpio_output(void) {
  putstr("Periodically blinking LED\n");

  // Start repeating timer
  timer_subscribe(timer_cb, NULL);
  timer_start_repeating(500);

  while (1) {
    led_toggle(0);
    yield();
  }
}

//**************************************************
// GPIO input example
//**************************************************
static void gpio_input(void) {
  putstr("Periodically reading value of the GPIO 0 pin\n");
  putstr("Jump pin high to test (defaults to low)\n");

  // set LED pin as input and start repeating timer
  // pin is configured with a pull-down resistor, so it should read 0 as default
  gpio_enable_input(0, PullDown);
  timer_subscribe(timer_cb, NULL);
  timer_start_repeating(500);

  while (1) {
    // print pin value
    int pin_val = gpio_read(0);
    printf("\tValue(%d)\n", pin_val);
    yield();
  }
}

//**************************************************
// GPIO interrupt example
//**************************************************
static void gpio_cb (__attribute__ ((unused)) int pin_num,
              __attribute__ ((unused)) int arg2,
              __attribute__ ((unused)) int arg3,
              __attribute__ ((unused)) void* userdata) {
}

static void gpio_interrupt(void) {
  putstr("Print GPIO 0 pin reading whenever its value changes\n");
  putstr("Jump pin high to test\n");

  // set callback for GPIO interrupts
  gpio_interrupt_callback(gpio_cb, NULL);

  // set LED as input and enable interrupts on it
  gpio_enable_interrupt(0, PullDown, Change);

  while (1) {
    yield();
    putstr("\tGPIO Interrupt!\n");
  }
}


int main(void) {
  putstr("*********************\n");
  putstr("GPIO Test Application\n");

  // uncomment whichever example you want
  // gpio_output();
  // gpio_input();
  gpio_interrupt();

  return 0;
}

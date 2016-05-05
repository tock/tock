/* vim: set sw=2 expandtab tw=80: */

#include <string.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

#include <tock.h>
#include <firestorm.h>
#include <gpio.h>

void gpio_cb (int pin_num, int arg2, int arg3, void* userdata) {
  // set gpio pin raw to indicate event received
  putstr("Got interrupt\n");
}

void main() {
  putstr("*********************\n");
  putstr("Event Overhead - GPIO\n");
  putstr("*********************\n");

  // set callback for GPIO interrupts
  gpio_interrupt_callback(gpio_cb, NULL);

  // set P2 as interrupt input, active high
  gpio_enable_interrupt(P2, PullNone, RisingEdge);

  putstr("Ready to start\n");
}


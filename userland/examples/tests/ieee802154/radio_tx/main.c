#include <stdbool.h>

#include "gpio.h"
#include "led.h"
#include "radio.h"
#include "timer.h"
#include "tock.h"

#define BUF_SIZE 60
char packet[BUF_SIZE];
bool toggle = true;

int main(void) {
  int i;
  for (i = 0; i < BUF_SIZE; i++) {
    packet[i] = i;
  }
  gpio_enable_output(0);
  radio_set_addr(0x1540);
  radio_set_pan(0xABCD);
  radio_commit();
  radio_init();
  while (1) {
    led_toggle(0);
    int err = radio_send(0x0802, packet, BUF_SIZE);
    if (err != TOCK_SUCCESS) {
      gpio_toggle(0);
    }
    delay_ms(250);
  }
}

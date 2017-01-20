#include <stdbool.h>

#include "led.h"
#include "radio.h"
#include "timer.h"

#define BUF_SIZE 60
char packet[BUF_SIZE];
bool toggle = true;

int main(void) {
  int i;
  for (i = 0; i < BUF_SIZE; i++) {
    packet[i] = i;
  }
  radio_init();
  radio_set_addr(0x1540);
  radio_set_pan(0x802);
  while (1) {
    led_toggle(0);
    radio_send(0xFFFF, packet, BUF_SIZE);
    delay_ms(250);
  }
}

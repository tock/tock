#include <stdbool.h>
#include <stdio.h>
#include <led.h>
#include <radio.h>

#define BUF_SIZE 60
char packet[BUF_SIZE];
bool toggle = true;

int main(void) {
  int i;
  //printf("Starting 802.15.4 packet reception app.\n");
  for (i = 0; i < BUF_SIZE; i++) {
    packet[i] = i;
  }
  radio_set_addr(0x802);
  radio_set_pan(0xABCD);
  radio_commit();
  while (1) {
    if (radio_receive(packet, BUF_SIZE) >= 0) {
      radio_send(0xFFFF, packet, BUF_SIZE);
    }
    led_toggle(0);
  }
}

#include <stdbool.h>
#include <stdio.h>

#include "led.h"
#include "ieee802154.h"

#define BUF_SIZE 60
char packet[BUF_SIZE];
bool toggle = true;

int main(void) {
  int i;
  // printf("Starting 802.15.4 packet reception app.\n");
  for (i = 0; i < BUF_SIZE; i++) {
    packet[i] = i;
  }
  ieee802154_set_address(0x802);
  ieee802154_set_pan(0xABCD);
  ieee802154_config_commit();
  ieee802154_up();
  while (1) {
    if (ieee802154_receive_sync(packet, BUF_SIZE) >= 0) {
      ieee802154_send(0xFFFF,
                      SEC_LEVEL_NONE,
                      0,
                      NULL,
                      packet,
                      BUF_SIZE);
    }
    led_toggle(0);
  }
}

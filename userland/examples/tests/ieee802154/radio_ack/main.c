#include <stdbool.h>
#include <stdio.h>

#include "led.h"
#include "ieee802154.h"
#include "timer.h"

#define RADIO_FRAME_SIZE 129
#define BUF_SIZE 60
char packet_rx[RADIO_FRAME_SIZE];
char packet_tx[BUF_SIZE];
bool toggle = true;

static void callback(__attribute__ ((unused)) int pans,
                     __attribute__ ((unused)) int dst_addr,
                     __attribute__ ((unused)) int src_addr,
                     __attribute__ ((unused)) void* ud) {
  led_toggle(0);
  ieee802154_receive(callback, packet_rx, RADIO_FRAME_SIZE);
}

int main(void) {
  int i;
  char counter = 0;
  // printf("Starting 802.15.4 packet reception app.\n");
  for (i = 0; i < RADIO_FRAME_SIZE; i++) {
    packet_rx[i] = 0;
  }
  for (i = 0; i < BUF_SIZE; i++) {
    packet_tx[i] = i;
  }
  ieee802154_set_address(0x802);
  ieee802154_set_pan(0xABCD);
  ieee802154_config_commit();
  ieee802154_up();
  ieee802154_receive(callback, packet_rx, RADIO_FRAME_SIZE);
  while (1) {
    int err = ieee802154_send(0x0802,
                              SEC_LEVEL_NONE,
                              0,
                              NULL,
                              packet_tx,
                              BUF_SIZE);
    printf("Packet sent, return code: %i\n", err);
    counter++;
    packet_tx[0] = counter;
    delay_ms(4000);
  }
}

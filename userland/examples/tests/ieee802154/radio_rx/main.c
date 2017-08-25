#include <stdbool.h>
#include <stdio.h>

#include "led.h"
#include "ieee802154.h"
#include "timer.h"
#include "tock.h"

#define RADIO_FRAME_SIZE 129
#define BUF_SIZE 60
char packet_rx[RADIO_FRAME_SIZE];

static void callback(__attribute__ ((unused)) int pans,
                     __attribute__ ((unused)) int dst_addr,
                     __attribute__ ((unused)) int src_addr,
                     __attribute__ ((unused)) void* ud) {
  led_toggle(0);

#define PRINT_PAYLOAD 0
#if PRINT_PAYLOAD
  int payload_offset = ieee802154_get_frame_payload_offset(packet_rx);
  int payload_length = ieee802154_get_frame_payload_offset(packet_rx);
  printf("Received packet with payload of %d bytes from offset %d\n", payload_length, payload_offset);
  int i;
  for (i = 0; i < payload_length; i++) {
    printf("%02x%c", packet_rx[payload_offset + i],
           ((i + 1) % 16 == 0 || i + 1 == payload_length) ? '\n' : ' ');
  }
#endif

  ieee802154_receive(callback, packet_rx, RADIO_FRAME_SIZE);
}

int main(void) {
  int i;
  /* printf("Starting 802.15.4 packet reception app.\n"); */
  for (i = 0; i < RADIO_FRAME_SIZE; i++) {
    packet_rx[i] = 0;
  }
  ieee802154_set_address(0x802);
  ieee802154_set_pan(0xABCD);
  ieee802154_config_commit();
  ieee802154_up();
  ieee802154_receive(callback, packet_rx, RADIO_FRAME_SIZE);
  while (1) {
    delay_ms(4000);
  }
}

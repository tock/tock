#include <stdbool.h>
#include <stdio.h>

#include "led.h"
#include "timer.h"
#include "tock.h"

#include <ieee802154.h>
#include <udp.h>

// UDP sample packet reception app.
// Continually receives frames at the specified address and port.

char packet_rx[IEEE802154_FRAME_LEN];
sock_handle_t* handle;
sock_addr_t *incoming_addr;

void print_ipv6(ipv6_addr_t *);

void print_ipv6(ipv6_addr_t *ipv6_addr) {
    for(int j = 0; j < 14; j+=2)
        printf("%02x%02x:", ipv6_addr->addr[j], ipv6_addr->addr[j+1]);
    printf("%02x%02x", ipv6_addr->addr[14], ipv6_addr->addr[15]);
}

static void callback(int payload_len,
                     __attribute__ ((unused)) int arg2,
                     __attribute__ ((unused)) int arg3,
                     __attribute__ ((unused)) void* ud) {
  led_toggle(0);

#define PRINT_STRING 1
#if PRINT_STRING
  printf("%.*s\n", payload_len, packet_rx);
#else
  for (i = 0; i < payload_len; i++) {
    printf("%02x%c", packet_rx[i],
           ((i + 1) % 16 == 0 || i + 1 == payload_len) ? '\n' : ' ');
  }
#endif //PRINT_STRING

  udp_recv_from(callback, handle, packet_rx, IEEE802154_FRAME_LEN, incoming_addr);
}

int main(void) {

  ipv6_addr_t ifaces[10];
  udp_list_ifaces(ifaces, 10);

  sock_addr_t addr = {
    ifaces[1],
    16123
  };

  printf("Opening socket on ");
  print_ipv6(&ifaces[1]);
  printf(" : %d\n", addr.port);
  sock_handle_t h;
  udp_socket(&h, &addr);
  handle = &h;

  sock_addr_t in = {
    ifaces[0],
    15123
  };
  incoming_addr = &in;
  printf("Listening for UDP packets from ");
  print_ipv6(&ifaces[0]);
  printf(" : %d\n", incoming_addr->port);

  /*
  ieee802154_set_address(0x802);
  ieee802154_set_pan(0xABCD);
  ieee802154_config_commit();
  ieee802154_up();
  */

  memset(packet_rx, 0, IEEE802154_FRAME_LEN);
  udp_recv_from(callback, handle, packet_rx, IEEE802154_FRAME_LEN, incoming_addr);
  while (1) {
    delay_ms(4000);
  }

  // udp_close(handle);
}

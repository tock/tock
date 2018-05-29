#include <stdbool.h>
#include <stdio.h>

#include <ambient_light.h>
#include <humidity.h>
#include <temperature.h>
#include <timer.h>

#include <ieee802154.h>
#include <udp.h>

void print_ipv6(ipv6_addr_t *);

void print_ipv6(ipv6_addr_t *ipv6_addr) {
    for(int j = 0; j < 14; j+=2)
        printf("%02x%02x:", ipv6_addr->addr[j], ipv6_addr->addr[j+1]);
    printf("%02x%02x", ipv6_addr->addr[14], ipv6_addr->addr[15]);
}

int main(void) {
  printf("[Sensors] Starting Sensors App.\n");
  printf("[Sensors] All available sensors on the platform will be sampled.\n");

  unsigned int humi;
  int temp, lux;
  char packet[64];

  /*
  ieee802154_set_address(0x1540);
  ieee802154_set_pan(0xABCD);
  ieee802154_config_commit();
  ieee802154_up();
  */

  ipv6_addr_t ifaces[10];
  udp_list_ifaces(ifaces, 10);

  sock_handle_t handle;
  sock_addr_t addr = {
    ifaces[0],
    15123
  };

  printf("Opening socket on ");
  print_ipv6(&ifaces[0]);
  printf(" : %d\n", addr.port);
  udp_socket(&handle, &addr);

  sock_addr_t destination = {
    ifaces[1],
    16123
  };

  while (1) {
    temperature_read_sync(&temp);
    humidity_read_sync(&humi);
    ambient_light_read_intensity_sync(&lux);

    int len = snprintf(packet, sizeof(packet), "%d deg C; %d%%; %d lux;\n",
                       temp, humi, lux);

    printf("Sending packet (length %d) --> ", len);
    print_ipv6(&(destination.addr));
    printf(" : %d\n", destination.port);
    ssize_t bytes_sent = udp_send_to(&handle, packet, len, &destination);
    if (bytes_sent < 0) {
        printf("    UDP TX ERROR: %d\n", bytes_sent);
    } else {
        printf("    bytes sent: %d\n", bytes_sent);
    }

    /*
    switch (err) {
      case TOCK_SUCCESS:
        printf("Sent and acknowledged\n");
        break;
      case TOCK_ENOACK:
        printf("Sent but not acknowledged\n");
        break;
      default:
        printf("Error sending packet %d\n", err);
    }
    */

    delay_ms(1000);
  }

  udp_close(&handle);
}

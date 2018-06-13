#include <stdbool.h>
#include <stdio.h>

#include <ambient_light.h>
#include <humidity.h>
#include <temperature.h>
#include <timer.h>

#include <ieee802154.h>

int main(void) {
  printf("[Sensors] Starting Sensors App.\n");
  printf("[Sensors] All available sensors on the platform will be sampled.\n");

  unsigned int humi;
  int temp, lux;

  char packet[64];

  /* { IEEE802.15.4 configuration... temporary until we have full IP */
  ieee802154_set_address(0x1540);
  ieee802154_set_pan(0xABCD);
  ieee802154_config_commit();
  ieee802154_up();
  /* } IEEE802.15.4 configuration */

  while (1) {
    temperature_read_sync(&temp);
    humidity_read_sync(&humi);
    ambient_light_read_intensity_sync(&lux);

    int len = snprintf(packet, sizeof(packet), "%d deg C; %d%%; %d lux;\n",
                       temp, humi, lux);

    int err = ieee802154_send(0x0802, // destination address (short MAC address)
                              SEC_LEVEL_NONE, // No encryption
                              0, // unused since SEC_LEVEL_NONE
                              NULL, // unused since SEC_LEVEL_NONE
                              packet,
                              len);
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

    delay_ms(1000);
  }
}

#include <simple_ble.h>
#include <stdbool.h>
#include <stdio.h>
#include <tock.h>

#define DEVICE_NAME_SIZE 6

/*******************************************************************************
 * MAIN
 ******************************************************************************/

int main(void) {
  int err;
  printf("[Tutorial] BLE Advertising\n");

  // declarations of variables to be used in this BLE example application
  uint16_t advertising_interval_ms = 200;
  uint8_t device_name[] = "CoolOS";

  // configure LE only and discoverable
  // configure advertisement address as 1,2,3,4,5,6
  printf(" - Initializing BLE... %s\n", device_name);
  err = ble_initialize(advertising_interval_ms, true);
  if (err < TOCK_SUCCESS)
    printf("ble_initialize, error: %s\r\n", tock_strerror(err));

  // configure device name as TockOS
  printf(" - Setting the device name...%s\n", device_name);
  err = ble_advertise_name(device_name, DEVICE_NAME_SIZE);
  if (err < TOCK_SUCCESS)
    printf("ble_advertise_name, error: %s\r\n", tock_strerror(err));

  // start advertising
  printf(" - Begin advertising as %s\n", device_name);
  err = ble_start_advertising();
  if (err < TOCK_SUCCESS)
    printf("ble_start_advertising, error: %s\r\n", tock_strerror(err));

  // configuration complete
  printf("Now advertising every %d ms as '%s'\n", advertising_interval_ms,
         device_name);
  return 0;
}

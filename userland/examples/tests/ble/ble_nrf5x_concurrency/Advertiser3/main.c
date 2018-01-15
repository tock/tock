#include <simple_ble.h>
#include <stdbool.h>
#include <stdio.h>
#include <tock.h>

// Sizes in bytes
#define DEVICE_NAME_SIZE 7

int main(void) {
  int err;

  uint16_t advertising_interval_ms = 20;
  uint8_t device_name[]            = "TockOS3";

  // configure LE only and discoverable
  printf(" - Initializing BLE... %s\n", device_name);
  err = ble_initialize(advertising_interval_ms, true);
  if (err < TOCK_SUCCESS)
    printf("ble_initialize, error: %s\r\n", tock_strerror(err));

  printf(" - Setting the device name... %s\n", device_name);
  err = ble_advertise_name(device_name, DEVICE_NAME_SIZE);
  if (err < TOCK_SUCCESS)
    printf("ble_advertise_name, error: %s\r\n", tock_strerror(err));

  // start advertising
  printf(" - Begin advertising! %s\n", device_name);
  err = ble_start_advertising();
  if (err < TOCK_SUCCESS)
    printf("ble_start_advertising, error: %s\r\n", tock_strerror(err));

  // configuration complete
  printf("Now advertising every %d ms as '%s'\n", advertising_interval_ms,
         device_name);
  return 0;
}

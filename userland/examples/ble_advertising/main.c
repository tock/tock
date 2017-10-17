#include <simple_ble.h>
#include <stdbool.h>
#include <stdio.h>
#include <tock.h>

// Sizes in bytes
#define DEVICE_NAME_SIZE            6
#define UUIDS_SIZE                  4
#define MANUFACTURER_DATA_SIZE      2
#define FAKE_TEMPERATURE_DATA_SIZE  2

/*******************************************************************************
 * MAIN
 ******************************************************************************/

int main (void) {
  int err;
  printf("[Tutorial] BLE Advertising\n");

  // declarations of variables to be used in this BLE example application
  uint16_t advertising_interval_ms = 300;
  uint8_t device_name[]            = "TockOS";
  uint16_t uuids[] = {0x1800, 0x1809};
  uint8_t manufacturer_data[]     = {0x13, 0x37};
  uint8_t fake_temperature_data[] = {0x00, 0x00};

  // configure advertisement interval to 300ms
  // configure LE only and discoverable
  // configure advertisement address as 1,2,3,4,5,6
  printf(" - Initializing BLE...\n");
  err = ble_initialize(advertising_interval_ms, true);
  if (err < TOCK_SUCCESS) printf("ble_initialize, error: %s\r\n", tock_strerror(err));

  // configure device name as TockOS
  printf(" - Setting the device name...\n");
  err = ble_advertise_name(device_name, DEVICE_NAME_SIZE);
  if (err < TOCK_SUCCESS) printf("ble_advertise_name, error: %s\r\n", tock_strerror(err));

  // configure list of UUIDs
  printf(" - Setting the device UUID...\n");
  err = ble_advertise_uuid16(uuids, UUIDS_SIZE);
  if (err < TOCK_SUCCESS) printf("ble_advertise_uuid16, error: %s\r\n", tock_strerror(err));

  // configure manufacturer data
  printf(" - Setting manufacturer data...\n");
  err = ble_advertise_manufacturer_specific_data(manufacturer_data, MANUFACTURER_DATA_SIZE);
  if (err < TOCK_SUCCESS) printf("ble_advertise_manufacturer_specific_data, error: %s\r\n", tock_strerror(err));

  // configure service data
  printf(" - Setting service data...\n");
  err = ble_advertise_service_data(uuids[1], fake_temperature_data, FAKE_TEMPERATURE_DATA_SIZE);
  if (err < TOCK_SUCCESS) printf("ble_advertise_service_data, error: %s\r\n", tock_strerror(err));

  // start advertising
  printf(" - Begin advertising!\n");
  err = ble_start_advertising();
  if (err < TOCK_SUCCESS) printf("ble_start_advertising, error: %s\r\n", tock_strerror(err));

  // configuration complete
  printf("Now advertising every %d ms as '%s'\n", advertising_interval_ms, device_name);
  return 0;
}

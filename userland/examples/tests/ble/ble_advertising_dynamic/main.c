#include <rng.h>
#include <simple_ble.h>
#include <stdbool.h>
#include <stdio.h>
#include <timer.h>
#include <tock.h>

// Sizes in bytes
#define DEVICE_NAME_SIZE 6
#define MANUFACTURER_DATA_SIZE 2

/*******************************************************************************
 * MAIN
 ******************************************************************************/

int main(void) {
  int err;
  printf("[Test] BLE Dynamic Advertising\r\n");

  uint16_t advertising_interval_ms = 500;
  uint8_t device_name[]            = "TockOS";
  uint8_t rand[1];
  uint8_t manufacturer_data[] = {0x13, 0x37};
  uint8_t device_name2[]      = "CoolOS";

  printf(" - Initializing BLE...\r\n");
  err = ble_initialize(advertising_interval_ms, true);
  if (err < TOCK_SUCCESS)
    printf("ble_initialize, error: %s\r\n", tock_strerror(err));

  printf(" - Configure Tx Power: %d dBm ...\r\n", (int8_t)NEGATIVE_20_DBM);
  err = ble_set_tx_power(NEGATIVE_20_DBM);
  if (err < TOCK_SUCCESS)
    printf("ble_set_tx_power, error: %s\r\n", tock_strerror(err));


  // start advertising
  printf(" - Begin advertising! %s\r\n", device_name);
  err = ble_start_advertising();
  if (err < TOCK_SUCCESS)
    printf("ble_start_advertising, error: %s\r\n", tock_strerror(err));

  // configuration complete
  printf("Now advertising every %d ms as '%s'\r\n", advertising_interval_ms,
         device_name);

  bool toggle = true;

  for ( ; ; ) {
    // this should always return error if the logic is correct 
    if (ble_start_advertising() >= TOCK_SUCCESS) {
      printf("TEST FAILED\r\n");
      return TOCK_FAIL;
    }

    // Reset advertisement buffer
    // Use a while loop because the driver may be busy and we don't know that */
    while (ble_reset_advertisement() != TOCK_SUCCESS) ;

    // configure advertisement name the buffer should have an offset of 16 at
    // this point
    if (toggle == true) {
      while (ble_advertise_name(device_name2, DEVICE_NAME_SIZE) != TOCK_SUCCESS) ;
      while (ble_set_tx_power(ZERO_DBM) != TOCK_SUCCESS) ;
      while (ble_set_advertisement_interval(100) != TOCK_SUCCESS) ;
      printf("ble_dynamic_update: \r\ndevice_name: %s\t power_level: %d dBm\t advertising_interval_ms: %d\r\n", device_name2, ZERO_DBM, 100);
      
    }else {
      while (ble_advertise_name(device_name, DEVICE_NAME_SIZE) != TOCK_SUCCESS) ;
      while (ble_set_tx_power(POSITIVE_4_DBM) != TOCK_SUCCESS) ;
      while (ble_set_advertisement_interval(300) != TOCK_SUCCESS) ;
      printf("ble_dynamic_update: \r\ndevice_name: %s\t power_level: %d dBm \t, advertising_interval_ms: %d\r\n", device_name, POSITIVE_4_DBM, 300);
    }

    // generate a random value between 0 - 20
    rng_sync(rand, 4, 4);
    int r = rand[0] % 20;

    // Each iteration will the advertisement buffer with 4 bytes
    // Len | 0xff | 0x13 | 0x37
    // So, 39 - 16 = 23
    // r >= 8 could create a buffer overflow in the kernel if the logic is
    // broken, thus if the kernel crashes => test failed
    for (int i = 0; i < r; i++) {
      while (ble_advertise_manufacturer_specific_data(manufacturer_data, MANUFACTURER_DATA_SIZE) != TOCK_SUCCESS) ;
    }

    toggle = !toggle;
    delay_ms(1000);
  }

  return 0;
}

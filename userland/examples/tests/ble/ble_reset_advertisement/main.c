#include <rng.h>
#include <simple_ble.h>
#include <stdbool.h>
#include <stdio.h>
#include <timer.h>
#include <tock.h>

int test_reset_advertisement_buffer(void);

/*******************************************************************************
 * MAIN
 ******************************************************************************/
int main(void) {
  int err;
  printf("[Test] Bluetooth Low Energy Buffer Management\r\n");

  err = test_reset_advertisement_buffer();
  if (err < TOCK_SUCCESS) {
    printf("test_reset_advertisement_buffer failed: %s\r\n", tock_strerror(err));
    return err;
  }
  
  printf("TEST PASSED\r\n");
  return 0;
}

/*******************************************************************************
 * TESTS
 ******************************************************************************/

// Configures `device_name` as TockOS in 5 seconds
// And then changes `device_name` to DynamicOS
int test_reset_advertisement_buffer(void) {
  unsigned char device_name[] = "TockOS";
  unsigned char device_name2[] = "DynamicOS";
 
  int advertising_interval_ms = 20;
  int err = ble_initialize(advertising_interval_ms, true);
  if (err < TOCK_SUCCESS)
    return err;
  
  err = ble_advertise_name(device_name, sizeof(device_name) -1);
  if (err < TOCK_SUCCESS)
    return err;
  
  err = ble_start_advertising();
  if (err < TOCK_SUCCESS)
    return err;

  delay_ms(5000);

  while (ble_reset_advertisement() != TOCK_SUCCESS);

  while (ble_stop_advertising() != TOCK_SUCCESS);

  err = ble_advertise_name(device_name2, sizeof(device_name2) -1);
  if (err < TOCK_SUCCESS)
    return err;

  return ble_start_advertising();
}




#include <rng.h>
#include <simple_ble.h>
#include <stdbool.h>
#include <stdio.h>
#include <timer.h>
#include <tock.h>

int test_off_by_one_name(void);
int test_off_by_one_service_data(void);
int test_exactly_full_buffer(void);
int test_exactly_full_buffer_service_data(void);

/*******************************************************************************
 * MAIN
 ******************************************************************************/
int main(void) {
  int err;
  printf("[Test] Bluetooth Low Energy Buffer Management\r\n");

  err = test_off_by_one_name();
  if (err == TOCK_SUCCESS) {
    printf("test_off_by_one_name failed: %s\r\n", tock_strerror(err));
    return err;
  }
  
  err = test_off_by_one_service_data();
  if (err == TOCK_SUCCESS) {
    printf("test_off_by_one_service_data failed: %s\r\n", tock_strerror(err));
    return err;
  }

  err = test_exactly_full_buffer_service_data();
  if (err == TOCK_SUCCESS) {
    printf("test_exactly_full_buffer_service_data failed: %s\r\n", tock_strerror(err));
    return err;
  }

  err = test_exactly_full_buffer();
  if (err != TOCK_SUCCESS) {
    printf("test_exactly_full_buffer failed: %s\r\n", tock_strerror(err));
    return err;
  }

  printf("TEST PASSED\r\n");
  return 0;
}

/*******************************************************************************
 * TESTS
 ******************************************************************************/

// Name (30 bytes)
// Test internal function `s_configure_adv_data` which appends 2 bytes
// Length (1 byte) || Local Name (1 byte) 
// Total 32 bytes
int test_off_by_one_name(void) {
  unsigned char device_name[] = "TockTockTockTockTockTockTockTo";
 
  int advertising_interval_ms = 20;
  int err = ble_initialize(advertising_interval_ms, true);
  if (err < TOCK_SUCCESS) {
    return TOCK_SUCCESS;
  }
  
  return ble_advertise_name(device_name, sizeof(device_name) -1);
}

// UUID16 || Service Data => 32 bytes 
// Note, this only tests that the wrapper handles buffer management correct
// Internal function `s_configure_adv_data` will fail because it will append 2
// bytes of header
int test_off_by_one_service_data(void) {
  unsigned char data[] = "TockTockTockTockTockTockTockTo";
  
  int advertising_interval_ms = 20;
  int err = ble_initialize(advertising_interval_ms, true);
  if (err < TOCK_SUCCESS) {
    return TOCK_SUCCESS;
  }
  
  return ble_advertise_service_data(0x1801, data, sizeof(data) -1);  
}

// UUID16 || Service Data => 31 bytes
// Note, this only tests that the wrapper handles buffer management correct
// Internal function `s_configure_adv_data` will fail because it will append 2
// bytes of header
int test_exactly_full_buffer_service_data(void) {
  unsigned char data[] = "TockTockTockTockTockTockTockT";

  int advertising_interval_ms = 20;
  int err = ble_initialize(advertising_interval_ms, true);
  if (err < TOCK_SUCCESS) {
    return TOCK_SUCCESS;
  }

  return ble_advertise_service_data(0x1801, data, sizeof(data) -1);
}


// Name (29 bytes)
// Test internal function `s_configure_adv_data` which appends 2 bytes
// Length (1 byte) || Local Name (1 byte) 
// Total 31 bytes
int test_exactly_full_buffer(void) {
  unsigned char device_name[] = "TockTockTockTockTockTockTockT";

  int advertising_interval_ms = 20;
  int err = ble_initialize(advertising_interval_ms, true);
  if (err < TOCK_SUCCESS) {
    return TOCK_SUCCESS;
  }

  return ble_advertise_name(device_name, sizeof(device_name) -1);
}

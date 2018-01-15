#include <rng.h>
#include <simple_ble.h>
#include <stdbool.h>
#include <stdio.h>
#include <timer.h>
#include <tock.h>

#define BUF_SIZE 39

int test_not_initialized(void);
int test_scan_advertise_concurrently(void);

// not used
static void callback(__attribute__((unused)) int a, __attribute__((unused)) int b, __attribute__((unused)) int c,
                     __attribute__((unused)) void *d) {
  return;
}

static unsigned char scan[BUF_SIZE];


/*******************************************************************************
 * MAIN
 ******************************************************************************/
int main(void) {
  int err;
  printf("[Test] BLE States\n");

  err = test_not_initialized();
  if (err < TOCK_SUCCESS) {
    printf("test_not_initialized: %s\r\n", tock_strerror(err));
    return err;
  }

  err = test_scan_advertise();
  if (err < TOCK_SUCCESS) {
    printf("test_scan_advertise: %s\r\n", tock_strerror(err));
    return err;
  }
  printf("TEST PASSED\r\n");
  return 0;
}

/*******************************************************************************
 * TESTS
 ******************************************************************************/
int test_not_initialized(void) {
  int err = ble_start_advertising();
  if (err >= TOCK_SUCCESS) {
    return err;
  }

  err = ble_stop_advertising();
  if (err >= TOCK_SUCCESS) {
    return err;
  }

  err = ble_start_passive_scan(0, 0, 0);
  if (err >= TOCK_SUCCESS) {
    return err;
  }

  err = ble_stop_passive_scan();
  if (err >= TOCK_SUCCESS) {
    return err;
  }

  err = ble_advertise_name(0, 0);
  if (err >= TOCK_SUCCESS) {
    return err;
  }
  return TOCK_SUCCESS;
}

int test_scan_advertise(void) {
  int advertising_interval_ms = 20;
  // valid
  int err = ble_initialize(advertising_interval_ms, true);
  if (err < TOCK_SUCCESS) {
    return err;
  }

  // valid
  err = ble_start_advertising();
  if (err < TOCK_SUCCESS) {
    return err;
  }

  // invalid, the app can't advertise and scan at the same time
  err = ble_start_passive_scan(scan, BUF_SIZE, callback);
  if (err >= TOCK_SUCCESS) {
    return err;
  }

  // valid stop advertise
  err = ble_stop_advertising();
  if (err < TOCK_SUCCESS) {
    return err;
  }

  // valid
  err = ble_start_passive_scan(scan, BUF_SIZE, callback);
  if (err < TOCK_SUCCESS) {
    return err;
  }

  // invalid, the app can't advertise and scan at the same time
  err = ble_start_advertising();
  if (err >= TOCK_SUCCESS) {
    return err;
  }

  // valid stop scanning
  err = ble_stop_passive_scan();
  if (err < TOCK_SUCCESS) {
    return err;
  }

  return TOCK_SUCCESS;
}

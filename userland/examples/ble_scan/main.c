#include <ble.h>
#include <led.h>
#include <stdio.h>
#include <string.h>
#include <timer.h>

/*
 * BLE Demo Application
 * Active scanner for BLE advertisements
 */

#define BUF_SIZE 39
unsigned char scan[BUF_SIZE] = { 0 };

static void callback(__attribute__((unused)) int unused0,
                     __attribute__((unused)) int unused1, __attribute__((unused)) int unused2,
                     __attribute__((unused)) void* ud)
{
  for (int i = 0; i < BUF_SIZE; i++) {
    printf("%02x ", scan[i]);
  }
  printf("\r\n");
}

int main(void)
{
  int err;

  printf("BLE Scanner\r\n");

  // using the pre-configured adv interval
  err = ble_adv_scan(scan, BUF_SIZE, callback);
  if (err < 0) {
    printf("ble_adv_start error %d\r\n", err);
  }

  return 0;
}

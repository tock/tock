#include <stdio.h>
#include <string.h>

#include <alarm.h>
#include <ble.h>

/*
 * BLE Demo Application
 * 1. Configures transmitting power, advertisement interval & advertisement address
 * 2. Configures advertisement data
 * 3. Start advertisment again and runs forever
 */

int main(void)
{
  printf("\r\nBLE ADVERTISEMENT DEMO APP\r\n");
  unsigned char name[] = "TockOS";
  unsigned char data[] = "1337";
  unsigned char addr[] = {0x1, 0x2, 0x3, 0x4, 0x5, 0x6};

  int err;
  err = ble_adv_set_txpower(0);
  if (err < 0) {
    printf("ble_adv_set_txpower error %d\r\n", err);
  }

  err = ble_adv_set_interval(150);
  if ( err < 0) {
    printf("ble_adv_set_interval error %d\r\n", err);
  }

  err = ble_adv_set_address(addr, sizeof(addr));
  if (err < 0) {
    printf("ble_adv_set_address error %d\r\n", err);
  }

  // name and data are strings, remove \0 by subtracting 1
  err = ble_adv_data(BLE_HS_ADV_TYPE_COMP_NAME, sizeof(name) - 1, name);
  if (err < 0) {
    printf("ble_adv_data error %d\r\n", err);
  }

  err = ble_adv_data(BLE_HS_ADV_TYPE_MFG_DATA, sizeof(data) - 1, data);
  if (err < 0) {
    printf("ble_adv_data error %d\r\n", err);
  }

  err = ble_adv_start();
  if (err < 0) {
    printf("ble_adv_start error %d\r\n", err);
  }

  return 0;
}

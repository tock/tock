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
  int err;

  unsigned char name[] = "TockOS";
  unsigned char data[] = "HEJSAHHEJSANNANSANANA";
  unsigned char addr[] = {0x1, 0x2, 0x3, 0x4, 0x5, 0x6};

  BLE_Gap_Mode_t mode  = CONN_NON;
  BLE_TX_Power_t power = ODBM;

  err = ble_adv_set_txpower(power);
  if (err < 0) {
    printf("ble_adv_set_txpower error %d\r\n", err);
  }

  err = ble_adv_set_interval(10);
  if ( err < 0) {
    printf("ble_adv_set_interval error %d\r\n", err);
  }

  err = ble_adv_set_address(addr, sizeof(addr));
  if (err < 0) {
    printf("ble_adv_set_address error %d\r\n", err);
  }

  // name and data are strings, remove \0 by subtracting 1
  err = ble_adv_data(BLE_HS_ADV_TYPE_COMP_NAME, name, sizeof(name) - 1);
  if (err < 0) {
    printf("ble_adv_data error %d\r\n", err);
  }

  err = ble_adv_data(BLE_HS_ADV_TYPE_MFG_DATA, data, sizeof(data) - 1);
  if (err < 0) {
    printf("ble_adv_data error %d\r\n", err);
  }

  err = ble_adv_start(mode);
  if (err < 0) {
    printf("ble_adv_start error %d\r\n", err);
  }

  return 0;
}

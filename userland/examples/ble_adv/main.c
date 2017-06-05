#include <stdio.h>
#include <ble.h>
#include <timer.h>
#include <string.h>

/*
 * BLE Demo Application
 * 1. Configures transmitting power and advertisement interval
 * 2. Configures advertisement data
 * 3. Starts advertisement
 * 4. Stops advertisements
 * 5. Clears advertisment
 * 6. Configures new advertisement data
 * 7. Start advertisment again and runs forever
 */
int main(void)
{
  printf("\r\nBLE ADVERTISEMENT DEMO APP\r\n");
  unsigned char name[] = "TockOS";
  unsigned char data[] = "1337";
  unsigned char tx[] = {0x1};

  int err = ble_adv_set_txpower(0);
  if(err < 0) { printf("ble_adv_set_txpower error %d\r\n", err); }

  err = ble_adv_set_interval(150);
  if( err < 0) { printf("ble_adv_set_interval error %d\r\n", err); }

  // name and data are strings, remove \0 by subtracting 1
  err = ble_adv_data(BLE_HS_ADV_TYPE_COMP_NAME, sizeof(name) - 1, name);
  if(err < 0) { printf("ble_adv_data error %d\r\n", err); }

  err = ble_adv_data(BLE_HS_ADV_TYPE_MFG_DATA, sizeof(data) - 1, data);
  if(err < 0) { printf("ble_adv_data error %d\r\n", err); }

  err = ble_adv_data(BLE_HS_ADV_TYPE_TX_PWR_LVL, 1, tx);
  if(err < 0) { printf("ble_adv_data error %d\r\n", err); }

  err = ble_adv_start();
  if(err < 0) { printf("ble_adv_start error %d\r\n", err); }

  delay_ms(5000);
  err = ble_adv_stop();
  if(err < 0) { printf("ble_adv_start error %d\r\n", err); }

  err = ble_adv_clear_data();
  if(err < 0) { printf("ble_adv_start error %d\r\n", err); }

  delay_ms(5000);
  strcpy((char *)name, "CLEAR");

  err = ble_adv_data(BLE_HS_ADV_TYPE_COMP_NAME, sizeof(name) - 1, name);
  if(err < 0) { printf("ble_adv_data %d\r\n", err);}

  err = ble_adv_start();
  if(err < 0) { printf("ble_adv_start error %d\r\n", err); }

  return 0;
}

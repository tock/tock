#include <stdio.h>
#include <ble.h>
#include <timer.h>
#include <string.h>

int main(void)
{
  printf("\r\ndemo app\n");
  unsigned char name[] = "TockOS";
  unsigned char data[] = "1337";
  unsigned char tx[] = {0x1};

  printf("return for ble_adv_set_power %d\r\n", ble_adv_set_txpower(0));
  printf("return for ble_adv_set_interval %d\r\n", ble_adv_set_interval(150));

  // name and data are strings, remove \0 by subtracting 1
  ble_adv_data(BLE_HS_ADV_TYPE_COMP_NAME, sizeof(name) - 1, name);
  ble_adv_data(BLE_HS_ADV_TYPE_MFG_DATA, sizeof(data) - 1, data);
  ble_adv_data(BLE_HS_ADV_TYPE_TX_PWR_LVL, 1, tx);


  printf("return from start_ble_advertisement %d\r\n", ble_adv_start());
  delay_ms(5000);
  printf("return from stop_ble_advertisement %d\r\n", ble_adv_stop());
  ble_adv_clear_data();
  delay_ms(5000);
  strcpy((char * restrict)name, "CLEAR");
  ble_adv_data(BLE_HS_ADV_TYPE_COMP_NAME, sizeof(name) - 1, name);

  return 0;
}

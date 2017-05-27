#include <stdio.h>
#include "ble.h"
#include <timer.h>

int main(void)
{
  printf("\r\ndemo app\n");
  char name[] = "USERLAND";
  char data[] = "AAAAAAAAAA";
  char test[] = "adada";

  printf("return for ble_adv_set_power %d\r\n", ble_adv_set_txpower(0));
  printf("return for ble_adv_set_interval %d\r\n", ble_adv_set_interval(0));
  printf("return for ble_adv_data %d\r\n", ble_adv_data(BLE_HS_ADV_TYPE_FLAGS, 5, test)); 


  for(;;){
    printf("return from start_ble_advertisement %d\r\n", ble_adv_start(name, data));
    delay_ms(50000);
    printf("return from stop_ble_advertisement %d\r\n", ble_adv_stop());
    delay_ms(5000);
  }
  return 0;
}

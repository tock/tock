#include <stdio.h>
#include "radio_nrf51dk.h"
#include <timer.h>

/** #define RECEIVER */

#define BUF_SIZE 16

#ifdef RECEIVER
static void callback(int type,
		__attribute__ ((unused)) int not_used2,
		__attribute__ ((unused)) int arg2,
		__attribute__ ((unused)) void *ud){

  if (type == 12 ) printf("callback rx\n");
  else if (type == 13) printf("callback tx\n");
}

#endif

int main(void)
{
  printf("demo app\n");
  char packet[] = "u2";
  char data[] = "41";
  start_ble_advertisement(packet, 0, data, 0);
  volatile int i = 0;
  for(;;){
	i = 33;
	delay_ms(10000);
  }
  
  return 0;
}

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
	char packet[] = "From userland";

	#ifdef RECEIVER
  int ret = subscribe_rx(callback, NULL);
  printf("subscribe %d\n", ret);
  for(;;){
    //printf("in receive mode\n");
   rx_data(packet,BUF_SIZE);
   delay_ms(150);
  }
#else
  //int ret = subscribe_tx(callback, NULL);
	//ret = tx_data(packet, BUF_SIZE);
  //int ch = 39;
	for(;;){
		delay_ms(10000);
		printf("after delay \r\n");
		printf("return from start_ble_advertisement %d\r\n",start_ble_advertisement(packet,BUF_SIZE));
		delay_ms(50000);
		//for(int i = 0; i < 1000000; i++){}
		printf("return from stop_ble_advertisement %d\r\n",stop_ble_advertisement());
		printf("after everything\r\n");
	}
  /*for (;;) {

    //set_channel(ch);
    if ( ch < 39 ) {
      ch++;
    }
    else {
      ch = 37;
    	//delay_ms(1000);
    }
    //int send = tx_data(packet, BUF_SIZE);
    //printf("send channel %d\r\n", ch);
  }*/
#endif
  return 0;
}

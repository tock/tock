#include <firestorm.h>
#include <gpio.h>
#include <spi.h>
#include <stdint.h>
#include <timer.h>

#include "rf233-const.h"
#include "rf233-config.h"
#include "rf233-arch.h"
#include "trx_access.h"
#include "rf233.h"

int callback(void*, int); 

int main() { 
  //               FCF   FCF  Seq# Addr1 Addr1  Addr2 Addr2 Pan1  Pan2  Payload
  // char buf[10] = {0x61, 0xAA, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xdd};
  char buf[2] = { 0xde, 0xad }; 
 
  rf233_init(0xab, 0xbc, 0xcd);
  rf233_rx_data(callback);

  /*while (1) {
    rf233_tx_data(0x00, buf, 2);
    delay_ms(10);
    rf233_sleep();
    delay_ms(1000);
    rf233_on();
    delay_ms(10000);
  }*/
  //while(1) {}
}

int callback(void* buffer, int buffer_len) {
	printf("Rx callback!\n"); 
  uint8_t* bytes = (uint8_t*) buffer; 
  for (int i = 0; i < 15; i ++) {
    printf("  Byte %i = %02x\n", i, bytes[i]); 
  }

  return 0; 
}
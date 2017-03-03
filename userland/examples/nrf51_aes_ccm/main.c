#include <stdio.h>
#include "aes.h"
#include <timer.h>

#define SIZE 16

static char data[SIZE+4];

static void callback(int cb, 
    __attribute__ ((unused)) int len,
    __attribute__ ((unused)) int arg2,
    __attribute__ ((unused)) void *ud){

  if ( cb == 0 ) {
    printf("\rKEY IS CONFIGURED\r\n");
  }

  if ( cb == 1 ) 
  {
    printf("CIPHERTEXT + 4 BYTES MIC: \r\n");
    for (int i = 0; i < SIZE+4; i++) {
      printf("%d ", data[i]);
    }
    printf("\r\n");
  }

  if ( cb == 2 ) 
  {
    printf("PLAINTEXT: \r\n");
    for (int i = 0; i < 16; i++) {
      printf("%d ", data[i]);
    }
    printf("\r\n");
  }
}

int main(void)
{
  char key[SIZE];

  for (int i = 0; i < 16; i++) {
    key[i] = i;
  }

  for (int i = 0; i < 20; i++) {
    data[i] = i;
  }

  // SUBSCRIBE
  aes_ccm_init(callback, NULL);
  int config = aes_ccm_configure_key(key, 16);
  if(config < 0) {
    printf("set key error %d\r\n", config);
  }

  for (int i = 0; i < 5; i++) {
    // ALLOW + COMMAND
    delay_ms(500);
    if (aes_ccm_encrypt(data, SIZE) < 0) {
      printf("encrypt error\r\n");
    }
    delay_ms(500);
    if (aes_ccm_decrypt(data, SIZE) < 0) {
      printf("decrypt error\r\n");
    }
  }
  return 0;
}

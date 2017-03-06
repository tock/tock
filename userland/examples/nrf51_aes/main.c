#include <stdio.h>
#include "aes.h"
#include <timer.h>

#define KEY_SIZE  16
#define SIZE      113

static char data[SIZE];

static void callback(int cb, 
    __attribute__ ((unused)) int len,
    __attribute__ ((unused)) int arg2,
    __attribute__ ((unused)) void *ud){

  if ( cb == 0 ) {
    printf("\rKEY IS CONFIGURED\r\n");
  }

  if ( cb == 1 ) 
  {
    printf("CIPHERTEXT \r\n");
    for (int i = 0; i < SIZE; i++) {
      printf("%d ", data[i]);
    }
    printf("\r\n");
  }

  if ( cb == 2 ) 
  {
    printf("PLAINTEXT: \r\n");
    for (int i = 0; i < SIZE; i++) {
      printf("%d ", data[i]);
    }
    printf("\r\n");
  }
}

int main(void)
{
  char key[KEY_SIZE] = {0};

  for (int i = 0; i < SIZE; i++) {
    data[i] = i;
  }


  // SUBSCRIBE
  aes_ecb_init(callback, NULL);
  int config = aes_ecb_configure_key(key, KEY_SIZE);
  if(config < 0) {
    printf("set key error %d\r\n", config);
  }

  for (int i = 0; i < 1; i++) {
    // ALLOW + COMMAND
    delay_ms(500);
    if(aes_ecb_encrypt(data, SIZE) < 0) {
      printf("encrypt error\r\n");
    }
    delay_ms(500);
    if(aes_ecb_decrypt(data, SIZE) < 0) {
      printf("encrypt error\r\n");
    }

  }
  return 0;
}

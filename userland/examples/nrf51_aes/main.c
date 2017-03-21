#include <stdio.h>
#include "aes.h"
#include <timer.h>
#include <string.h>

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
//  char ctr[KEY_SIZE] = {0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd, 0xfe, 0xff};

  for(int i = 0; i < SIZE; i++) {
    data[i] = i+1; 
  }

  // SUBSCRIBE
  aes128_init(callback, NULL);
  int config = aes128_configure_key(key, KEY_SIZE);
  if(config < 0) {
    printf("set key error %d\r\n", config);
  }

  for (int i = 0; i < 1; i++) {
    delay_ms(500);
    if(aes128_encrypt_ctr(data, SIZE) < 0) {
      printf("encrypt error\r\n");
    }
    delay_ms(500);
    if(aes128_decrypt_ctr(data, SIZE) < 0) {
      printf("encrypt error\r\n");
    }

  }
  return 0;
}

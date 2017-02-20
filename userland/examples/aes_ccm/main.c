#include <stdio.h>
#include <aes.h>
#include <timer.h>

#define SIZE 16

static char plaintext[SIZE];

static void callback(int cb, int len,
    __attribute__ ((unused)) int arg2,
    __attribute__ ((unused)) void *ud){

  if ( cb == 0 ) {
    printf("KEY IS CONFIGURED\r\n");
  }

  if ( cb == 1 ) 
  {
    printf("CIPHERTEXT: \r\n");
    for (int i = 0; i < SIZE; i++) {
      printf("%d ", plaintext[i]);
    }
    printf("\r\n");
  }
}

int main(void)
{
  char key[SIZE];

  for (int i = 0; i < SIZE; i++) {
    plaintext[i] = 9;
    key[i] = 1;
  }

  // SUBSCRIBE
  aes_ccm_init(callback, NULL);

  for (int i = 0; i < 1; i++) {
    // ALLOW + COMMAND
    int config = aes_ccm_configure_key(key, SIZE);

    delay_ms(1000);
    int enc = aes_ccm_encrypt(plaintext, SIZE);

    /** int dec = aes_ccm_decrypt(plaintext, SIZE); */
    /** printf("decrypt return %d\n", dec); */
  }
  return 0;
}

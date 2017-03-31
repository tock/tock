#include <stdio.h>
#include <aes.h>
#include <timer.h>
#include <string.h>

/*

   NIST TEST CASE
   ------------------------------------------------------------
   F.5.1       CTR-AES128.Encrypt
   Key            2b7e151628aed2a6abf7158809cf4f3c
   Init. Counter  f0f1f2f3f4f5f6f7f8f9fafbfcfdfeff
   Block #1
   Input Block    f0f1f2f3f4f5f6f7f8f9fafbfcfdfeff
   Output Block   ec8cdf7398607cb0f2d21675ea9ea1e4
   Plaintext      6bc1bee22e409f96e93d7e117393172a
   Ciphertext     874d6191b620e3261bef6864990db6ce
   Block #2
   Input Block    f0f1f2f3f4f5f6f7f8f9fafbfcfdff00
   Output Block   362b7c3c6773516318a077d7fc5073ae
   Plaintext      ae2d8a571e03ac9c9eb76fac45af8e51
   Ciphertext     9806f66b7970fdff8617187bb9fffdff
   Block #3
   Input Block    f0f1f2f3f4f5f6f7f8f9fafbfcfdff01
   Output Block   6a2cc3787889374fbeb4c81b17ba6c44
   Plaintext      30c81c46a35ce411e5fbc1191a0a52ef
   Ciphertext     5ae4df3edbd5d35e5b4f09020db03eab
   Block #4
   Input Block    f0f1f2f3f4f5f6f7f8f9fafbfcfdff02
   Output Block   e89c399ff0f198c6d40a31db156cabfe
   55
   Plaintext      f69f2445df4f9b17ad2b417be66c3710
   Ciphertext     1e031dda2fbe03d1792170a0f3009cee


   F.5.2       CTR-AES128.Decrypt
   Key            2b7e151628aed2a6abf7158809cf4f3c
   Init. Counter  f0f1f2f3f4f5f6f7f8f9fafbfcfdfeff
   Block #1
   Input Block    f0f1f2f3f4f5f6f7f8f9fafbfcfdfeff
   Output Block   ec8cdf7398607cb0f2d21675ea9ea1e4
   Ciphertext     874d6191b620e3261bef6864990db6ce
   Plaintext      6bc1bee22e409f96e93d7e117393172a
   Block #2
   Input Block    f0f1f2f3f4f5f6f7f8f9fafbfcfdff00
   Output Block   362b7c3c6773516318a077d7fc5073ae
   Ciphertext     9806f66b7970fdff8617187bb9fffdff
   Plaintext      ae2d8a71e03ac9c9eb76fac45af8e51
   Block #3
   Input Block    f0f1f2f3f4f5f6f7f8f9fafbfcfdff01
   Output Block   6a2cc3787889374fbeb4c81b17ba6c44
   Ciphertext     5ae4df3edbd5d35e5b4f09020db03eab
   Plaintext      30c81c46a35ce411e5fbc1191a0a52ef
   Block #4
   Input Block    f0f1f2f3f4f5f6f7f8f9fafbfcfdff02
   Output Block   e89c399ff0f198c6d40a31db156cabfe
   Ciphertext     1e031dda2fbe03d1792170a0f3009cee
   Plaintext      f69f2445df4f9b17ad2b417be66c3710
   */

/* DATA BUFFER */
// Block #1: 6bc1bee22e409f96e93d7e117393172a
// Block #2: ae2d8a571e03ac9c9eb76fac45af8e51
// Block #3: 30c81c46a35ce411e5fbc1191a0a52ef
// Block #4  f69f2445df4f9b17ad2b417be66c3710
static unsigned char data[] = {
  0x6b, 0xc1, 0xbe, 0xe2, 0x2e, 0x40, 0x9f, 0x96, 0xe9, 0x3d, 0x7e, 0x11, 0x73, 0x93, 0x17, 0x2a,
  0xae, 0x2d, 0x8a, 0x57, 0x1e, 0x03, 0xac, 0x9c, 0x9e, 0xb7, 0x6f, 0xac, 0x45, 0xaf, 0x8e, 0x51,
  0x30, 0xc8, 0x1c, 0x46, 0xa3, 0x5c, 0xe4, 0x11, 0xe5, 0xfb, 0xc1, 0x19, 0x1a, 0x0a, 0x52, 0xef,
  0xf6, 0x9f, 0x24, 0x45, 0xdf, 0x4f, 0x9b, 0x17, 0xad, 0x2b, 0x41, 0x7b, 0xe6, 0x6c, 0x37, 0x10
};

/* EXPECTED PLAINTEXT */
// Block #1: 6bc1bee22e409f96e93d7e117393172a
// Block #2: ae2d8a571e03ac9c9eb76fac45af8e51
// Block #3: 30c81c46a35ce411e5fbc1191a0a52ef
// Block #4  f69f2445df4f9b17ad2b417be66c3710
static unsigned char exp_pt[] = {
  0x6b, 0xc1, 0xbe, 0xe2, 0x2e, 0x40, 0x9f, 0x96, 0xe9, 0x3d, 0x7e, 0x11, 0x73, 0x93, 0x17, 0x2a,
  0xae, 0x2d, 0x8a, 0x57, 0x1e, 0x03, 0xac, 0x9c, 0x9e, 0xb7, 0x6f, 0xac, 0x45, 0xaf, 0x8e, 0x51,
  0x30, 0xc8, 0x1c, 0x46, 0xa3, 0x5c, 0xe4, 0x11, 0xe5, 0xfb, 0xc1, 0x19, 0x1a, 0x0a, 0x52, 0xef,
  0xf6, 0x9f, 0x24, 0x45, 0xdf, 0x4f, 0x9b, 0x17, 0xad, 0x2b, 0x41, 0x7b, 0xe6, 0x6c, 0x37, 0x10
};

/* EXPECTED CIPHERTEXT */
// Block #1: 874d6191b620e3261bef6864990db6ce
// Block #2: 9806f66b7970fdff8617187bb9fffdff
// Block #3: 5ae4df3edbd5d35e5b4f09020db03eab
// Block #4: 1e031dda2fbe03d1792170a0f3009cee
static unsigned char exp_ct[] = {
  0x87, 0x4d, 0x61, 0x91, 0xb6, 0x20, 0xe3, 0x26, 0x1b, 0xef, 0x68, 0x64, 0x99, 0x0d, 0xb6, 0xce,
  0x98, 0x06, 0xf6, 0x6b, 0x79, 0x70, 0xfd, 0xff, 0x86, 0x17, 0x18, 0x7b, 0xb9, 0xff, 0xfd, 0xff,
  0x5a, 0xe4, 0xdf, 0x3e, 0xdb, 0xd5, 0xd3, 0x5e, 0x5b, 0x4f, 0x09, 0x02, 0x0d, 0xb0, 0x3e, 0xab,
  0x1e, 0x03, 0x1d, 0xda, 0x2f, 0xbe, 0x03, 0xd1, 0x79, 0x21, 0x70, 0xa0, 0xf3, 0x00, 0x9c, 0xee
};

static void callback(int cb,
    __attribute__ ((unused)) int len,
    __attribute__ ((unused)) int arg2,
    __attribute__ ((unused)) void *ud) {

  if ( cb == 0 ) {
    printf("\rKEY IS CONFIGURED\r\n");
  }

  // test according to NIST TEST AES-128-CTR
  if ( cb == 1 )
  {
    // FAIL
    if(memcmp(data, exp_ct, sizeof(data)) != 0) {
      printf("ENCRYPTION TEST FAILED\r\n");
      printf("CIPHERTEXT \r\n");
      for (uint8_t i = 0; i < sizeof(data); i++) {
        printf("%02x ", data[i]);
      }
      printf("\r\n");
      printf("EXPECTED CIPHERTEXT: \r\n");
      for (uint8_t i = 0; i < sizeof(exp_ct); i++) {
        printf("%02x ", exp_ct[i]);
      }
      printf("\r\n");
    }
    // PASS
    else {
      printf("CIPHERTEXT \r\n");
      for (uint8_t i = 0; i < sizeof(data); i++) {
        printf("%02x ", data[i]);
      }
      printf("\r\n");
    }
  }

  if ( cb == 2 )
  {
    // FAIL
    if(memcmp(data, exp_pt, sizeof(data)) != 0) {
      printf("DECRYPT TEST FAILED\r\n");
      printf("PLAINTEXT: \r\n");
      for (uint8_t i = 0; i < sizeof(data); i++) {
        printf("%02x ", data[i]);
      }
      printf("\r\n");
      printf("EXPECTED PLAINTEXT: \r\n");
      for (uint8_t i = 0; i < sizeof(exp_pt); i++) {
        printf("%02x ", exp_pt[i]);
      }
      printf("\r\n");
    }
    // PASS
    else {
      printf("PLAINTEXT: \r\n");
      for (uint8_t i = 0; i < sizeof(data); i++) {
        printf("%02x ", data[i]);
      }
      printf("\r\n");
    }
  }
}

  int main(void)
  {
    /* SET KEY */
    unsigned char key[] = {0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6, 0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf, 0x4f, 0x3c};
    /* INITIAL COUNTER */
    unsigned char ctr[] = {0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd, 0xfe, 0xff};

    // SUBSCRIBE
    aes128_init(callback, NULL);
    int config = aes128_configure_key(key, sizeof(key));
    if(config < 0) {
      printf("set key error %d\r\n", config);
    }

    for (int i = 0; i < 10; i++) {
      delay_ms(500);
      if(aes128_encrypt_ctr(data, sizeof(data), ctr, sizeof(ctr)) < 0) {
        printf("encrypt error\r\n");
      }
      delay_ms(500);
      if(aes128_decrypt_ctr(data, sizeof(data), ctr, sizeof(ctr)) < 0) {
        printf("decrypt error\r\n");
      }
    }
    return 0;
  }

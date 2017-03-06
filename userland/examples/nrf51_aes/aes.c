#include "aes.h"

int aes_ecb_init(subscribe_cb callback, void *ud) {
  return subscribe(DRIVER_ECB, 0, callback, ud);
}

int aes_ecb_configure_key(const char* key, unsigned char len) {
  int err = allow(DRIVER_ECB, KEY, (void*)key, len);
  if (err < 0)  {
    return err;
  }
  return command(DRIVER_ECB, KEY, len);
}

int aes_ecb_encrypt(const char* msg, unsigned char len) {
  int err = allow(DRIVER_ECB, ENC, (void*)msg, len);
  if (err < 0)  {
    return err;
  }
  return command(DRIVER_ECB, ENC, len);
}

int aes_ecb_decrypt(const char* ciphertext, unsigned char len) {
  int err = allow(DRIVER_ECB, DEC, (void*)ciphertext, len);
  if (err < 0)  {
    return err;
  }
  return command(DRIVER_ECB, DEC, len);
}

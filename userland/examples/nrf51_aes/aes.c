#include "aes.h"

int aes128_init(subscribe_cb callback, void *ud) {
  return subscribe(DRIVER_ECB, 0, callback, ud);
}

int aes128_configure_key(const char* key, unsigned char len) {
  int err = allow(DRIVER_ECB, KEY, (void*)key, len);
  if (err < 0)  {
    return err;
  }
  return command(DRIVER_ECB, KEY, 0);
}

int aes128_encrypt_ctr(const char* buf, const char* ctr, unsigned char len) {
  int err = allow(DRIVER_ECB, ENC, (void*)buf, len);
  if (err < 0)  {
    return err;
  }
  err = allow(DRIVER_ECB, CTR, (void*)ctr, 16);
  if (err < 0) {
    return err;
  }
  return command(DRIVER_ECB, ENC, 0);
}

int aes128_decrypt_ctr(const char* buf, const char* ctr, unsigned char len) {
  int err = allow(DRIVER_ECB, DEC, (void*)buf, len);
  if (err < 0)  {
    return err;
  }
  err = allow(DRIVER_ECB, CTR, (void*)ctr, 16);
  if (err < 0) {
    return err;
  }
  return command(DRIVER_ECB, DEC, 0);
}

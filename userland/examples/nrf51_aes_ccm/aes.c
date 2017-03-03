#include "aes.h"

int aes_ccm_init(subscribe_cb callback, void *ud) {
  return subscribe(DRIVER_CCM, 0, callback, ud);
}

int aes_ccm_configure_key(const char* key, unsigned char len) {
  int err = allow(DRIVER_CCM, KEY, (void*)key, len);
  if (err < 0)  {
    return err;
  }
  return command(DRIVER_CCM, KEY, len);
}

int aes_ccm_encrypt(const char* msg, unsigned char len) {
  int err = allow(DRIVER_CCM, ENC, (void*)msg, len+4);
  if (err < 0)  {
    return err;
  }
  return command(DRIVER_CCM, ENC, len);
}

int aes_ccm_decrypt(const char* ciphertext, unsigned char len) {
  int err = allow(DRIVER_CCM, DEC, (void*)ciphertext, len+4);
  if (err < 0)  {
    return err;
  }
  return command(DRIVER_CCM, DEC, len+4);
}

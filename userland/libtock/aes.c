#include "aes.h"

struct aes_data {
  bool fired;
  int type;
};

static struct aes_data result = { .fired = false, .type = -1};


// 0 - set_key
// 1 - encrypt
// 2 - decrypt
static void aes_cb(int cb,
    __attribute__ ((unused)) int len,
    __attribute__ ((unused)) int arg2,
    __attribute__ ((unused)) void *ud) {
  printf("cb %d\r\n", cb);
  result.fired = true;
  result.type = cb;
}


int aes128_set_callback(subscribe_cb callback, void *ud) {
  return subscribe(AES_DRIVER, 0, callback, ud);
}

int aes128_configure_key(const unsigned char* key, unsigned char len) {
  int err = aes128_set_callback(aes_cb, NULL);
  if (err < 0) return err;

  err = allow(AES_DRIVER, AES_KEY, (void*)key, len);
  if (err < 0) return err;
  
  result.fired = false;
  
  err = command(AES_DRIVER, AES_KEY, 0);
  if(err < 0) return err;
  
  yield_for(&result.fired);
  return result.type;
}

int aes128_encrypt_ctr(unsigned const char* buf, unsigned char buf_len, unsigned const char* ctr, unsigned char ctr_len) {
  int err = allow(AES_DRIVER, AES_DATA, (void*)buf, buf_len);
  if (err < 0) return err;

  err = allow(AES_DRIVER, AES_CTR, (void*)ctr, ctr_len);
  if (err < 0) return err;
  
  result.fired = false;
  err = command(AES_DRIVER, AES_ENC, 0);
  yield_for(&result.fired);
  return result.type;
}

int aes128_decrypt_ctr(const unsigned char* buf, unsigned char buf_len, const unsigned char* ctr, unsigned char ctr_len) {
  int err = allow(AES_DRIVER, AES_DATA, (void*)buf, buf_len);
  if (err < 0) return err;

  err = allow(AES_DRIVER, AES_CTR, (void*)ctr, ctr_len);
  if (err < 0) return err;

  result.fired = false;
  err = command(AES_DRIVER, AES_DEC, 0);
  if (err < 0) return err;
  
  yield_for(&result.fired);
  return result.type;
}

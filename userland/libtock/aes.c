#include "aes.h"


// used for creating synchronous versions of functions
//
// fired -  set when the callback has been called
// error - error received from the kernel less than zero indicates an error
typedef struct {
  bool fired;
  int error;
} aes_data_t;


static void aes_cb(int cb,
    __attribute__ ((unused)) int len,
    __attribute__ ((unused)) int arg2,
    __attribute__ ((unused)) void *callback_args) {

  aes_data_t* result = (aes_data_t*)callback_args;
  result->fired = true;
  result->error = cb;
}


int aes128_set_callback(subscribe_cb callback, void *ud) {
  return subscribe(AES_DRIVER, 0, callback, ud);
}


int aes128_set_key_sync(const unsigned char* key, unsigned char len) {
  
  int err;
  aes_data_t result = { .fired = false, .error = SUCCESS };

  err = aes128_set_callback(aes_cb, &result);
  if (err < SUCCESS) return err;

  err = allow(AES_DRIVER, AES_KEY, (void*)key, len);
  if (err < SUCCESS) return err;

  err = command(AES_DRIVER, AES_KEY, 0);
  if (err < SUCCESS) return err;

  yield_for(&result.fired);
  
  return result.error;
}


int aes128_encrypt_ctr_sync(unsigned const char* buf, unsigned char buf_len, 
    unsigned const char* ctr, unsigned char ctr_len) {
  
  int err;
  aes_data_t result = { .fired = false, .error = SUCCESS };

  err = aes128_set_callback(aes_cb, &result);
  if (err < SUCCESS) return err;

  err = allow(AES_DRIVER, AES_DATA, (void*)buf, buf_len);
  if (err < SUCCESS) return err;

  err = allow(AES_DRIVER, AES_CTR, (void*)ctr, ctr_len);
  if (err < SUCCESS) return err;

  err = command(AES_DRIVER, AES_ENC, 0);
  if (err < SUCCESS) return err;

  yield_for(&result.fired);

  return result.error;
}


int aes128_decrypt_ctr_sync(const unsigned char* buf, unsigned char buf_len, 
    const unsigned char* ctr, unsigned char ctr_len) {
  
  int err;
  aes_data_t result = { .fired = false, .error = SUCCESS };

  err = aes128_set_callback(aes_cb, &result);
  if (err < SUCCESS) return err;

  err = allow(AES_DRIVER, AES_DATA, (void*)buf, buf_len);
  if (err < SUCCESS) return err;

  err = allow(AES_DRIVER, AES_CTR, (void*)ctr, ctr_len);
  if (err < SUCCESS) return err;

  err = command(AES_DRIVER, AES_DEC, 0);
  if (err < SUCCESS) return err;

  yield_for(&result.fired);
  
  return result.error;
}

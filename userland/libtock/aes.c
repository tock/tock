#include "aes.h"

// used for creating synchronous versions of functions
//
// fired -  set when the callback has been called
// error - error received from the kernel less than zero indicates an error
typedef struct {
  bool fired;
  int error;
} aes_data_t;


// Internal callback for creating synchronous functions
//
// callback_type - number indicating which type of callback occurred
// currently 1(encryption) and 2(decryption)
// callback_args - user data passed into the set_callback function
//
static void aes_cb(int callback_type,
                   __attribute__ ((unused)) int unused1,
                   __attribute__ ((unused)) int unused2,
                   void *callback_args) {

  aes_data_t *result = (aes_data_t*)callback_args;
  result->fired = true;
  result->error = callback_type;
}


// ***** System Call Interface *****

// Internal callback for encryption and decryption
int aes128_set_callback(subscribe_cb callback, void *ud) {
  return subscribe(AES_DRIVER, 0, callback, ud);
}

// Internal function to configure a payload to encrypt or decrypt
int aes128_set_data(const unsigned char *data, unsigned char len) {
  return allow(AES_DRIVER, AES_DATA, (void*)data, len);
}

// Internal function to configure a initial counter to be used on counter-mode
int aes128_set_ctr(const unsigned char* ctr, unsigned char len) {
  return allow(AES_DRIVER, AES_CTR, (void*)ctr, len);
}

// Internal function to trigger encryption operation. Note that this doesn't
// work by itself aes128_set_data() and aes128_set_ctr() must be called first
int aes128_encrypt_start(void) {
  return command(AES_DRIVER, AES_ENC, 0, 0);
}

// Internal function to trigger encryption operation. Note that this doesn't
// work by itself aes128_set_data() and aes128_set_ctr() must be called first
int aes128_decrypt_start(void) {
  return command(AES_DRIVER, AES_DEC, 0, 0);
}

// Function to encrypt by aes128 counter-mode with a given payload and
// initial counter asynchronously
int aes128_encrypt_ctr(unsigned const char* buf, unsigned char buf_len,
                       unsigned const char* ctr, unsigned char ctr_len, subscribe_cb callback) {

  int err;

  err = aes128_set_callback(callback, NULL);
  if (err < TOCK_SUCCESS) return err;

  err = aes128_set_data(buf, buf_len);
  if (err < TOCK_SUCCESS) return err;

  err = aes128_set_ctr(ctr, ctr_len);
  if (err < TOCK_SUCCESS) return err;

  return aes128_encrypt_start();
}

// Function to decrypt by aes128 counter-mode with a given payload and
// initial counter asynchronously
int aes128_decrypt_ctr(const unsigned char* buf, unsigned char buf_len,
                       const unsigned char* ctr, unsigned char ctr_len, subscribe_cb callback) {

  int err;

  err = aes128_set_callback(callback, NULL);
  if (err < TOCK_SUCCESS) return err;

  err = aes128_set_data(buf, buf_len);
  if (err < TOCK_SUCCESS) return err;

  err = aes128_set_ctr(ctr, ctr_len);
  if (err < TOCK_SUCCESS) return err;

  return aes128_decrypt_start();
}

// ***** Synchronous Calls *****


// Call to configure a buffer with an encryption key in the
// kernel. No need to for a callback for this since it is synchronous in
// the kernel as well.
int aes128_set_key_sync(const unsigned char* key, unsigned char len) {

  int err;

  err = allow(AES_DRIVER, AES_KEY, (void*)key, len);
  if (err < TOCK_SUCCESS) return err;

  return command(AES_DRIVER, AES_KEY, 0, 0);
}


// Function to encrypt by aes128 counter-mode with a given payload and
// initial counter synchronously
int aes128_encrypt_ctr_sync(unsigned const char* buf, unsigned char buf_len,
                            unsigned const char* ctr, unsigned char ctr_len) {

  int err;
  aes_data_t result = { .fired = false, .error = TOCK_SUCCESS };

  err = aes128_set_callback(aes_cb, &result);
  if (err < TOCK_SUCCESS) return err;

  err = aes128_set_data(buf, buf_len);
  if (err < TOCK_SUCCESS) return err;

  err = aes128_set_ctr(ctr, ctr_len);
  if (err < TOCK_SUCCESS) return err;

  err = aes128_encrypt_start();
  if (err < TOCK_SUCCESS) return err;

  yield_for(&result.fired);

  return result.error;
}


// Function to decrypt by aes128 counter-mode with a given payload and
// initial counter synchronously
int aes128_decrypt_ctr_sync(const unsigned char* buf, unsigned char buf_len,
                            const unsigned char* ctr, unsigned char ctr_len) {

  int err;
  aes_data_t result = { .fired = false, .error = TOCK_SUCCESS };

  err = aes128_set_callback(aes_cb, &result);
  if (err < TOCK_SUCCESS) return err;

  err = aes128_set_data(buf, buf_len);
  if (err < TOCK_SUCCESS) return err;

  err = aes128_set_ctr(ctr, ctr_len);
  if (err < TOCK_SUCCESS) return err;

  err = aes128_decrypt_start();
  if (err < TOCK_SUCCESS) return err;

  yield_for(&result.fired);

  return result.error;
}

#pragma once

#include <tock.h>

#ifdef __cplusplus
extern "C" {
#endif

#define AES_DRIVER 17
#define AES_KEY    0
#define AES_DATA   1
#define AES_ENC    2
#define AES_DEC    3
#define AES_CTR    4

// set the function called by the encryption or decryption operations are 
// complete
//
// callback - pointer to function to be called
// callback_args - pointer to data provided to the callback
int aes128_set_callback(subscribe_cb callback, void *callback_args);

// configures an encryption key to be used for encryption and decryption
//
// key - a buffer containing the key (should be 16 bytes for aes128)
// len - length of the buffer (should be 16 bytes for aes128)
int aes128_set_key_sync(const unsigned char* key, unsigned char len);


// encrypts a payload according to aes-128 counter-mode
//
// buf      - buffer to encrypt (currently max 128 bytes are supported)
// buf_len  - length of the buffer to encrypt
// ctr      - buffer with the initial counter
// ctr_len  - length of buffer with the initial counter 
int aes128_encrypt_ctr_sync(const unsigned char* buf, unsigned char buf_len, 
    const unsigned char* ctr, unsigned char ctr_len);


// decrypts a payload according to aes-128 counter-mode
//
// buf      - buffer to decrypt (currently max 128 bytes are supported)
// buf_len  - length of the buffer to decrypt
// ctr      - buffer with the initial counter
// ctr_len  - length of buffer with the initial counter 
int aes128_decrypt_ctr_sync(const unsigned char* buf, unsigned char buf_len, 
    const unsigned char* ctr, unsigned char ctr_len);

#ifdef __cplusplus
}
#endif

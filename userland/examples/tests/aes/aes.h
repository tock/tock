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

int aes128_init(subscribe_cb callback, void *ud);
int aes128_configure_key(const unsigned char* key, unsigned char len);
int aes128_encrypt_ctr(const unsigned char* buf, unsigned char buf_len, const unsigned char* ctr, unsigned char ctr_len);
int aes128_decrypt_ctr(const unsigned char* buf, unsigned char buf_len, const unsigned char* ctr, unsigned char ctr_len);


#pragma once

#ifdef __cplusplus
extern "C" {
#endif

#include <tock.h>

#define DRIVER_ECB 34
#define DRIVER_CCM 35
#define KEY        0
#define ENC        1
#define DEC        2 

int aes128_init(subscribe_cb callback, void *ud);
int aes128_configure_key(const char* key, unsigned char len);
int aes128_encrypt_ctr(const char* buf, unsigned char len);
int aes128_decrypt_ctr(const char* buf, unsigned char len);


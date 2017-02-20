#pragma once

#include <tock.h>

#define DRIVER_ECB 34
#define DRIVER_CCM 35
#define KEY        0
#define ENC        1
#define DEC        2 

#ifdef __cplusplus
extern "C" {
#endif

int aes_ecb_init(subscribe_cb callback, void *ud);
int aes_ecb_configure_key(const char* packet, unsigned char len);
int aes_ecb_encrypt(const char* packet, unsigned char len);
int aes_ecb_decrypt(const char* packet, unsigned char len);


int aes_ccm_init(subscribe_cb callback, void *ud);
int aes_ccm_configure_key(const char* packet, unsigned char len);
int aes_ccm_encrypt(const char* packet, unsigned char len);
int aes_ccm_decrypt(const char* packet, unsigned char len);

#ifdef __cplusplus
}
#endif


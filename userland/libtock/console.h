#pragma once

#include "tock.h"

#ifdef __cplusplus
extern "C" {
#endif

int putstr(const char* str);
int putnstr(const char* str, size_t len);
int putnstr_async(const char* str, size_t len, subscribe_cb cb, void* userdata);

#ifdef __cplusplus
}
#endif

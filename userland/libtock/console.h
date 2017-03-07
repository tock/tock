#pragma once

#include "tock.h"

#ifdef __cplusplus
extern "C" {
#endif

void putstr(const char* str);
void putnstr(const char* str, size_t len);
void putnstr_async(const char* str, size_t len, subscribe_cb cb, void* userdata);

#ifdef __cplusplus
}
#endif

#ifndef CONSOLE_H
#define CONSOLE_H

#include "tock.h"

void putstr(const char* str);
void putnstr(const char* str, size_t len);
void putnstr_async(const char* str, size_t len, subscribe_cb cb, void* userdata);

#endif // CONSOLE_H

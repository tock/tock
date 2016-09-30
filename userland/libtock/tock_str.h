/* tock_str
 *
 * Lightweight string maniuplation and printing for Tock
 *
 */

#ifndef TOCK_STR_H
#define TOCK_STR_H

#include "tock.h"

void putstr(const char* str);
void putnstr(const char* str, size_t len);
void putnstr_async(const char* str, size_t len, subscribe_cb cb, void* userdata);

#endif // TOCK_STR_H

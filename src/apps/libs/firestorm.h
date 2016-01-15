#ifndef _FIRESTORM_H
#define _FIRESTORM_H

#include <unistd.h>
#include "tock.h"

#ifdef __cplusplus
extern "C" {
#endif

enum firestorm_cb_type {
  PUTSTR,
  READTMP,
  ASYNC
};

int gpio_enable(unsigned int pin);
int gpio_set(unsigned int pin);
int gpio_clear(unsigned int pin);

void putstr(const char* str);
void putnstr(const char* str, size_t len);
void putnstr_async(const char* str, size_t len, subscribe_cb cb, void* userdata);

int tmp006_enable();
int tmp006_read(int16_t *temperature);
int tmp006_read_async(subscribe_cb cb, void* userdata);

#ifdef __cplusplus
}
#endif

#endif // _FIRESTORM_H

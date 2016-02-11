#ifndef _FIRESTORM_H
#define _FIRESTORM_H

#include <unistd.h>
#include "tock.h"

// Pin definitions
#define LED_0 0

#ifdef __cplusplus
extern "C" {
#endif

enum firestorm_cb_type {
  PUTSTR,
  READTMP,
  ASYNC,
  SPIBUF
};

int gpio_enable(unsigned int pin);
int gpio_set(unsigned int pin);
int gpio_clear(unsigned int pin);
int gpio_toggle(unsigned int pin);

void putstr(const char* str);
void putnstr(const char* str, size_t len);
void putnstr_async(const char* str, size_t len, subscribe_cb cb, void* userdata);

int timer_oneshot_subscribe(subscribe_cb cb, void *userdata);
int timer_repeating_subscribe(subscribe_cb cb, void *userdata);

#ifdef __cplusplus
}
#endif

#endif // _FIRESTORM_H

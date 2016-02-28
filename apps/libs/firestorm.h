#ifndef _FIRESTORM_H
#define _FIRESTORM_H

#include <unistd.h>
#include "tock.h"
#include "gpio.h"

// Pin definitions
#define LED_0 PC10

#ifdef __cplusplus
extern "C" {
#endif

enum firestorm_cb_type {
  PUTSTR,
  READTMP,
  ASYNC,
  SPIBUF,
  GPIO,
};

void putstr(const char* str);
void putnstr(const char* str, size_t len);
void putnstr_async(const char* str, size_t len, subscribe_cb cb, void* userdata);

int timer_oneshot_subscribe(subscribe_cb cb, void *userdata);
int timer_repeating_subscribe(subscribe_cb cb, void *userdata);


int spi_read_write(const char* write, char* read, size_t  len, subscribe_cb cb);

#ifdef __cplusplus
}
#endif

#endif // _FIRESTORM_H

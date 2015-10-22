#ifndef _FIRESTORM_H
#define _FIRESTORM_H

#include <unistd.h>
#include "tock.h"

enum firestorm_cb_type {
  PUTSTR,
  READTMP,
  ASYNC
};

void putstr(char* str);
void putnstr(char* str, size_t len);
void putnstr_async(char* str, size_t len, subscribe_cb cb, void* userdata);

int tmp006_enable();
int tmp006_read(int16_t *temperature);
int tmp006_read_async(subscribe_cb cb, void* userdata);

#endif

#ifndef _FIRESTORM_H
#define _FIRESTORM_H

#include <unistd.h>
#include "tock.h"

void putstr(char* str);
void putnstr(char* str, size_t len);

int tmp006_enable();
int tmp006_read(int16_t *temperature);
int tmp006_read_async(subscribe_cb cb, void* userdata);

#endif

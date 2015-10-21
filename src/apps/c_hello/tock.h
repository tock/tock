#ifndef _TOCK_H
#define _TOCK_H

#include <inttypes.h>
#include <unistd.h>

typedef int (subscribe_cb)(int, int, int,void*);

int wait();
int command(uint32_t driver, uint32_t command, int data);
int subscribe(uint32_t driver, uint32_t subscribe,
              subscribe_cb cb, void* userdata);
int allow(uint32_t driver, uint32_t allow, void* ptr, size_t size);

#endif

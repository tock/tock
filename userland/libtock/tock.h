#ifndef _TOCK_H
#define _TOCK_H

#include <inttypes.h>
#include <stdbool.h>
#include <unistd.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef void (subscribe_cb)(int, int, int,void*);

void yield();
void yield_for(bool*);
int command(uint32_t driver, uint32_t command, int data);
int subscribe(uint32_t driver, uint32_t subscribe,
              subscribe_cb cb, void* userdata);
int allow(uint32_t driver, uint32_t allow, void* ptr, size_t size);

// op_type can be:
// 0: brk, arg1 is pointer to new memory break
// 1: sbrk, arg1 is increment to increase/decrease memory break
int memop(uint32_t op_type, int arg1);

#ifdef __cplusplus
}
#endif

#endif // _TOCK_H

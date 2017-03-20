#pragma once

#include <stdbool.h>
#include <stdint.h>
#include <unistd.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef void (subscribe_cb)(int, int, int,void*);

void yield(void);
void yield_for(bool*);
int command(uint32_t driver, uint32_t command, int data);
int subscribe(uint32_t driver, uint32_t subscribe,
              subscribe_cb cb, void* userdata);
int allow(uint32_t driver, uint32_t allow, void* ptr, size_t size);

// op_type can be:
// 0: brk, arg1 is pointer to new memory break
// 1: sbrk, arg1 is increment to increase/decrease memory break
void* memop(uint32_t op_type, int arg1);

// Checks to see if the given driver number exists on this platform.
bool driver_exists(uint32_t driver);

#define SUCCESS   0
#define FAIL     -1
#define EBUSY    -2
#define EALREADY -3
#define EOFF     -4
#define ERESERVE -5
#define EINVAL   -6
#define ESIZE    -7
#define ECANCEL  -8
#define ENOMEM   -9
#define ENOSUPPORT -10
#define ENODEVICE  -11
#define EUNINSTALLED -12
#define ENOACK -13

#ifdef __cplusplus
}
#endif

#pragma once

#include <stdbool.h>
#include <stdint.h>
#include <unistd.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef void (subscribe_cb)(int, int, int,void*);

int tock_enqueue(subscribe_cb cb, int arg0, int arg1, int arg2, void* ud);

void yield(void);
void yield_for(bool*);

__attribute__ ((warn_unused_result))
int command(uint32_t driver, uint32_t command, int data, int arg2);

__attribute__ ((warn_unused_result))
int subscribe(uint32_t driver, uint32_t subscribe,
              subscribe_cb cb, void* userdata);

__attribute__ ((warn_unused_result))
int allow(uint32_t driver, uint32_t allow, void* ptr, size_t size);

// op_type can be:
// 0: brk, arg1 is pointer to new memory break
// 1: sbrk, arg1 is increment to increase/decrease memory break
void* memop(uint32_t op_type, int arg1);

// Wrappers around memop to support app introspection
void* tock_app_memory_begins_at(void);
void* tock_app_memory_ends_at(void);
void* tock_app_flash_begins_at(void);
void* tock_app_flash_ends_at(void);
void* tock_app_grant_begins_at(void);
int tock_app_number_writeable_flash_regions(void);
void* tock_app_writeable_flash_region_begins_at(int region_index);
void* tock_app_writeable_flash_region_ends_at(int region_index);


// Checks to see if the given driver number exists on this platform.
bool driver_exists(uint32_t driver);

#define TOCK_SUCCESS       0
#define TOCK_FAIL         -1
#define TOCK_EBUSY        -2
#define TOCK_EALREADY     -3
#define TOCK_EOFF         -4
#define TOCK_ERESERVE     -5
#define TOCK_EINVAL       -6
#define TOCK_ESIZE        -7
#define TOCK_ECANCEL      -8
#define TOCK_ENOMEM       -9
#define TOCK_ENOSUPPORT   -10
#define TOCK_ENODEVICE    -11
#define TOCK_EUNINSTALLED -12
#define TOCK_ENOACK       -13

const char* tock_strerror(int tock_errno);

#ifdef __cplusplus
}
#endif

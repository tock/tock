#ifndef _TOCK_H
#define _TOCK_H

#include <inttypes.h>
#include <unistd.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef int CB_TYPE;

typedef CB_TYPE (subscribe_cb)(int, int, int,void*);

CB_TYPE wait();
CB_TYPE wait_for();
int command(uint32_t driver, uint32_t command, int arg1, int arg2);
int subscribe(uint32_t driver, uint32_t subscribe,
              subscribe_cb cb, void* userdata);
int allow(uint32_t driver, uint32_t allow, void* ptr, size_t size);

#ifdef __cplusplus
}
#endif

#endif // _TOCK_H

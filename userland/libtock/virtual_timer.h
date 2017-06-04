#pragma once

#ifdef __cplusplus
extern "C" {
#endif

#include <tock.h>

typedef struct virtual_timer virtual_timer_t;

virtual_timer_t *virtual_timer_start(int ms, subscribe_cb, void*);

void virtual_timer_cancel(virtual_timer_t*);

#ifdef __cplusplus
}
#endif

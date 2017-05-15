#pragma once

#include "tock.h"

#ifdef __cplusplus
extern "C" {
#endif

#define DRIVER_NUM_SYSTEM 254

unsigned system_tock_major_version(void);

void* system_app_memory_begins_at(void);
void* system_app_memory_ends_at(void);
void* system_app_flash_begins_at(void);
void* system_app_flash_ends_at(void);
void* system_app_grant_begins_at(void);

#ifdef __cplusplus
}
#endif


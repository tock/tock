#include "tock.h"
#include "system.h"

unsigned system_tock_major_version(void) {
	return command(DRIVER_NUM_SYSTEM, 0, 0);
}

// Allow casting a function that returns `int` to `void*`
#pragma GCC diagnostic ignored "-Wbad-function-cast"

void* system_app_memory_begins_at(void) {
	return (void*) command(DRIVER_NUM_SYSTEM, 1, 0);
}

void* system_app_memory_ends_at(void) {
	return (void*) command(DRIVER_NUM_SYSTEM, 2, 0);
}

void* system_app_flash_begins_at(void) {
	return (void*) command(DRIVER_NUM_SYSTEM, 3, 0);
}

void* system_app_flash_ends_at(void) {
	return (void*) command(DRIVER_NUM_SYSTEM, 4, 0);
}

void* system_app_grant_begins_at(void) {
	return (void*) command(DRIVER_NUM_SYSTEM, 5, 0);
}

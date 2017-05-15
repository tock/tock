#include "tock.h"
#include "system.h"

void* system_app_memory_begins_at(void) {
	return memop(2, 0);
}

void* system_app_memory_ends_at(void) {
	return memop(3, 0);
}

void* system_app_flash_begins_at(void) {
	return memop(4, 0);
}

void* system_app_flash_ends_at(void) {
	return memop(5, 0);
}

void* system_app_grant_begins_at(void) {
	return memop(6, 0);
}

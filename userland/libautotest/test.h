#pragma once

#include <stdbool.h>

typedef bool (*test_fun)(void);

void test_runner(test_fun *tests, uint32_t test_count, void *test_buf, uint32_t timeout_ms, const char *svc_name);
void test_service(void);

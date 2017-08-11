#include <test.h>
#include <timer.h>
#include <tock.h>

#include <stdbool.h>

uint32_t test_buf[128] __attribute__((aligned(128)));

static bool test_pass(void) {
    return true;
}

static bool test_fail(void) {
    return false;
}


static bool test_timeout(void) {
    while (1) { yield(); }
    return true;
}

int main(void) {
    test_fun tests[6] = { test_pass, test_pass, test_pass, test_fail, test_fail, test_pass };
    test_runner(tests, 6, &test_buf[0], 300, "org.tockos.autotest");
    return 0;
}

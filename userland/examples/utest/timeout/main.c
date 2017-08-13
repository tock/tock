#include <utest.h>
#include <timer.h>
#include <tock.h>

#include <stdbool.h>

static bool test_pass(void) {
    delay_ms(100);
    return true;
}

static bool test_fail(void) {
    delay_ms(100);
    return false;
}


static bool test_timeout(void) {
    delay_ms(500);
    return true;
}

int main(void) {
    utest_fun tests[3] = { test_pass, test_fail, test_timeout };
    utest_runner(tests, 3, 300, "org.tockos.utest");
    return 0;
}

#include <utest.h>
#include <tock.h>
#include <stdio.h>
#include <timer.h>

#include <stdbool.h>

static bool test_pass(void) {
    return true;
}

static bool test_fail(void) {
    return false;
}


int main(void) {
    delay_ms(10000);
    utest_fun tests[6] = { test_pass, test_pass, test_pass, test_fail, test_fail, test_pass };
    utest_runner(tests, 6, 300, "org.tockos.utest");
    return 0;
}

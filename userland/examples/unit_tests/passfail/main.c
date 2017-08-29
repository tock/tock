#include <stdio.h>
#include <timer.h>
#include <tock.h>
#include <unit_test.h>

#include <stdbool.h>

static bool test_pass(void) {
  return true;
}

static bool test_fail(void) {
  return false;
}


int main(void) {
  unit_test_fun tests[6] = { test_pass, test_pass, test_pass, test_fail, test_fail, test_pass };
  unit_test_runner(tests, 6, 300, "org.tockos.unit_test");
  return 0;
}

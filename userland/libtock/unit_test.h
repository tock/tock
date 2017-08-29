/** @file unit_test.h
 * @brief Basic framework for running userland unit tests.
 *
 * This library provides a simple and easy infrastructure for writing and
 * running unit tests for Tock userland applications.
 *
 * To run tests, there are two applications needed. One is the 'test
 * supervisor', whose function is to facilitate the unit tests and report their
 * status (pass/fail/timeout). The other application is the test application,
 * which implements the specific unit test that is to be run. The test supervisor
 * provides an IPC service to which the test applications subscribe.
 *
 * Note that there is only one test supervisor needed, you write it once and
 * forget it. Just make sure it's loaded onto the board with your test runner
 * when it's time to run the tests!
 *
 * To write a test runner, simply write each test as a function which returns
 * true or false depending on whether the test passes. Then pass an array of the
 * test functions to `unit_test_runner`. That's it!
 *
 * ## Example
 *
 * --> unit_test_supervisor/main.c: (with PACKAGE_NAME "org.tockos.unit_test")
 *
 *    #include <unit_test.h>
 *
 *    int main(void) {
 *      unit_test_service();
 *      return 0;
 *    }
 *
 *
 * --> mytest/main.c:
 *
 *    #include <unit_test.h>
 *    #include <tock.h>
 *    #include <stdbool.h>
 *
 *    static bool test_pass(void) {
 *      return true;
 *    }
 *
 *    static bool test_fail(void) {
 *      return false;
 *    }
 *
 *    static bool test_timeout(void) {
 *      while (1) { yield(); }
 *      return true;
 *    }
 *
 *    int main(void) {
 *      unit_test_fun tests[3] = { test_pass, test_fail, test_timeout };
 *      uint32_t test_timeout_ms = 300;
 *
 *      unit_test_runner(tests, 3, test_timeout_ms, "org.tockos.unit_test");
 *
 *      return 0;
 *    }
 *
 *
 * In this case, if you load both applications on the board, the serial output
 * will be:
 *
 *    1.0: [✓]
 *    1.1: [FAILED]
 *    1.2: [ERROR: Timeout]
 *    Summary 1: [1/3] Passed, [1/3] Failed, [1/3] Incomplete
 *
 * Author: Shane Leonard <shanel@stanford.edu>
 * Modified: 8/13/2017
 */

#pragma once

#ifdef __cplusplus
extern "C" {
#endif

#include <stdbool.h>

/** \brief Unit test function signature.
 *
 * All unit tests should return a boolean representing true for PASS and false
 * for FAIL.
 */
typedef bool (*unit_test_fun)(void);

/** \brief Unit test runner.
 *
 * \param tests An array of boolean functions which return true for PASS and
 *              false for FAIL.
 * \param test_count The total number of tests in the tests array.
 * \param timeout_ms The maximum amount of time each test is allowed to run
 *                   before being timed out.
 * \param svc_name The IPC service name of the test supervisor (e.g.
 *                 "org.tockos.unit_test")
 *
 * The test runner should pass in an array of boolean functions representing the
 * individual tests.
 * Example:
 *
 *    #include <unit_test.h>
 *    #include <tock.h>
 *    #include <stdbool.h>
 *
 *    static bool test_pass(void) {
 *      return true;
 *    }
 *
 *    static bool test_fail(void) {
 *      return false;
 *    }
 *
 *    static bool test_timeout(void) {
 *      while (1) { yield(); }
 *      return true;
 *    }
 *
 *    int main(void) {
 *      unit_test_fun tests[3] = { test_pass, test_fail, test_timeout };
 *      uint32_t test_timeout_ms = 300;
 *
 *      unit_test_runner(tests, 3, test_timeout_ms, "org.tockos.unit_test");
 *
 *      return 0;
 *    }
 */
void unit_test_runner(unit_test_fun *tests, uint32_t test_count,
                  uint32_t timeout_ms, const char *svc_name);

/** \brief Test supervisor entry point.
 *
 * The test supervisor should call this function in main and then return.
 * Example:
 *
 *    #include <unit_test.h>
 *
 *    int main(void) {
 *      unit_test_service();
 *      return 0;
 *    }
 */
void unit_test_service(void);

#ifdef __cplusplus
}
#endif

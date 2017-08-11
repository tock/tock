#include <string.h>
#include <stdio.h>

#include <test.h>
#include <ipc.h>
#include <timer.h>

typedef enum test_state {
    NotStarted,
    Testing,
    Complete,
    ErrorTimeout,
    ErrorExtraNotify,
} test_state_t;

typedef struct test_header {
    uint32_t count;
    uint32_t current;
    uint32_t timeout_ms;
    uint32_t pass_count;
    uint32_t fail_count;
    int pid;
    tock_timer_t timer;
} test_header_t;

typedef struct test_status {
    test_state_t state;
    bool passed;
} test_status_t;

//
// CLIENT FUNCTIONS
//

static void sigkill_cb(__attribute__ ((unused)) int pid,
                       __attribute__ ((unused)) int arg2,
                       __attribute__ ((unused)) int arg3,
                       __attribute__ ((unused)) void *ud) {
    printf("Recieved SIGKILL from test service; exiting...\n");
    exit(1);
}

void test_runner(test_fun *tests, uint32_t test_count, void *test_buf, uint32_t timeout_ms, const char *svc_name) {
    delay_ms(30);

    uint32_t test_buf_sz = sizeof(test_header_t) + (sizeof(test_status_t) * test_count);
    if (test_buf_sz > 128) return;

    memset(test_buf, 0, test_buf_sz);

    test_header_t *header = (test_header_t *)(test_buf);
    test_status_t *statuses = (test_status_t *)(header + 1);

    // Initialize header. PID and timer fields are initialized by the service.
    header->count = test_count;
    header->timeout_ms = timeout_ms;

    int test_svc = ipc_discover(svc_name);
    if (test_svc < 0) return;

    ipc_register_client_cb(test_svc, sigkill_cb, NULL);
    ipc_share(test_svc, test_buf, 128);

    uint32_t i = 0;
    for (i = 0; i < test_count; i++) {
        test_status_t *status = &statuses[i];

        ipc_notify_svc(test_svc);

        status->passed = tests[i]();

        ipc_notify_svc(test_svc);
        delay_ms(30);

        header->current++;
    }
}

//
// SERVICE FUNCTIONS
//

static void print_test_result(test_status_t *test, uint32_t test_num, uint32_t pid) {
    if (test->state == Complete) {
        if (test->passed) {
            printf("%lu.%lu: [PASSED]\n", pid, test_num);
        } else {
            printf("%lu.%lu: [FAILED]\n", pid, test_num);
        }
    } else if (test->state == ErrorTimeout) {
        printf("%lu.%lu: [ERROR: Timeout]\n", pid, test_num);
    } else if (test->state == ErrorExtraNotify) {
        printf("%lu.%lu: [ERROR: Extra Notify]\n", pid, test_num);
    } else {
        printf("%lu.%lu: [ERROR: Test status incorrect]\n", pid, test_num);
    }
}

static void timeout_cb(__attribute__ ((unused)) int now,
                       __attribute__ ((unused)) int expiration,
                       __attribute__ ((unused)) int unused, void* ud) {

    test_header_t *header = (test_header_t *)ud;
    test_status_t *tests = (test_status_t *)(header + 1);
    test_status_t *test = &tests[header->current];

    test->state = ErrorTimeout;
    print_test_result(test, header->current, header->pid);
    ipc_notify_client(header->pid);
}

static void test_service_cb(int pid, __attribute__ ((unused)) int len, int buf,
                           __attribute__ ((unused)) void *ud) {
    // TODO: bounds checking
    if (buf == 0) {
        printf("Null buffer encountered.\n");
        return;
    }

    test_header_t *header = (test_header_t *)buf;
    test_status_t *tests = (test_status_t *)(header + 1);

    header->pid = pid;
    test_status_t *test = &tests[header->current];

    switch (test->state) {
        case NotStarted:
            // The test has found the service successfully and is signalling
            // that the test is starting
            test->state = Testing;
            timer_in(header->timeout_ms, timeout_cb, header, &header->timer);
            break;
        case Testing:
            // Test terminated and is signalling completion
            timer_cancel(&header->timer);
            test->state = Complete;
            print_test_result(test, header->current, pid);
            break;
        case Complete:
            test->state = ErrorExtraNotify;
            print_test_result(test, header->current, pid);
            break;
        default:
            break;
    }
}

void test_service(void) {
    ipc_register_svc(test_service_cb, NULL);
}

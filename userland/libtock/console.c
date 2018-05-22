#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include "console.h"

typedef struct putstr_data {
  char* buf;
  int len;
  bool called;
  struct putstr_data* next;
} putstr_data_t;

static putstr_data_t *putstr_head = NULL;
static putstr_data_t *putstr_tail = NULL;

static void putstr_cb(int _x __attribute__ ((unused)),
                      int _y __attribute__ ((unused)),
                      int _z __attribute__ ((unused)),
                      void* ud __attribute__ ((unused))) {
  putstr_data_t* data = putstr_head;
  data->called = true;
  putstr_head  = data->next;

  if (putstr_head == NULL) {
    putstr_tail = NULL;
  } else {
    int ret;
    ret = putnstr_async(putstr_head->buf, putstr_head->len, putstr_cb, NULL);
    if (ret < 0) {
      // XXX There's no path to report errors currently, so just drop it
      putstr_cb(0, 0, 0, NULL);
    }
  }
}

int putnstr(const char *str, size_t len) {
  int ret = TOCK_SUCCESS;

  putstr_data_t* data = (putstr_data_t*)malloc(sizeof(putstr_data_t));
  if (data == NULL) return TOCK_ENOMEM;

  data->len    = len;
  data->called = false;
  data->buf    = (char*)malloc(len * sizeof(char));
  if (data->buf == NULL) {
    ret = TOCK_ENOMEM;
    goto putnstr_fail_buf_alloc;
  }
  strncpy(data->buf, str, len);
  data->next = NULL;

  if (putstr_tail == NULL) {
    // Invariant, if tail is NULL, head is also NULL
    ret = putnstr_async(data->buf, data->len, putstr_cb, NULL);
    if (ret < 0) goto putnstr_fail_async;
    putstr_head = data;
    putstr_tail = data;
  } else {
    putstr_tail->next = data;
    putstr_tail       = data;
  }

  yield_for(&data->called);

putnstr_fail_async:
  free(data->buf);
putnstr_fail_buf_alloc:
  free(data);

  return ret;
}

int putnstr_async(const char *str, size_t len, subscribe_cb cb, void* userdata) {
  int ret;
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wcast-qual"
  // Currently, allow gives RW access, but we should have a richer set of
  // options, such as kernel RO, which would be let us preserve type semantics
  // all the way down
  void* buf = (void*) str;
#pragma GCC diagnostic pop

  ret = allow(DRIVER_NUM_CONSOLE, 1, buf, len);
  if (ret < 0) return ret;

  ret = subscribe(DRIVER_NUM_CONSOLE, 1, cb, userdata);
  if (ret < 0) return ret;

  ret = command(DRIVER_NUM_CONSOLE, 1, len, 0);
  return ret;
}

int getnstr_async(char *str, size_t len, subscribe_cb cb, void* userdata) {
  int ret;

  ret = allow(DRIVER_NUM_CONSOLE, 2, str, len);
  if (ret < 0) return ret;

  ret = subscribe(DRIVER_NUM_CONSOLE, 2, cb, userdata);
  if (ret < 0) return ret;

  ret = command(DRIVER_NUM_CONSOLE, 2, len, 0);
  return ret;
}

typedef struct getnstr_data {
  bool called;
  int result;
} getnstr_data_t;

static getnstr_data_t getnstr_data = { true, 0 };

static void getnstr_cb(int result,
                       int _y __attribute__ ((unused)),
                       int _z __attribute__ ((unused)),
                       void* ud __attribute__ ((unused))) {
  getnstr_data.result = result;
  getnstr_data.called = true;
}

int getnstr(char *str, size_t len) {
  int ret;

  if (!getnstr_data.called) {
    // A call is already in progress
    return TOCK_EALREADY;
  }
  getnstr_data.called = false;

  ret = getnstr_async(str, len, getnstr_cb, NULL);
  if (ret < 0) return ret;

  yield_for(&getnstr_data.called);

  return getnstr_data.result;
}

int getch(void) {
  int r;
  char buf[1];

  r = getnstr(buf, 1);

  return (r == TOCK_SUCCESS) ? buf[0] : TOCK_FAIL;
}

int getnstr_abort(void) {
  return command(DRIVER_NUM_CONSOLE, 3, 0, 0);
}

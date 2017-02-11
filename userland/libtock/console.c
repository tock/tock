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

static void putstr_cb(
                int _x __attribute__ ((unused)),
                int _y __attribute__ ((unused)),
                int _z __attribute__ ((unused)),
                void* ud __attribute__ ((unused))) {
  putstr_data_t* data = putstr_head;
  data->called = true;
  putstr_head = data->next;

  if (putstr_head == NULL) {
    putstr_tail = NULL;
  } else {
    putnstr_async(putstr_head->buf, putstr_head->len, putstr_cb, NULL);
  }
}

void putnstr(const char *str, size_t len) {
  putstr_data_t* data = (putstr_data_t*)malloc(sizeof(putstr_data_t));

  data->len = len;
  data->called = false;
  data->buf = (char*)malloc(len * sizeof(char));
  strncpy(data->buf, str, len);
  data->next = NULL;

  if (putstr_tail == NULL) {
    // Invariant, if tail is NULL, head is also NULL
    putstr_head = data;
    putstr_tail = data;
    putnstr_async(data->buf, data->len, putstr_cb, NULL);
  } else {
    putstr_tail->next = data;
    putstr_tail = data;
  }

  yield_for(&data->called);

  free(data->buf);
  free(data);
}

void putnstr_async(const char *str, size_t len, subscribe_cb cb, void* userdata) {
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wcast-qual"
  // Currently, allow gives RW access, but we should have a richer set of
  // options, such as kernel RO, which would be let us preserve type semantics
  // all the way down
  void* buf = (void*) str;
#pragma GCC diagnostic pop
  allow(0, 1, buf, len);
  subscribe(0, 1, cb, userdata);
}

void putstr(const char *str) {
  putnstr(str, strlen(str));
}

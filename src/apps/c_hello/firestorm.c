#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include "firestorm.h"
#include "tock.h"

static int putstr_cb(int _x, int _y, int _z, void* str) {
  free(str);
  return 0;
}


void putnstr(char *str, size_t len) {
  char* buf = (char*)malloc(len * sizeof(char));
  strncpy(buf, str, len);
  allow(0, 1, buf, len);
  subscribe(0, 1, putstr_cb, buf);
  wait();
}

void putstr(char *str) {
  putnstr(str, strlen(str));
}

static int read_tmp006_cb(int r0, int r1, int r2, void* ud) {
  int16_t *res = (int16_t*)ud;
  *res = (int16_t)r0 / 32;

  return 0;
}

void enable_tmp006() {
  command(2, 0, 0);
}

int tmp006_read(int16_t *temperature) {
  int error = tmp006_read_async(read_tmp006_cb, (void*)temperature);
  if (error < 0) {
    return error;
  }
  wait();
  return 0;
}

int tmp006_read_async(subscribe_cb cb, void* userdata) {
  return subscribe(2, 0, cb, userdata);
}

int tmp006_enable() {
  command(2, 0, 0);
}


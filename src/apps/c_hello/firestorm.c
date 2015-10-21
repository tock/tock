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

int16_t read_tmp006() {
  command(2, 0, 0); // enable tmp006

  int16_t result;
  subscribe(2, 0, read_tmp006_cb, &result);
  wait();
  return result;
}


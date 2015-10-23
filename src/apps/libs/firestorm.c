#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <firestorm.h>
#include <tock.h>

static CB_TYPE putstr_cb(int _x, int _y, int _z, void* str) {
  free(str);
  return PUTSTR;
}

void putnstr(char *str, size_t len) {
  char* buf = (char*)malloc(len * sizeof(char));
  strncpy(buf, str, len);
  putnstr_async(buf, len, putstr_cb, buf);
  wait_for(PUTSTR);
}

void putnstr_async(char *str, size_t len, subscribe_cb cb, void* userdata) {
  allow(0, 1, str, len);
  subscribe(0, 1, cb, userdata);
}

void putstr(char *str) {
  putnstr(str, strlen(str));
}

void enable_tmp006() {
  command(2, 0, 0);
}

static CB_TYPE read_tmp006_cb(int r0, int r1, int r2, void* ud) {
  int16_t *res = (int16_t*)ud;
  *res = (int16_t)r0 / 32;

  return READTMP;
}


int tmp006_read(int16_t *temperature) {
  int error = tmp006_read_async(read_tmp006_cb, (void*)temperature);
  if (error < 0) {
    return error;
  }
  wait_for(READTMP);
  return 0;
}

int tmp006_read_async(subscribe_cb cb, void* userdata) {
  return subscribe(2, 0, cb, userdata);
}

int tmp006_enable() {
  command(2, 0, 0);
}


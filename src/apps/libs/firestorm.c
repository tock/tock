#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <firestorm.h>
#include <tock.h>

int gpio_enable(unsigned int pin) {
  return command(1, 0, pin);
}

int gpio_set(unsigned int pin) {
  return command(1, 2, pin);
}

int gpio_clear(unsigned int pin) {
  return command(1, 3, pin);
}

int gpio_toggle(unsigned int pin) {
  return command(1, 4, pin);
}

static CB_TYPE putstr_cb(int _x, int _y, int _z, void* str) {
  free(str);
  return PUTSTR;
}

void putnstr(const char *str, size_t len) {
  char* buf = (char*)malloc(len * sizeof(char));
  strncpy(buf, str, len);
  putnstr_async(buf, len, putstr_cb, buf);
  wait_for(PUTSTR);
}

void putnstr_async(const char *str, size_t len, subscribe_cb cb, void* userdata) {
  allow(0, 1, (void*)str, len);
  subscribe(0, 1, cb, userdata);
}

void putstr(const char *str) {
  putnstr(str, strlen(str));
}

int timer_oneshot_subscribe(subscribe_cb cb, void *userdata) {
  return subscribe(3, 0, cb, userdata);
}

int timer_repeating_subscribe(subscribe_cb cb, void *userdata) {
  return subscribe(3, 1, cb, userdata);
}


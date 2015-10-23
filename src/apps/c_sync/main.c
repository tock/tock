/* vim: set sw=2 expandtab tw=80: */

#include <string.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

#include <tock.h>
#include <firestorm.h>

void main() {
  int err;
  int16_t temperature;

  putstr("Welcome to Tock in C (with libc)\r\n\
Reading temperature... ");

  err = tmp006_enable();
  if (err < 0) {
    char buf[64];
    snprintf(buf, 64, "Error(%d): Failed to enable TMP006.\r\n", err);
    putstr(buf);
    return;
  }

  err = tmp006_read(&temperature);
  if (err < 0) {
    char buf[64];
    snprintf(buf, 64, "Error(%d): TMP006 not enabled.\r\n", err);
    putstr(buf);
  } else {
    char* str = malloc(128);
    sprintf(str, "%d\u00B0C \r\n", temperature);
    putstr(str);
  }
}


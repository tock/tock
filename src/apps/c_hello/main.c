/* vim: set sw=2 expandtab tw=80: */

#include <string.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

#include "tock.h"


int write_done(int _x, int _y, int _z, void* str) {
  free(str);
  return 0;
}

int tmp_available(int r0, int r1, int r2, void* ud) {
  int16_t tmp = (int16_t)r0;
  char* str = malloc(128);
  sprintf(str, "%d\u2103 \r\n", tmp / 32);

  allow(0, 1, str, strlen(str));
  return subscribe(0, 1, write_done, str);
}

void main() {
  command(1, 0, 0); // enable pin 0
  command(1, 2, 0); // set pin 0

  command(2, 0, 0); // enable tmp

  char hello[] = "Welcome to Tock in C (with libc)\r\n\
Reading temperature... ";

  char* str = malloc(sizeof(hello));
  strncpy(str, hello, sizeof(hello));

  allow(0, 1, str, strlen(hello));
  subscribe(0, 1, &write_done, str);
  wait();

  subscribe(2, 0, tmp_available, 0);

  while(1) wait();
}


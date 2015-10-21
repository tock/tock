/* vim: set sw=2 expandtab tw=80: */

#include <string.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

extern void __wait();
extern int __command();
extern int __allow();
extern int __subscribe();

void _putc(char c) {
  __command(0, 0, c);
}

void putstr(char* str) {
  char c = *str;
  while (c != '\0') {
    _putc(c);
    ++str;
    c = *str;
  }
}

void write_done(int _x, int _y, int _z, char *str) {
  free(str);
}

void noop() {}

void tmp_available(int16_t tmp) {
  char* str = malloc(128);
  sprintf(str, "%d\u2103 \r\n", tmp / 32);

  __allow(0, 1, str, strlen(str));
  __subscribe(0, 1, write_done, 0);
}

void main() {
  __command(1, 0, 0); // enable pin 0
  __command(1, 2, 0); // set pin 0

  __command(2, 0, 0); // enable tmp

  char hello[] = "Welcome to Tock in C (with libc)\r\n\
Reading temperature... ";

  char* str = malloc(sizeof(hello));
  strncpy(str, hello, sizeof(hello));

  __allow(0, 1, str, strlen(hello));
  __subscribe(0, 1, &write_done, str);
  __wait();

  __subscribe(2, 0, tmp_available);

  while(1) __wait();
}


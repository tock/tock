/* vim: set sw=2 expandtab tw=80: */

#include <string.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

#include "tock.h"
#include "firestorm.h"

void main() {
  putstr("Welcome to Tock in C (with libc)\r\n\
Reading temperature... ");

  int16_t temp = read_tmp006();
  char* str = malloc(128);
  sprintf(str, "%d\u2103 \r\n", temp);
  putstr(str);

  while(1) wait();
}


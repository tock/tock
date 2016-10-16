/* vim: set sw=2 expandtab tw=80: */

#include <string.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

#include <console.h>

char hello[] = "Hello World!\r\n";

void nop() {}

int main() {
  putnstr_async(hello, sizeof(hello), nop, NULL);
  return 0;
}


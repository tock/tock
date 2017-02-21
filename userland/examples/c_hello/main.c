/* vim: set sw=2 expandtab tw=80: */

#include <string.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

#include <console.h>

char hello[] = "Hello World!\r\n";

static void nop(
    int a __attribute__((unused)),
    int b __attribute__((unused)),
    int c __attribute__((unused)),
    void* d __attribute__((unused))) {
}

int main(void) {
  putnstr_async(hello, sizeof(hello), nop, NULL);
  return 0;
}

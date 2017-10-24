#if 0
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#endif

#include <console.h>

char hello[] = "Hello World!\r\n";

static void print_complete(int a __attribute__((unused)),
                           int b __attribute__((unused)),
                           int c __attribute__((unused)),
                           void* d __attribute__((unused)))
{
  // The message has been printed to the console
}

int main(void)
{
  putnstr_async(hello, sizeof(hello), print_complete, NULL);
  return 0;
}

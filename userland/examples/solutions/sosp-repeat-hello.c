#include <console.h>
#include <timer.h>

char hello[] = "Hello World!\r\n";

static void print_complete(int a __attribute__((unused)),
                           int b __attribute__((unused)),
                           int c __attribute__((unused)),
                           void* d __attribute__((unused)))
{
  // The message has been printed to the console

  delay_ms(2000);
  putnstr_async(hello, sizeof(hello), print_complete, NULL);
}

int main(void)
{
  putnstr_async(hello, sizeof(hello), print_complete, NULL);
  return 0;
}

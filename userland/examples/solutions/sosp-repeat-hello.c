#include <stdio.h>
#include <timer.h>

int main(void)
{
  while (1) {
    printf("Hello, World!\n");
    delay_ms(2000);
  }
}

#include <stdbool.h>
#include <stdio.h>

#include <timer.h>

int main (void) {
  while (true) {
    printf("Hello, World!\n");
    delay_ms(500);
  }
}


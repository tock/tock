/* vim: set sw=2 expandtab tw=80: */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include <console.h>

//#define BUFSIZE 27*1024 works normally
//#define BUFSIZE 45*1024 hardfaults
#define BUFSIZE 38*1024
static uint8_t bigbuf[BUFSIZE];

int main(void) {
  printf("Staring bigapp\n");

  printf("Writing buffer...\n");
  for (int i=0; i<BUFSIZE; i++) {
    bigbuf[i] = (uint8_t)i;

    if (i%1024 == 0) {
      printf("At %d kB\n", i/1024);
    }
  }

  printf("Checking buffer...\n");
  for (int i=0; i<BUFSIZE; i++) {
    if (bigbuf[i] != (uint8_t)i) {
      printf("ERROR: bigbuf[%d] equals %u not %u\n", i, bigbuf[i], (uint8_t)i);
    }

    if (i%1024 == 0) {
      printf("At %d kB\n", i/1024);
    }
  }

  printf("Complete!\n");
}


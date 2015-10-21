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

char hello[] = "Hello World!\r\n";

void write_done(int _x, int _y, int _z, char *str) {
  putstr(str);
  free(str);
  __command(1, 2, 0); // set pin 0
}


void main() {
  __command(1, 0, 0); // enable pin 0

  char* str = malloc(sizeof(hello));
  //siprintf(str, "%s (0x%x) (0x%x)\r\n", hello, res, hello);
  strncpy(str, hello, sizeof(hello));

  __allow(0, 1, str, strlen(hello));
  __subscribe(0, 1, &write_done, str);
  __wait();


  while(1) __wait();
}


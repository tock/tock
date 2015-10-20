#include <string.h>
#include <stdlib.h>
#include <unistd.h>

extern void __wait();
extern int __command();
extern int __allow();
extern int __subscribe();

caddr_t _sbrk(int incr)
{
  return 0;
}

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

void write_done(char* str) {
  putstr(str);
  //free(str);
}


void main() {
  __command(1, 0, 0); // enable pin 0

  //char* str = (char*)malloc(sizeof(hello));
  //strncpy(str, hello, sizeof(hello));
  //str[0] = 'H';
  __allow(0, 1, hello, strlen(hello));
  __subscribe(0, 1, write_done, hello);
  __wait();

  __command(1, 2, 0); // set pin 0

  while(1) __wait();
}


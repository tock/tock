#include <stdlib.h>
#include <unistd.h>
#include <tock.h>

extern unsigned int* _etext;
extern unsigned int* _edata;
extern unsigned int* _got;
extern unsigned int* _egot;
extern unsigned int* _bss;
extern unsigned int* _ebss;

void main();

void _start();

void* heap_base;

caddr_t _sbrk(int incr)
{
  heap_base += incr;
  return heap_base;
}

__attribute__ ((section(".start"), used, naked))
void _start(void* heap_start) {
  void main();

  heap_base = heap_start;

  main();

  while(1) { wait(); }
}


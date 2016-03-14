#include <errno.h>
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

#ifndef STACK_SIZE
#define STACK_SIZE 1024
#endif

__attribute__ ((section(".start"), used, naked))
void _start(void* mem_start, __attribute__((unused))void* mem_end) {
  void main();

  /* Setup the stack and heap.
   * We setup the stack at the bottom of memory (directory after the GOT, data
   * and BSS segments. The stack size is, therefore, fixed at load time, so
   * adjust it appropriately using `-DSTACK_SIZE 1024` during compilation.
   *
   * The heap will begin directly above the stack, and grow upwards towards
   * kernel borrowed heap (which grows downwards from the top of memory).
   */
  void* heap_start;
  int stack_size = STACK_SIZE;

  asm volatile ("add %0, %1, %2\n\t" // Stack start = `mem_start` + `stack_size`
                "mov sp, %0\n\t"     //
                "add %0, %0, #4\n\t" // Heap starts 4 bytes above stack
                : "=r" (heap_start)
                : "r" (mem_start), "r" (stack_size));

  heap_base = heap_start;

  main();

  while(1) { wait(); }
}


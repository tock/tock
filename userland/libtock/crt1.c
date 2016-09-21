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

caddr_t _sbrk(int incr)
{
  return (void*)memop(1, incr);
}

int brk(void* memory_break) {
  return memop(0, (int)memory_break);
}

#ifndef STACK_SIZE
#define STACK_SIZE 2048
#endif

__attribute__ ((section(".start"), used, naked))
void _start(void* mem_start,
    __attribute__((unused))void* app_memory_break,
    __attribute__((unused))void* kernel_memory_break) {
  void main();

  /* Setup the stack and heap.
   * We setup the stack at the bottom of memory (directory after the GOT, data
   * and BSS segments. The stack size is, therefore, fixed at load time, so
   * adjust it appropriately using `-DSTACK_SIZE 1024` during compilation.
   *
   * The heap will begin directly above the stack, and grow upwards towards
   * kernel borrowed heap (which grows downwards from the top of memory).
   */
  void* stack_bottom = mem_start + STACK_SIZE;
  brk(stack_bottom);

  __asm volatile ("mov sp, %0\n\t" : : "r" (stack_bottom));

  main();

  while(1) { yield(); }
}


/* vim: set sw=2 expandtab tw=80: */

#include <string.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

#include <console.h>


static uint32_t read_cpsr(void) {
  register uint32_t ret asm ("r0");
  asm volatile(
      "mrs r0, CONTROL"
      : "=r"(ret)
      :
      :
      );
  return ret;
}

/*
static void clear_priv(void) {
  asm volatile(
      "mov r0, #1\n\tmsr CONTROL, r0"
      :
      :
      : "r0"
      );
}
*/

__attribute__((noinline))
static void dowork(uint32_t from, uint32_t to, uint32_t incr) {
  volatile uint8_t* p_from = (uint8_t*) from;
  volatile uint8_t* p_to = (uint8_t*) to;

  printf("%p -> %p, incr 0x%lx\n", p_from, p_to, incr);
  printf("       CPSR: %08lx\n", read_cpsr());

  while (p_from < p_to) {
    printf("%p: ", p_from);
    fflush(stdout);
    printf("%08x\n", *p_from);
    p_from += incr;
    asm("nop;");
  }
}

// Try not to move the stack much so main's reading the sp reg is meaningful
__attribute__((noinline))
static void start(
    void* mem_start,
    void* app_heap_break,
    void* kernel_memory_break,
    void* sp) {
  printf("\n[TEST] MPU Walk Regions\n");
  putchar('\n');

  printf("  mem_start:           %p\n", mem_start);
  printf("  app_heap_break:      %p\n", app_heap_break);
  printf("  kernel_memory_break: %p\n", kernel_memory_break);
  printf("  stack pointer (ish): %p\n", sp);

  putchar('\n');

  dowork(0x20004000, (uint32_t)kernel_memory_break & 0xfffffe00, 0x100);
  dowork(0x20000000, 0x20004000, 0x100);
}

// override default _start symbol to access memory regions
//
// Note: parameters passed from the kernel to _start are considered unstable and
// subject to change in the future
#pragma GCC diagnostic ignored "-Wmissing-prototypes"
__attribute__ ((section(".start"), used))
__attribute__ ((noreturn))
void _start(
    void* mem_start,
    void* app_heap_break,
    void* kernel_memory_break) {
  register uint32_t* sp asm ("sp");
  start(mem_start, app_heap_break, kernel_memory_break, sp);
  while(1) { yield(); }
}

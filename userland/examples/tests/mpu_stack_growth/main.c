/* vim: set sw=2 expandtab tw=80: */

#include <string.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

#include <console.h>

#define GROW_BY 0x100

uint32_t size_is_at_least = 0;

__attribute__((noinline))
static void write_ptr(uint32_t* p) {
  printf(" write to %p\n", p);
  *p = 33;
}

__attribute__((noinline))
static void read_ptr(uint32_t* p) {
  printf("read from %p\n", p);
  printf("    value %lu\n", *p);
}

static void grow_stack(void) {
  register uint32_t* sp asm ("sp");

  uint32_t buffer[GROW_BY];
  printf("stack: %p - buffer: %p - STACK_SIZE: 0x%x - at_least: 0x%4lx\n",
      sp, buffer, STACK_SIZE, size_is_at_least);

  write_ptr(buffer);
  read_ptr(buffer);

  size_is_at_least += GROW_BY;

  if (size_is_at_least > STACK_SIZE) {
    printf("This should never print\n");
  }

  grow_stack();
}

// Try not to move the stack much so main's reading the sp reg is meaningful
__attribute__((noinline))
static void start(
    void* mem_start,
    void* app_heap_break,
    void* kernel_memory_break,
    void* sp) {
  printf("\n[TEST] MPU Stack Growth\n");

  printf("This test should recursively add stack frames until exceeding\n");
  printf("the available stack space and triggering a fault\n\n");

  printf("  mem_start:           %p\n", mem_start);
  printf("  app_heap_break:      %p\n", app_heap_break);
  printf("  kernel_memory_break: %p\n", kernel_memory_break);
  printf("  stack pointer (ish): %p\n", sp);

  putchar('\n');

  grow_stack();
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

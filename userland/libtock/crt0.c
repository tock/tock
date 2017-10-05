#include <tock.h>

extern int main(void);

// Allow _start to go undeclared
#pragma GCC diagnostic ignored "-Wmissing-declarations"
#pragma GCC diagnostic ignored "-Wmissing-prototypes"

__attribute__ ((section(".start"), used))
__attribute__ ((weak))
__attribute__ ((noreturn))
__attribute__ ((naked))
void _start(void* text_start __attribute__((unused)),
            void* mem_start __attribute__((unused)),
            void* memory_len __attribute__((unused)),
            void* app_heap_break __attribute__((unused))) {
  /*
   * 1. Setup r9 to point to the GOT (assumes the kernel places the GOT right
   *    above the stack)
   * 2. Call `main`
   * 3. Loop on `yield` forever
   */
  asm volatile ("mov r9, sp; \
                 bl main; \
                 1: bl yield; \
                 b 1b");
}


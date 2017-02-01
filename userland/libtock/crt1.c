#include <tock.h>

extern unsigned int* _etext;
extern unsigned int* _edata;
extern unsigned int* _got;
extern unsigned int* _egot;
extern unsigned int* _bss;
extern unsigned int* _ebss;
extern int main();

/* Define stack and heap "arrays"
 *
 * By putting arrays into dedicated sections, the linker script can put in
 * appropriately sized gaps in memory, and this size information will be
 * preserved as available symbols in the ELF, which can then get packed into
 * the load_info struct.
 *
 * By doing this via #defines, an application that needs a larger stack/heap
 * can simply -D define their needed size and the rest of the toolchain will
 * pick up the change.
 */

#ifndef STACK_SIZE
#error STACK_SIZE not defined.\
       libtock expects STACK_SIZE to be defined by the compiling environment\
       and for the compilation to check and warn for oversized stacks, i.e.\
         $(CC) -DSTACK_SIZE=$(SIZE) -fstack-usage -Wstack-usage=$(SIZE)
#endif

__attribute__ ((section(".stack")))
unsigned char _dont_use_stack[STACK_SIZE];

#ifndef APP_HEAP_SIZE
#define APP_HEAP_SIZE 1024
#endif

__attribute__ ((section(".app_heap")))
unsigned char _dont_use_app_heap[APP_HEAP_SIZE];

#ifndef KERNEL_HEAP_SIZE
#define KERNEL_HEAP_SIZE 1024
#endif

__attribute__ ((section(".kernel_heap")))
unsigned char _dont_use_kernel_heap[KERNEL_HEAP_SIZE];


__attribute__ ((section(".start"), used))
void _start(
    __attribute__((unused))void* mem_start,
    __attribute__((unused))void* app_memory_break,
    __attribute__((unused))void* kernel_memory_break) {
  main();
  while(1) { yield(); }
}


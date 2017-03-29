#include <tock.h>

extern unsigned int* _etext;
extern unsigned int* _edata;
extern unsigned int* _got;
extern unsigned int* _egot;
extern unsigned int* _bss;
extern unsigned int* _ebss;
extern int main(void);

// Allow _start to go undeclared
#pragma GCC diagnostic ignored "-Wmissing-prototypes"

__attribute__ ((section(".start"), used))
__attribute__ ((weak))
__attribute__ ((noreturn))
void _start(
    void* mem_start __attribute__((unused)),
    void* app_heap_break __attribute__((unused)),
    void* kernel_memory_break __attribute__((unused))) {
  main();
  while(1) { yield(); }
}


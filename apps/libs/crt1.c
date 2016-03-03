#include <stdlib.h>
#include <unistd.h>
#include <tock.h>

extern unsigned int* _etext;
extern unsigned int* _edata;
extern unsigned int* _reldata;
extern unsigned int* _ereldata;
extern unsigned int* _got;
extern unsigned int* _egot;
extern unsigned int* _bss;
extern unsigned int* _ebss;

void main();

void _start();

typedef struct {
    unsigned int* entry_loc;        /* Entry point for user application */
    unsigned int* init_data_loc;    /* Data initialization information in flash */
    unsigned int init_data_size;    /* Size of initialization information */
    unsigned int got_start_offset;  /* Offset to start of GOT */
    unsigned int got_end_offset;    /* Offset to end of GOT */
    unsigned int bss_start_offset;  /* Offset to start of BSS */
    unsigned int bss_end_offset;    /* Offset to end of BSS */
    unsigned int rel_start_offset;  /* Offset to start of relocate data */
    unsigned int rel_end_offset;    /* Offset to start of relocate data */
} Load_Info;

// Load Info is used by the runtime in order to load the application
//  Note that locations in the text section assume .text starts at 0x0
//  and are therefore updated
__attribute__ ((section(".load_info"), used))
Load_Info app_info = {
    .entry_loc          = (unsigned int*)((unsigned int)_start - 0x80000000),
    .init_data_loc      = (unsigned int*)((unsigned int)(&_etext) - 0x80000000),
    .init_data_size     = (unsigned int)(&_edata),
    .got_start_offset   = (unsigned int)(&_got),
    .got_end_offset     = (unsigned int)(&_egot),
    .bss_start_offset   = (unsigned int)(&_bss),
    .bss_end_offset     = (unsigned int)(&_ebss),
    .rel_start_offset   = (unsigned int)(&_reldata),
    .rel_end_offset     = (unsigned int)(&_ereldata),
};

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


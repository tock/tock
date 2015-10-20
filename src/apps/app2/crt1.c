#include <stdlib.h>
#include <unistd.h>

extern unsigned int* _etext;
extern unsigned int* _edata;
extern unsigned int* _got;
extern unsigned int* _egot;
extern unsigned int* _bss;
extern unsigned int* _ebss;

void _start();

typedef struct {
    unsigned int* entry_loc;        /* Entry point for user application */
    unsigned int* init_data_loc;    /* Data initialization information in flash */
    unsigned int init_data_size;    /* Size of initialization information */
    unsigned int got_start_offset;  /* Offset to start of GOT */
    unsigned int got_end_offset;    /* Offset to end of GOT */
    unsigned int bss_start_offset;  /* Offset to start of BSS */
    unsigned int bss_end_offset;    /* Offset to end of BSS */
} Load_Info;

// Load Info is used by the runtime in order to load the application
//  Note that locations in the text section assume .text starts at 0x10000000
//  and are therefore updated
__attribute__ ((section(".load_info"), used))
Load_Info app_info = {
    .entry_loc          = (unsigned int*)((unsigned int)_start - 0x10000000),
    .init_data_loc      = (unsigned int*)((unsigned int)(&_etext) - 0x10000000),
    .init_data_size     = (unsigned int)(&_edata),
    .got_start_offset   = (unsigned int)(&_got),
    .got_end_offset     = (unsigned int)(&_egot),
    .bss_start_offset   = (unsigned int)(&_bss),
    .bss_end_offset     = (unsigned int)(&_ebss),
};

void* heap_base;

__attribute__ ((section(".start"), used))
void _start(void* heap_start) {
  void main();

  heap_base = heap_start;

  main();
}


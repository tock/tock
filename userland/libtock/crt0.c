#include <tock.h>

extern int main(void);

// Allow _start to go undeclared
#pragma GCC diagnostic ignored "-Wmissing-declarations"
#pragma GCC diagnostic ignored "-Wmissing-prototypes"

struct hdr {
  uint32_t got_sym_start;
  uint32_t got_start;
  int got_size;
  uint32_t bss_start;
  int bss_size;
};


__attribute__ ((section(".start"), used))
__attribute__ ((weak))
__attribute__ ((noreturn))
void _start(void* text_start __attribute__((unused)),
            void* mem_start __attribute__((unused)),
            void* memory_len __attribute__((unused)),
            void* app_heap_break __attribute__((unused))) {

  // Allocate stack. `brk` to 1024 from start of memory
  {
    int stacktop = (int)mem_start + 1024;
    memop(0, stacktop + 1024);
    asm volatile ("mov sp, %[stacktop]" :: [stacktop] "r" (stacktop));
    asm volatile ("mov r9, sp");
  }


  struct hdr* myhdr = (struct hdr*)text_start;
  int stacktop = (int)mem_start + 1024;

  // fix up GOT
  volatile uint32_t* got_start = (uint32_t*)(myhdr->got_start + stacktop);
  volatile uint32_t* got_sym_start = (uint32_t*)(myhdr->got_sym_start + (uint32_t)text_start);
  for (int i = 0; i < (myhdr->got_size / (int)sizeof(uint32_t)); i++) {
    if ((got_sym_start[i] & 0x80000000) == 0) {
      got_start[i] = got_sym_start[i] + stacktop + myhdr->got_size;
    } else {
      got_start[i] = (got_sym_start[i] ^ 0x80000000) + (uint32_t)text_start;
    }
  }

  // zero BSS
  volatile uint32_t* bss_start = (uint32_t*)(myhdr->bss_start + stacktop);
  for (int i = 0; i < (myhdr->bss_size / (int)sizeof(uint32_t)); i++) {
    bss_start[i] = 0;
  }

  // TODO: copy data section

  main();
  while(1) {
    yield();
  }
}


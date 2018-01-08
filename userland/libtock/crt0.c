#include <tock.h>

extern int main(void);

// Allow _start to go undeclared
#pragma GCC diagnostic ignored "-Wmissing-declarations"
#pragma GCC diagnostic ignored "-Wmissing-prototypes"

struct hdr {
  uint32_t got_sym_start;
  uint32_t got_start;
  int got_size;
  uint32_t data_sym_start;
  uint32_t data_start;
  int data_size;
  uint32_t bss_start;
  int bss_size;
  uint32_t reldata_start;
};

struct reldata {
  int len;
  int data[];
};

__attribute__ ((section(".start"), used))
__attribute__ ((weak))
__attribute__ ((noreturn))
void _start(void* text_start,
            void* mem_start,
            void* memory_len __attribute__((unused)),
            void* app_heap_break __attribute__((unused))) {

  // Allocate stack and data. `brk` to STACK_SIZE + got_size + data_size +
  // bss_size from start of memory
  uint32_t stacktop = (uint32_t)mem_start + STACK_SIZE;
  struct hdr* myhdr = (struct hdr*)text_start;

  {
    uint32_t heap_size = myhdr->got_size + myhdr->data_size + myhdr->bss_size;
    memop(0, stacktop + heap_size);
    asm volatile ("mov sp, %[stacktop]" :: [stacktop] "r" (stacktop) : "memory");
    asm volatile ("mov r9, sp");
  }

  // fix up GOT
  volatile uint32_t* got_start     = (uint32_t*)(myhdr->got_start + stacktop);
  volatile uint32_t* got_sym_start = (uint32_t*)(myhdr->got_sym_start + (uint32_t)text_start);
  for (int i = 0; i < (myhdr->got_size / (int)sizeof(uint32_t)); i++) {
    if ((got_sym_start[i] & 0x80000000) == 0) {
      got_start[i] = got_sym_start[i] + stacktop;
    } else {
      got_start[i] = (got_sym_start[i] ^ 0x80000000) + (uint32_t)text_start;
    }
  }

  // load data section
  void* data_start     = (void*)(myhdr->data_start + stacktop);
  void* data_sym_start = (void*)(myhdr->data_sym_start + (uint32_t)text_start);
  memcpy(data_start, data_sym_start, myhdr->data_size);

  // zero BSS
  char* bss_start = (char*)(myhdr->bss_start + stacktop);
  memset(bss_start, 0, myhdr->bss_size);

  struct reldata* rd = (struct reldata*)(myhdr->reldata_start + (uint32_t)text_start);
  int i;
  for (i = 0; i < (rd->len / (int)sizeof(uint32_t)); i += 2) {
    uint32_t* target = (uint32_t*)(rd->data[i] + stacktop);
    if ((*target & 0x80000000) == 0) {
      *target += stacktop;
    } else {
      *target = (*target ^ 0x80000000) + (uint32_t)text_start;
    }
  }

  main();
  while (1) {
    yield();
  }
}


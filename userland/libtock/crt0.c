#include <tock.h>

#if defined(STACK_SIZE)
#warning Attempt to compile libtock with a fixed STACK_SIZE.
#warning
#warning Instead, STACK_SIZE should be a variable that is linked in,
#warning usually at compile time via something like this:
#warning   `gcc ... -Xlinker --defsym=STACK_SIZE=2048`
#warning
#warning This allows applications to set their own STACK_SIZE.
#error Fixed STACK_SIZE.
#endif

extern int main(void);

// Allow _start to go undeclared
#pragma GCC diagnostic ignored "-Wmissing-declarations"
#pragma GCC diagnostic ignored "-Wmissing-prototypes"

/*
 * The structure populated by the linker script at the very beginning of the
 * text segment. It represents sizes and offsets from the text segment of
 * sections that need some sort of loading and/or relocation.
 */
struct hdr {
  // Offset of GOT symbols in flash
  uint32_t got_sym_start;
  // Offset of GOT section in memory
  uint32_t got_start;
  // Size of GOT section
  uint32_t got_size;
  // Offset of data symbols in flash
  uint32_t data_sym_start;
  // Offset of data section in memory
  uint32_t data_start;
  // Size of data section
  uint32_t data_size;
  // Offset of BSS section in memory
  uint32_t bss_start;
  // Size of BSS section
  uint32_t bss_size;
  // First address offset after program flash, where elf2tab places
  // .rel.data section
  uint32_t reldata_start;
  // The size of the stack requested by this application
  uint32_t stack_size;
};

struct reldata {
  uint32_t len;
  uint32_t data[];
};

__attribute__ ((section(".start"), used))
__attribute__ ((weak))
__attribute__ ((noreturn))
void _start(void* app_start,
            void* mem_start,
            void* memory_len __attribute__((unused)),
            void* app_heap_break __attribute__((unused))) {

  // Allocate stack and data. `brk` to stack_size + got_size + data_size +
  // bss_size from start of memory. Also make sure that the stack starts on an
  // 8 byte boundary per section 5.2.1.2 here:
  // http://infocenter.arm.com/help/topic/com.arm.doc.ihi0042f/IHI0042F_aapcs.pdf
  struct hdr* myhdr = (struct hdr*)app_start;
  uint32_t stacktop = (((uint32_t)mem_start + myhdr->stack_size + 7) & 0xfffffff8);

  // fix up GOT
  volatile uint32_t* got_start     = (uint32_t*)(myhdr->got_start + stacktop);
  volatile uint32_t* got_sym_start = (uint32_t*)(myhdr->got_sym_start + (uint32_t)app_start);
  for (uint32_t i = 0; i < (myhdr->got_size / (uint32_t)sizeof(uint32_t)); i++) {
    if ((got_sym_start[i] & 0x80000000) == 0) {
      got_start[i] = got_sym_start[i] + stacktop;
    } else {
      got_start[i] = (got_sym_start[i] ^ 0x80000000) + (uint32_t)app_start;
    }
  }

  // load data section
  void* data_start     = (void*)(myhdr->data_start + stacktop);
  void* data_sym_start = (void*)(myhdr->data_sym_start + (uint32_t)app_start);
  memcpy(data_start, data_sym_start, myhdr->data_size);

  // zero BSS
  char* bss_start = (char*)(myhdr->bss_start + stacktop);
  memset(bss_start, 0, myhdr->bss_size);

  struct reldata* rd = (struct reldata*)(myhdr->reldata_start + (uint32_t)app_start);
  for (uint32_t i = 0; i < (rd->len / (int)sizeof(uint32_t)); i += 2) {
    uint32_t* target = (uint32_t*)(rd->data[i] + stacktop);
    if ((*target & 0x80000000) == 0) {
      *target += stacktop;
    } else {
      *target = (*target ^ 0x80000000) + (uint32_t)app_start;
    }
  }

  {
    uint32_t heap_size = myhdr->got_size + myhdr->data_size + myhdr->bss_size;
    memop(0, stacktop + heap_size);
    memop(11, stacktop + heap_size);
    memop(10, stacktop);
    asm volatile ("mov sp, %[stacktop]" :: [stacktop] "r" (stacktop) : "memory");
    asm volatile ("mov r9, sp");
  }

  main();
  while (1) {
    yield();
  }
}


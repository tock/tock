MEMORY
{
  rom (rx)  : ORIGIN = 0x00010000, LENGTH = 256K
  prog (rx) : ORIGIN = 0x00050000, LENGTH = 704K
  ram (rwx) : ORIGIN = 0x20000000, LENGTH = 256K
}

MPU_MIN_ALIGN = 8K;
PAGE_SIZE = 4K;

INCLUDE ../kernel_layout.ld

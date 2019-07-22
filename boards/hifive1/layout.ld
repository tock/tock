MEMORY
{
  rom (rx)  : ORIGIN = 0x20400000, LENGTH = 1M
  prog (rx) : ORIGIN = 0x20500000, LENGTH = 1M
  ram (rwx) : ORIGIN = 0x80000000, LENGTH = 64K
}

MPU_MIN_ALIGN = 8K;

INCLUDE ../kernel_layout.ld

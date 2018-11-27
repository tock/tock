MEMORY
{
  rom (rx)  : ORIGIN = 0x40400000, LENGTH = 512M
  prog (rx) : ORIGIN = 0x60300000, LENGTH = 1M
  ram (rwx) : ORIGIN = 0x80000000, LENGTH = 64K
}

MPU_MIN_ALIGN = 8K;

INCLUDE ../kernel_layout.ld

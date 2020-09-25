/* The entire memory is actually placed into DRAM by the bootloader */

MEMORY
{
  rom (rx)  : ORIGIN = 0x40000000, LENGTH = 0x1000000
  prog (rx) : ORIGIN = 0x41000000, LENGTH = 0x1000000
  ram (rwx) : ORIGIN = 0x42000000, LENGTH = 0x50000000 - 0x42000000
}

MPU_MIN_ALIGN = 1K;

INCLUDE ../kernel_layout.ld

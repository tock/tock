MEMORY
{
  rom (rx)  : ORIGIN = 0x40400000, LENGTH = 0x00030000
  prog (rx) : ORIGIN = 0x40430000, LENGTH = 1M
  ram (rwx) : ORIGIN = 0x80000000, LENGTH = 64K
}

INCLUDE ../kernel_layout.ld

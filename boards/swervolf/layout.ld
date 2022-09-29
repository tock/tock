MEMORY
{
  rom (rx)  : ORIGIN = 0x00000000, LENGTH = 0x30000
  prog (rx) : ORIGIN = 0x30000,    LENGTH = 0x30000
  ram (rwx) : ORIGIN = 0x60000,    LENGTH = 0x30000
}

INCLUDE ../kernel_layout.ld

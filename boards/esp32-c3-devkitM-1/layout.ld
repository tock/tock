MEMORY
{
  rom (rx)  : ORIGIN = 0x40380000, LENGTH = 0x30000
  ram (rwx) : ORIGIN = 0x403B0000, LENGTH = 0x30000
  prog (rx) : ORIGIN = 0x42000000, LENGTH = 0x80000
}

MPU_MIN_ALIGN = 1K;

INCLUDE ../kernel_layout.ld

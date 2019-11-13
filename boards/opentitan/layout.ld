MEMORY
{
  rom   (rx)  : ORIGIN = 0x20000000, LENGTH = 0x30000
  prog  (rx)  : ORIGIN = 0x20030000, LENGTH = 0x100000-0x30000
  ram   (!rx) : ORIGIN = 0x10000000, LENGTH = 0x10000
}

MPU_MIN_ALIGN = 1K;

INCLUDE ../kernel_layout.ld

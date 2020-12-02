MEMORY
{
  # Make space for the UF2 bootloader
  rom (rx)  : ORIGIN = 0x00026000, LENGTH = 192K
  prog (rx) : ORIGIN = 0x00056000, LENGTH = 640K
  ram (rwx) : ORIGIN = 0x20006000, LENGTH = 232K
}

MPU_MIN_ALIGN = 8K;
PAGE_SIZE = 4K;

INCLUDE ../kernel_layout.ld

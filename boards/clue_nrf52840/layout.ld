MEMORY
{
  # Make space for the UF2 bootloader (152K)
  rom (rx)  : ORIGIN = 0x00026000, LENGTH = 360K
  prog (rx) : ORIGIN = 0x00080000, LENGTH = 512K

  # if you need more space for apps, this configuration works
  # but there might be some MPU issues
  # rom (rx)  : ORIGIN = 0x00026000, LENGTH = 256K
  # prog (rx) : ORIGIN = 0x00066000, LENGTH = 616K

  ram (rwx) : ORIGIN = 0x20006000, LENGTH = 232K
}

MPU_MIN_ALIGN = 8K;
PAGE_SIZE = 4K;

INCLUDE ../kernel_layout.ld

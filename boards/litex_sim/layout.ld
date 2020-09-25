/* RAM starts at 0x40000000, the binary is loaded into ROM at 0x0 */

MEMORY
{
  rom (rx)  : ORIGIN = 0x00000000, LENGTH = 0x80000
  prog (rx) : ORIGIN = 0x00080000, LENGTH = 0x80000
  ram (rwx) : ORIGIN = 0x40000000, LENGTH = 0x10000000
}

MPU_MIN_ALIGN = 1K;

INCLUDE ../kernel_layout.ld

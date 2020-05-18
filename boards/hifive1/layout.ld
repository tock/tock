/* The HiFive1a board has 512 MB of flash. The first 0x400000 is reserved for
 * the default bootloader provided by SiFive. We also reserve room for apps to
 * make all of the linker files work, but don't really support them on this
 * chip.
 */

MEMORY
{
  rom (rx)  : ORIGIN = 0x20010000, LENGTH = 0x30000
  prog (rx) : ORIGIN = 0x20040000, LENGTH = 512M-0x430000
  ram (rwx) : ORIGIN = 0x80000000, LENGTH = 0x4000
}

MPU_MIN_ALIGN = 1K;

INCLUDE ../kernel_layout.ld

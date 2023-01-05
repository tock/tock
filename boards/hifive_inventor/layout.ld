/* The HiFive inventor board has 512 MiB of flash and 64 KiB of RAM.
 */

MEMORY
{
  rom (rx)  : ORIGIN = 0x20010000, LENGTH = 0x30000
  prog (rx) : ORIGIN = 0x20040000, LENGTH = 512M-0x430000
  ram (rwx) : ORIGIN = 0x80000000, LENGTH = 0x10000
}

MPU_MIN_ALIGN = 1K;

INCLUDE ../kernel_layout.ld

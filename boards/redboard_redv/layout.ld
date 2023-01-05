/* The RedV board has 4 MB of flash. The first 0x10000 is reserved for
 * the default bootloader provided by SiFive. We also reserve room for apps to
 * make all of the linker files work.
 */

MEMORY
{
  rom (rx)  : ORIGIN = 0x20010000, LENGTH = 192K
  prog (rx) : ORIGIN = 0x20040000, LENGTH = 4M-192K
  ram (rwx) : ORIGIN = 0x80000000, LENGTH = 16K
}

INCLUDE ../kernel_layout.ld

/**
 * QEMU emulated DRAM region. Tock is currently designed to be placed
 * at the start of DRAM, using the `-bios` option in qemu-system-riscv32.
 *
 * We are using 4MB of RAM, which easily fits into the 128MB default
 * assignment of QEMU, and we can have compact VMs with `-m 4MB`
 */

MEMORY
{
  rom (rx)  : ORIGIN = 0x80000000, LENGTH = 0x100000
  prog (rx) : ORIGIN = 0x80100000, LENGTH = 0x100000
  ram (rwx) : ORIGIN = 0x80200000, LENGTH = 0x200000
}

MPU_MIN_ALIGN = 1K;

INCLUDE ../kernel_layout.ld

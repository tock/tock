MEMORY
{
  /* uncomment this to boot from RAM */
  /* reset (rx)  : ORIGIN = 0x20000000, LENGTH = 16K
  rom (rx)  : ORIGIN = 0x20000100, LENGTH = 128K
  prog (rx) : ORIGIN = 0x20040000, LENGTH = 1K
  ram (rwx) : ORIGIN = 0x20004000, LENGTH = 240K */

  /* boot from Flash */
  rom (rx)  : ORIGIN = 0x10000000, LENGTH = 128K
  prog (rx) : ORIGIN = 0x20020000, LENGTH = 128K
  ram (rwx) : ORIGIN = 0x20000000, LENGTH = 128K
}

MPU_MIN_ALIGN = 8K;
PAGE_SIZE = 4K;

ENTRY(reset)

SECTIONS {
  .text  : ALIGN(4)
    {
        KEEP (*(.flash_bootloader));
        . = ALIGN (256);
    } > rom 
}

INCLUDE ../kernel_layout.ld
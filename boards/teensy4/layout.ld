/*
 * Teensy 4 Linker Script
 *
 * We do not utilize any of the tightly-coupled memory (TCM)
 * regions. Notably, there's another 512KB region that could be
 * used for RAM, or for instruction execution (ITCM), or for quickly-
 * accessible data (DTCM).
 *
 * NXP's iMXRT processors require a special boot section, which
 * we call '.boot' below. It contains the Firmware Configuration
 * Block (FCB) and the image vector table (IVT). We supply these
 * two components as prebuilt binary blobs.
 */

MEMORY
{
  rom   (rx)  : ORIGIN = 0x60000000, LENGTH = 512K
  prog  (rx)  : ORIGIN = 0x60080000, LENGTH = 512K
  ram   (rwx) : ORIGIN = 0x20200000, LENGTH = 512K
}

/* TODO figure out what this should be */
MPU_MIN_ALIGN = 8K;

SECTIONS
{
  .boot : 
  {
    /* Firmware Configuration Block (defined in source) */
    KEEP(*(.fcb));
    KEEP(*(.fcb_buffer));
    /* ------------------
     * Image Vector Table
     * ------------------
     *
     * See section 9.7.1.1 "Image vector table structure"
     * of the IMXRT1060 reference manual.
     */
    _ivt = .;
    LONG(0x402000D1); /* Header, magic number */
    /* Address to the vector table
     *
     * Relies on the fact that the vector table is the
     * first thing in ROM.
     */
    LONG(_stext);
    LONG(0x00000000);           /* RESERVED */
    LONG(0x00000000);           /* Device Configuration Data (unused) */
    LONG(_boot_data);           /* Address of boot data */
    LONG(_ivt);                 /* Address to the IVT (self) */
    LONG(0x00000000);           /* Command Sequence File (unused) */
    LONG(0x00000000);           /* RESERVED */
    /* ---------
     * Boot Data
     * ---------
     *
     * See section 9.7.1.2 "Boot data structure" of the
     * IMXRT1060 reference manual.
     */
    _boot_data = .;
    LONG(ORIGIN(rom));          /* Start of image */
    /* Size of the program image (assuming that we also need the boot section) */
    LONG((_etext-_stext) + (_erelocate-_srelocate) + SIZEOF(.boot));
    LONG(0x00000000);           /* Plugin flag (unused) */
    /* End of iMXRT10xx magic
     *
     * Need to align vector table on a 1024-byte boundary given the size of the table
     */
    . = ALIGN(1024);
  } > rom
}

INCLUDE ../kernel_layout.ld
ENTRY(_ivt);
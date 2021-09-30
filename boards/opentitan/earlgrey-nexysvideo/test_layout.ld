/*
 * This is used when building test binaries with `make test`.
 * It reduces the size for apps, as we don't use apps with the test
 * binaries.
 */

MEMORY
{
  rom   (rx)  : ORIGIN = 0x20000000, LENGTH = 0x50000
  /* The Nexys video has half the amount of flash.
   * So we only support up to 0x2006_0000 for apps
   * and 0x2006_0000 to 0x2008_0000 is for flash storage.
   */
  prog  (rx)  : ORIGIN = 0x20050000, LENGTH = 0x10000
  ram   (!rx) : ORIGIN = 0x10000000, LENGTH = 0x10000
}

MPU_MIN_ALIGN = 1K;
SECTIONS {
    /*
     * The flash header needs to match what the boot ROM for OpenTitan is
     * expecting.  At the moment, it contains only the entry point, but it
     * will eventually contain the signature -- and (hopefully?!) some
     * versioning information to make it slightly easier to debug when the
     * boot ROM and Tock are out of sync with respect to the definition...
     */
    .flash_header : {
        LONG(_stext)
    } > rom
}

INCLUDE ../../kernel_layout.ld

/**
 * ROM_ORIGIN and ROM_LENGTH define the memory region given to the kernel.
 *
 * Teensy 3.5/3.6 boards run the HalfKay bootloader on an external Cortex-M0, so
 * there is no need to allocate space for a bootloader unless there is a reason
 * to specifically use tockloader.
 *
 * [Teensy 3.6 Schematic]
 */
ROM_ORIGIN  = 0x00000000;
ROM_LENGTH  = 0x00020000;

/**
 * Program flash is 1M total.
 *
 * [Kinetis K66 Sub-Family Reference Manual Section 2.3]
 * [Teensy 3.6 Schematic]
 */
PROG_ORIGIN = 0x00020000;
PROG_LENGTH = 0x000E0000;

/**
 * SRAM is 256K total, split into an upper 192K chunk and a lower 64K chunk.
 *
 * [Kinetis K66 Sub-Family Reference Manual Section 4.10]
 */
RAM_ORIGIN  = 0x20000000;
RAM_LENGTH  = 192K;

/**
 * Minimum memory protection unit alignment size.
 * [Kinetis K66 Sub-Family Reference Manual Section 22.3.2]
 *
 * TODO: verify this number
 */
MPU_MIN_ALIGN = 8K;

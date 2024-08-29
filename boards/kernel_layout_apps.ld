/* Licensed under the Apache License, Version 2.0 or the MIT License. */
/* SPDX-License-Identifier: Apache-2.0 OR MIT                         */
/* Copyright Tock Contributors 2024.                                  */

/* This Tock linker script includes all of the standard Tock regions but also
 * adds a `.apps` section in flash at the start of the region where apps are
 * installed.
 *
 * This linker script can be used for boards that want to be able to manually
 * edit the generated .elf file to concatenate application binaries with the
 * kernel binary.
 */

INCLUDE ./kernel_layout.ld

SECTIONS
{
    /* Section for application binaries in flash.
     *
     * This section is put into the "prog" memory, which is reserved for
     * applications. This section is not used for the kernel, but including it
     * in the .elf file allows for concatenating application binaries with the
     * kernel.
     */
    .apps :
    {
        . = ALIGN(4);

        /* Include placeholder bytes in this section so that the linker
         * includes a segment for it. Otherwise the section will be empty and
         * the linker will ignore it when defining the segments.
         * If less then 4 bytes, some linkers set this section to size 0
         * and openocd fails to write it.
         *
         * An issue has been submitted https://github.com/raspberrypi/openocd/issues/25
         */
        BYTE(0xFF)
        BYTE(0xFF)
        BYTE(0xFF)
        BYTE(0xFF)
    } > prog
}

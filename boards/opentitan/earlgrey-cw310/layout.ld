/* Licensed under the Apache License, Version 2.0 or the MIT License. */
/* SPDX-License-Identifier: Apache-2.0 OR MIT                         */
/* Copyright Tock Contributors 2023.                                  */

MEMORY
{
  rom   (rx)  : ORIGIN = 0x20000000, LENGTH = 0x30000
  /* Support up to 0x2009_0000 for apps
   * and 0x2009_0000 to 0x2010_0000 is for flash storage.
   */
  prog  (rx)  : ORIGIN = 0x20030000, LENGTH = 0x60000
  /* The first 0x650 bytes of RAM are reserved for the boot
   * ROM, so we have to ignore that space.
   * See https://github.com/lowRISC/opentitan/blob/master/sw/device/silicon_creator/lib/base/static_critical.ld
   * for details
   */
  ram   (!rx) : ORIGIN = 0x10000650, LENGTH = 0x20000 - 0x650
}

SECTIONS {
    .manifest ORIGIN(rom):
    {
        _manifest = .;
        /* see: sw/device/silicon_creator/lib/manifest.h */
        . += 384; /* rsa_signature */
        . += 4;   /* usage_constraints.selector_bits */
        . += 32;  /* usage_constraints.device_id */
        . += 4;   /* usage_constraints.manuf_state_creator */
        . += 4;   /* usage_constraints.manuf_state_owner */
        . += 4;   /* usage_constraints.life_cycle_state */
        . += 384; /* rsa_modulus */
        . += 4;   /* address_translation */
        . += 4;   /* identifier */
        . += 4;   /* manifest_version */
        . += 4;   /* signed_region_end */
        . += 4;   /* length */
        . += 4;   /* version_major */
        . += 4;   /* version_minor */
        . += 4;   /* security_version */
        . += 8;   /* timestamp */
        . += 32;  /* binding_value */
        . += 4;   /* max_key_version */
        . += 4;   /* code_start */
        . += 4;   /* code_end */
        LONG(_stext - ORIGIN(rom)); /* . = . + 4; entry_point */
        /* manifest extension table */
        /* see: sw/device/silicon_creator/lib/base/chip.h */
        /* CHIP_MANIFEST_EXT_TABLE_COUNT = 15 */
        /* sizeof(manifest_ext_table_entry) = 8 */
        . += 120; /* manifest_ext_table */
    } > rom
}
ASSERT (((_etext - _manifest) > 0), "Error: PMP and Flash protection setup assumes _etext follows _manifest");
INCLUDE ../../kernel_layout.ld

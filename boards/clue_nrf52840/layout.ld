/* Licensed under the Apache License, Version 2.0 or the MIT License. */
/* SPDX-License-Identifier: Apache-2.0 OR MIT                         */
/* Copyright Tock Contributors 2023.                                  */

MEMORY
{
  # Make space for the UF2 bootloader (152K)
  rom (rx)  : ORIGIN = 0x00036000, LENGTH = 296K
  # use without tock-bootloader
  # rom (rx)  : ORIGIN = 0x00026000, LENGTH = 360K
  prog (rx) : ORIGIN = 0x00080000, LENGTH = 512K

  # if you need more space for apps, this configuration works
  # but there might be some MPU issues and assumes tock-bootloader
  # is not flashed
  # rom (rx)  : ORIGIN = 0x00026000, LENGTH = 256K
  # prog (rx) : ORIGIN = 0x00066000, LENGTH = 616K

  ram (rwx) : ORIGIN = 0x20006000, LENGTH = 232K
}

PAGE_SIZE = 4K;

INCLUDE ../kernel_layout.ld

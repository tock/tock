# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2022.

# Makefile for building the tock kernel for the HiFive Inventor platform

TARGET=riscv32imac-unknown-none-elf
PLATFORM=hifive_inventor
QEMU ?= qemu-system-riscv32

include ../Makefile.common

# Default target for installing the kernel.
.PHONY: install
install: flash-jlink

TOCKLOADER=tockloader
TOCKLOADER_JTAG_FLAGS = --jlink --board hifive1b
KERNEL_ADDRESS = 0x20010000

# upload kernel over JTAG
.PHONY: flash-jlink
flash-jlink: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).bin
	$(TOCKLOADER) $(TOCKLOADER_GENERAL_FLAGS) flash --address $(KERNEL_ADDRESS) $(TOCKLOADER_JTAG_FLAGS) $<

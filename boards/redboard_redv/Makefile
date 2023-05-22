# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2022.

# Makefile for building the tock kernel for the RedV platform

TARGET=riscv32imac-unknown-none-elf
PLATFORM=redboard_redv
QEMU ?= qemu-system-riscv32

include ../Makefile.common

# Default target for installing the kernel.
.PHONY: install
install: flash

flash: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf
	openocd \
		-c "source [find board/sifive-hifive1-revb.cfg]; program $<; resume 0x20000000; exit"

qemu: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf
	$(QEMU) -M sifive_e,revb=true -kernel $^  -nographic

qemu-app: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf
	$(QEMU) -M sifive_e,revb=true -kernel $^ -device loader,file=$(APP),addr=0x20010000 -nographic


TOCKLOADER=tockloader
TOCKLOADER_JTAG_FLAGS = --jlink --board hifive1b
KERNEL_ADDRESS = 0x20010000

# upload kernel over JTAG
.PHONY: flash
flash-jlink: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).bin
	$(TOCKLOADER) $(TOCKLOADER_GENERAL_FLAGS) flash --address $(KERNEL_ADDRESS) $(TOCKLOADER_JTAG_FLAGS) $<

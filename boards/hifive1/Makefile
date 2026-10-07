# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2022.

# Makefile for building the tock kernel for the HiFive1 platform

include ../Makefile.common

# Default target for installing the kernel.
.PHONY: install
install: flash

flash: $(TARGET_PATH)/release/$(PLATFORM).elf
	openocd \
		-c "source [find board/sifive-hifive1-revb.cfg]; program $<; resume 0x20000000; exit"

TOCKLOADER=tockloader
TOCKLOADER_JTAG_FLAGS = --jlink --board hifive1b
KERNEL_ADDRESS = 0x20010000

# upload kernel over JTAG
.PHONY: flash
flash-jlink: $(TARGET_PATH)/release/$(PLATFORM).bin
	$(TOCKLOADER) $(TOCKLOADER_GENERAL_FLAGS) flash --address $(KERNEL_ADDRESS) $(TOCKLOADER_JTAG_FLAGS) $<


# QEMU version

QEMU_CMD ?= qemu-system-riscv32

QEMU_CMDLINE_EXTRA ?= \

QEMU_CMDLINE := \
    $(QEMU_CMD) \
    -M sifive_e,revb=true \
    -nographic \
    $(QEMU_CMDLINE_EXTRA)

init:
	tockloader local-board set hifive1 \
		--app-address 0x20040000 \
		--flash-address 0x20040000 \
		--binary-path hifive1-qemu.bin \
		--arch rv32imac \

run: $(TARGET_PATH)/release/$(PLATFORM).elf
	$(QEMU_CMDLINE) -kernel $^ -device loader,file=hifive1-qemu.bin,addr=0x20040000

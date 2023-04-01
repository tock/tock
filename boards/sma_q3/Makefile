# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2022.

# Makefile for building the tock kernel for the SMA Q3 smart watch

TARGET=thumbv7em-none-eabi
PLATFORM=sma_q3

include ../Makefile.common

TOCKLOADER=tockloader

# Where in the nrf52 flash to load the kernel with `tockloader`
KERNEL_ADDRESS=0x00000

TOCKLOADER_SWD_FLAGS = --openocd --board sma_q3

# Default target for installing the kernel.
.PHONY: install
install: flash

# Upload the kernel over SWD
.PHONY: flash
flash: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).bin
	$(TOCKLOADER) flash --address $(KERNEL_ADDRESS) $(TOCKLOADER_SWD_FLAGS) $<

# Upload the kernel over serial/bootloader
.PHONY: program
program: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).hex
	$(error Cannot program SMA Q3 over USB. Use \`make flash\` and SWD)

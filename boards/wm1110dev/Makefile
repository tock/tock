# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2022.

# Makefile for building the tock kernel for the Arduino Nano 33 BLE board.

TOCK_ARCH=cortex-m4
TARGET=thumbv7em-none-eabi
PLATFORM=wm1110dev

include ../Makefile.common

ifdef PORT
  FLAGS += --port $(PORT)
endif

# Default target for installing the kernel.
.PHONY: install
install: program

# Upload the kernel using tockloader and the tock bootloader
.PHONY: program
program: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).bin
	tockloader $(FLAGS) flash --address 0x10000 $<

.PHONY: flash-bootloader
flash-bootloader:
	tockloader flash --address 0 bootloader.bin
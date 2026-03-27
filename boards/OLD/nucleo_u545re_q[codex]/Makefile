# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2026.

# Makefile for building the Tock kernel for the NUCLEO-U545RE-Q platform

# Avoid relying on unstable `cargo config get` in Makefile.common.
TARGET=thumbv8m.main-none-eabi

include ../Makefile.common

OPENOCD=openocd
OPENOCD_INTERFACE?=interface/stlink-dap.cfg
OPENOCD_TARGET?=target/stm32u5x.cfg
OPENOCD_TRANSPORT?=dapdirect_swd
OPENOCD_SPEED_KHZ?=200

# Default target for installing the kernel.
.PHONY: install
install: flash

.PHONY: flash-debug
flash-debug: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/debug/$(PLATFORM).elf
	$(OPENOCD) -f $(OPENOCD_INTERFACE) -f $(OPENOCD_TARGET) -c "transport select $(OPENOCD_TRANSPORT); adapter speed $(OPENOCD_SPEED_KHZ); reset_config srst_only srst_nogate connect_assert_srst; init; reset halt; adapter speed $(OPENOCD_SPEED_KHZ); program $< verify reset exit"

.PHONY: flash
flash: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf
	$(OPENOCD) -f $(OPENOCD_INTERFACE) -f $(OPENOCD_TARGET) -c "transport select $(OPENOCD_TRANSPORT); adapter speed $(OPENOCD_SPEED_KHZ); reset_config srst_only srst_nogate connect_assert_srst; init; reset halt; adapter speed $(OPENOCD_SPEED_KHZ); program $< verify reset exit"

.PHONY: program
program: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/debug/$(PLATFORM).elf
	$(error See README.md and update this section accordingly)

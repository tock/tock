# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright OxidOS Automotive 2026.

# Makefile for building the tock kernel for the NUCLEO-U545RE-Q platform
#

include ../Makefile.common

# Toolchain Variables
PROBE_RS ?= probe-rs
CHIP = STM32U545RETx
OBJCOPY ?= arm-none-eabi-objcopy

# NOTE: STM32U5 Metadata Stripping
# The standard ELF file produced by the compiler contains metadata sections
# (like .ARM.attributes) that sometimes point to System Memory addresses
# (e.g., 0x0BFA0000). Many flashing tools, including probe-rs, will attempt
# to write to these addresses, which are protected on the STM32U5, causing
# the flash process to crash.
#
# We use 'objcopy -j' to extract ONLY the sections that belong
# in the physical Flash memory (0x0C000000). This ensures a clean,
# safe deployment to the hardware.

# Default target for installing the kernel.
.PHONY: install
install: flash

.PHONY: flash-debug
flash-debug: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/debug/$(PLATFORM).elf
	@echo "--- Preparing Clean Debug Kernel ELF ---"
	$(OBJCOPY) -j .text -j .ARM.exidx -j .storage -j .relocate $< $<.clean
	@echo "--- Flashing Debug Kernel via probe-rs ---"
	$(PROBE_RS) download --chip $(CHIP) --allow-erase-all $<.clean
	$(PROBE_RS) reset --chip $(CHIP)

.PHONY: flash
flash: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf
	@echo "--- Preparing Clean Release Kernel ELF ---"
	$(OBJCOPY) -j .text -j .ARM.exidx -j .storage -j .relocate $< $<.clean
	@echo "--- Flashing Release Kernel via probe-rs ---"
	$(PROBE_RS) download --chip $(CHIP) --allow-erase-all $<.clean
	$(PROBE_RS) reset --chip $(CHIP)

.PHONY: program
program:
	# Note: tockloader does not natively wrap probe-rs yet.
	# The --openocd flag was removed; tockloader will use its default method (usually serial/jlink).
	tockloader install --board nucleo_u545re_q $(APP)

# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2022.

# Makefile for building the tock kernel for the NUCLEO-F429ZI platform
#

include ../Makefile.common

PROBE_RS=probe-rs

# The specific chip on the NUCLEO-F429ZI board.
# probe-rs needs this to know what it's flashing.
iCHIP=LPC55S69JBD100

APP=/home/pc1/Projects/libtock-c/examples/blink/build/cortex-m7/cortex-m7.tbf
KERNEL=$(TOCK_ROOT_DIRECTORY)/target/$(TARGET)/release/$(PLATFORM).elf
KERNEL_WITH_APP=$(TOCK_ROOT_DIRECTORY)/target/$(TARGET)/release/$(PLATFORM)-app.elf

# Default target for installing the kernel.
.PHONY: install
install: flash

# flash-debug is no longer needed as a separate target, but kept for
# compatibility. `make debug` is the new recommended command.
.PHONY: flash-debug
flash-debug: debug

# Flash the release version of the kernel to the board.
.PHONY: flash
flash: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf
	$(PROBE_RS) flash --chip $(CHIP) --erase-all $<

# Flash the debug version and attach an RTT console to view kernel logs.
# This is the recommended command for debugging.
.PHONY: debug
debug: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/debug/$(PLATFORM).elf
	$(PROBE_RS) run --chip $(iCHIP) $<

# An alias for the default `install` target.
.PHONY: program
program: release
	arm-none-eabi-objcopy --set-section-flags .apps=LOAD,ALLOC $(KERNEL) $(KERNEL_WITH_APP)
	arm-none-eabi-objcopy --update-section .apps=$(APP) $(KERNEL_WITH_APP)
	$(PROBE_RS) run --chip $(iCHIP) $(KERNEL_WITH_APP)

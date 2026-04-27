# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright OxidOS Automotive 2026.

# Makefile for building the tock kernel for the NUCLEO-U545RE-Q platform
#

include ../Makefile.common

OPENOCD=openocd
OPENOCD_OPTIONS=-f interface/stlink.cfg -f target/stm32u5x.cfg

# Default target for installing the kernel.
.PHONY: install
install: flash

.PHONY: flash-debug
flash-debug: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/debug/$(PLATFORM).elf
	$(OPENOCD) $(OPENOCD_OPTIONS) -c "init; halt; stm32l4x mass_erase 0; program $< verify reset exit"

.PHONY: flash
flash: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf
	$(OPENOCD) $(OPENOCD_OPTIONS) -c "init; halt; stm32l4x mass_erase 0; program $< verify reset exit"

.PHONY: program
program:
	tockloader install --openocd --board nucleo_u545re_q $(APP)

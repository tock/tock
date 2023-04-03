# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2022.

# Makefile for building the tock kernel for the Teensy 4

TARGET=thumbv7em-none-eabi
PLATFORM=teensy40

include ../Makefile.common

# Default target for installing the kernel.
.PHONY: install
install: program

%.hex: %.elf
	$(Q)$(OBJCOPY) -O ihex $< $@

kernel: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf

.PHONY: program
program: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).hex
	$(Q)teensy_loader_cli --mcu=TEENSY40 -w -v $<

.PHONY: program-app
program-app: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf
	@: $(if $(value APP),,$(error Please set APP to the path of a TBF file to program applications))
	$(Q)arm-none-eabi-objcopy --update-section .apps=$(APP) $< $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM)-apps.elf
	$(Q)$(OBJCOPY) -O ihex $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM)-apps.elf $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM)-apps.hex
	teensy_loader_cli --mcu=TEENSY40 -w -v $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM)-apps.hex

# Unsupported; there's no easily-accessible JTAG interface
.PHONY: flash
flash:
	echo "Use 'make program' to program the Teensy 4"
	exit 1

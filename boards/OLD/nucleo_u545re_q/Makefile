# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2022.

# Makefile for building the Tock kernel for the Nucleo-32 STM32U545 platform

include ../Makefile.common

OPENOCD=openocd
OPENOCD_OPTIONS=-f interface/stlink-dap.cfg -f target/stm32u5x.cfg \
	-c "transport select dapdirect_swd" \
	-c "reset_config srst_only srst_nogate connect_assert_srst"

# Default target for installing the kernel.
.PHONY: install
install: flash

.PHONY: flash-debug
flash-debug: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/debug/$(PLATFORM).elf
	probe-rs run $< --chip stm32u545retxq
.PHONY: flash
flash: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf
	$(OPENOCD) $(OPENOCD_OPTIONS) -c "init; reset halt; flash write_image erase $<; verify_image $<; reset; shutdown"

#.PHONY: program
#program: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/debug/$(PLATFORM).elf
#	$(error See README.md and update this section accordingly)
APP=../../../libtock-c/examples/blink/build/cortex-m4/cortex-m4.tbf
KERNEL=$(TOCK_ROOT_DIRECTORY)/target/$(TARGET)/debug/$(PLATFORM).elf
KERNEL_WITH_APP=$(TOCK_ROOT_DIRECTORY)/target/$(TARGET)/debug/$(PLATFORM)-app.elf

.PHONY: program
program: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/debug/$(PLATFORM).elf
	arm-none-eabi-objcopy --set-section-flags .apps=LOAD,ALLOC $(KERNEL) $(KERNEL_WITH_APP)
	arm-none-eabi-objcopy --update-section .apps=$(APP) $(KERNEL_WITH_APP)
	probe-rs run $(KERNEL_WITH_APP) --chip stm32u545retxq

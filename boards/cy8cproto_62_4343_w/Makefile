# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright OxidOS Automotive 2025 SRL.

include ../Makefile.common

install: flash
.PHONY: install

flash: release
	gdb-multiarch -q --command=openocd.flash ../../target/thumbv6m-none-eabi/release/cy8cproto_62_4243_w.elf
.PHONY: flash

# Define the path to the TBF file to flash an application
APP=
KERNEL=$(TOCK_ROOT_DIRECTORY)/target/$(TARGET)/release/$(PLATFORM).elf
KERNEL_WITH_APP=$(TOCK_ROOT_DIRECTORY)/target/$(TARGET)/release/$(PLATFORM)-app.elf

.PHONY: program
program: release
ifeq ($(APP),)
	$(error Please define the APP variable with the TBF file to flash an application)
endif
	arm-none-eabi-objcopy --set-section-flags .apps=LOAD,ALLOC $(KERNEL) $(KERNEL_WITH_APP)
	arm-none-eabi-objcopy --update-section .apps=$(APP) $(KERNEL_WITH_APP)
	gdb-multiarch -q --command=openocd.flash $(KERNEL_WITH_APP)

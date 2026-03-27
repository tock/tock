# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Infineon Technologies AG 2026.

include ../Makefile.common

DEBUG ?= 0

ifeq ($(DEBUG), 1)
	BUILD_TYPE = debug
else
	BUILD_TYPE = release
endif

# Define the path to the TBF file to flash an application
APP?=
KERNEL=$(TOCK_ROOT_DIRECTORY)/target/$(TARGET)/$(BUILD_TYPE)/$(PLATFORM).elf
KERNEL_WITH_APP=$(TOCK_ROOT_DIRECTORY)/target/$(TARGET)/$(BUILD_TYPE)/$(PLATFORM)-app.elf

OPENOCD_ROOT = /opt/ModusToolboxProgtools-1.7/openocd

install: flash
.PHONY: install

build-with-app: $(BUILD_TYPE) # let cargo rebuild
ifeq ($(APP),)
	$(info Built kernel without application (old application will still be present))
else
	$(info Building kernel with application $(APP))
	arm-none-eabi-objcopy --set-section-flags .apps=LOAD,ALLOC $(KERNEL) $(KERNEL_WITH_APP)
	arm-none-eabi-objcopy --update-section .apps=$(APP) $(KERNEL_WITH_APP)
endif

.PHONY: flash
flash: build-with-app
	probe-rs run --chip PSC3M5FDS2AFQ1 $(KERNEL_WITH_APP)

.PHONY: program
program: build-with-app
	${OPENOCD_ROOT}/bin/openocd -f openocd.tcl $(OPENOCD_OPTIONS) -c "init; reset init; program $(KERNEL_WITH_APP); reset; shutdown"

   
   

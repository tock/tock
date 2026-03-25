# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright OxidOS Automotive 2025 SRL.

include ../Makefile.common

# Define the path to the TBF file to flash an application
APP=
KERNEL=$(TOCK_ROOT_DIRECTORY)/target/$(TARGET)/release/$(PLATFORM).elf
KERNEL_WITH_APP=$(TOCK_ROOT_DIRECTORY)/target/$(TARGET)/release/$(PLATFORM)-app.elf

install: flash
.PHONY: install

.PHONY: flash
flash: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).bin
	probe-rs download --chip CY8C624AAZI-S2D44 --binary-format bin --base-address 0x10000000 $<


.PHONY: program
program: release
ifeq ($(APP),)
	$(error Please define the APP variable with the TBF file to flash an application)
endif
	arm-none-eabi-objcopy --set-section-flags .apps=LOAD,ALLOC $(KERNEL) $(KERNEL_WITH_APP)
	arm-none-eabi-objcopy --update-section .apps=$(APP) $(KERNEL_WITH_APP)
	${OPENOCD_ROOT}/bin/openocd $(OPENOCD_OPTIONS) -c "program $(KERNEL_WITH_APP); verify_image $(KERNEL_WITH_APP); reset; shutdown"

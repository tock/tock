# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright OxidOS Automotive 2025 SRL.

include ../Makefile.common

# Define the path to the TBF file to flash an application
APP=
KERNEL=$(TOCK_ROOT_DIRECTORY)/target/$(TARGET)/release/$(PLATFORM).elf
KERNEL_WITH_APP=$(TOCK_ROOT_DIRECTORY)/target/$(TARGET)/release/$(PLATFORM)-app.elf

OPENOCD_ROOT = /opt/ModusToolboxProgtools-1.7/openocd

install: flash
.PHONY: install

.PHONY: flash
flash: $(KERNEL)
	probe-rs download --chip PSC3M5FDS2AFQ1 --binary-format bin --base-address 0x22000000 $<
# TODO


.PHONY: program
program: release
ifeq ($(APP),)
	$(error Please define the APP variable with the TBF file to flash an application)
endif
	arm-none-eabi-objcopy --set-section-flags .apps=LOAD,ALLOC $(KERNEL) $(KERNEL_WITH_APP)
	arm-none-eabi-objcopy --update-section .apps=$(APP) $(KERNEL_WITH_APP)
	${OPENOCD_ROOT}/bin/openocd -f openocd.cfg $(OPENOCD_OPTIONS) -c "init; reset init; program $(KERNEL_WITH_APP); reset; shutdown"
# TODO?: only works without verify

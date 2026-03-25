# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright OxidOS Automotive 2025 SRL.

include ../Makefile.common

# Define the path to the TBF file to flash an application
APP=
KERNEL=$(TOCK_ROOT_DIRECTORY)/target/$(TARGET)/release/$(PLATFORM).elf
KERNEL_WITH_APP=$(TOCK_ROOT_DIRECTORY)/target/$(TARGET)/release/$(PLATFORM)-app.elf

OPENOCD_ROOT = /opt/ModusToolboxProgtools-1.7/openocd
OPENOCD_OPTIONS = -s ${OPENOCD_ROOT}/scripts \
					-f interface/kitprog3.cfg \
					-c "set TARGET_VARIANT psc3m5" \
					-f target/infineon/psc3.cfg
KITPROG_ID = 0E18119E02272400

install: flash
.PHONY: install

.PHONY: flash
flash: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).bin
	probe-rs download --chip PSC3M5FDS2AFQ1 --binary-format bin --base-address 0x32000000 $<


.PHONY: program
program: release
ifeq ($(APP),)
	$(error Please define the APP variable with the TBF file to flash an application)
endif
	arm-none-eabi-objcopy --set-section-flags .apps=LOAD,ALLOC $(KERNEL) $(KERNEL_WITH_APP)
	arm-none-eabi-objcopy --update-section .apps=$(APP) $(KERNEL_WITH_APP)
	${OPENOCD_ROOT}/bin/openocd $(OPENOCD_OPTIONS) -c "cmsis_dap_serial $(KITPROG_ID); program $(KERNEL_WITH_APP) verify; reset; shutdown"

# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2022.

# Makefile for building the tock kernel for the i.MX RT1052 platform

TARGET=thumbv7em-none-eabi
PLATFORM=imxrt1050-evkb

include ../Makefile.common

# Default target for installing the kernel.
.PHONY: install
install: flash

.PHONY: flash
flash: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf
	crt_emu_cm_redlink \
		--flash-load-exec "$(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf" \
		-p MIMXRT1052xxxxB --ConnectScript RT1050_connect.scp \
		--flash-driver= -x . --flash-dir $(MCUX_IDE_BIN)/Flash --flash-hashing

.PHONY: flash-app
flash-app: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf
	@: $(if $(value APP),,$(error Please set APP to the path of a TBF file to program applications))
	arm-none-eabi-objcopy --update-section .apps=$(APP) $< \
		$(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM)-apps.elf
	crt_emu_cm_redlink \
		--flash-load-exec "$(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM)-apps.elf" \
		-p MIMXRT1052xxxxB --ConnectScript RT1050_connect.scp \
		--flash-driver= -x . --flash-dir $(MCUX_IDE_BIN)/Flash --flash-hashing

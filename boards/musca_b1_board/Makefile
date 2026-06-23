# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2026.

include ../Makefile.common

QEMU_CMD              := qemu-system-arm
WORKING_QEMU_VERSIONS := 10.0.2
BROKEN_QEMU_VERSIONS  := <= 8.1.5

# Musca-B1 specific parameters
QEMU_BASE_CMDLINE := \
    $(QEMU_CMD) \
    -machine musca-b1 \
    -cpu cortex-m33 \
    -nographic \
    -semihosting

APP?=
MUSCA_ELF := $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/debug/$(PLATFORM).elf
MUSCA_ELF_WITH_APP := $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/debug/$(PLATFORM)-app.elf

.PHONY: build-with-app
build-with-app: debug # let cargo rebuild
ifeq ($(APP),)
	$(info Built kernel without application (old application will still be present))
	cp $(MUSCA_ELF) $(MUSCA_ELF_WITH_APP)
else
	$(info Building kernel with application $(APP))
	arm-none-eabi-objcopy --set-section-flags .apps=LOAD,ALLOC $(MUSCA_ELF) $(MUSCA_ELF_WITH_APP)
	arm-none-eabi-objcopy --update-section .apps=$(APP) $(MUSCA_ELF_WITH_APP)
endif

.PHONY: qemu
qemu: build-with-app
	@echo "To exit type C-a x"
	@echo
	$(QEMU_BASE_CMDLINE) -kernel $(MUSCA_ELF_WITH_APP)

.PHONY: qemu-gdb-listen
qemu-gdb-listen: build-with-app
	@killall qemu-system-arm || true
	@echo Starting QEMU...
	$(QEMU_BASE_CMDLINE) -kernel $(MUSCA_ELF_WITH_APP) -S -gdb tcp::1234

.PHONY: gdb-connect
gdb-connect:
	arm-none-eabi-gdb \
	 -ex "directory $(TOCK_ROOT_DIRECTORY)" \
	 -ex "set architecture armv8-m.main" \
	 -ex "target remote localhost:1234" \
	 -ex "set confirm off" \
	 -ex "file $(MUSCA_ELF)"

# somehow sp is not set correctly by gdb/qemu
.PHONY: gdb-connect-elf
gdb-connect-elf:
	arm-none-eabi-gdb -q \
	 -ex "directory $(TOCK_ROOT_DIRECTORY)" \
	 -ex "target remote localhost:1234" \
	 -ex "set confirm off" \
	 -ex "set architecture armv8-m.main" \
	 -ex "file $(MUSCA_ELF)" \
	 -ex "load" \
	 -ex "set \$$sp = 0x30003000" \
	 -ex "b cortexv7m::hard_fault_handler_arm_v7m" 

.PHONY: flash
flash:
	@echo "Flashing to Musca-B1 is not supported yet. Please use QEMU for now."

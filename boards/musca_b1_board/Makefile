# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2025.

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

MUSCA_ELF := $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/debug/$(PLATFORM).elf
MUSCA_BIN := $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/debug/$(PLATFORM).bin
QEMU_MUSCA_B1_BIN := qemu_musca_b1.bin

# Set this as the default local board to write apps to with tockloader
.PHONY: qemu-init
qemu-init:
	tockloader local-board set qemu_musca_b1 --binary-path $(QEMU_MUSCA_B1_BIN)

# Write the kernel to the flash file with tockloader
.PHONY: qemu-flash
qemu-flash: $(MUSCA_BIN)
	tockloader flash --address 0x10000000 $<

# does not overwrite installed app
$(QEMU_MUSCA_B1_BIN): qemu-flash

.PHONY: qemu
qemu: $(QEMU_MUSCA_B1_BIN)
	@echo "To exit type C-a x"
	@echo
	$(QEMU_BASE_CMDLINE) -device loader,file=$<,addr=0x10000000

.PHONY: qemu-elf
qemu-elf: $(MUSCA_ELF)
	$(QEMU_BASE_CMDLINE) -kernel '$<'

.PHONY: qemu-gdb-listen
qemu-gdb-listen: $(MUSCA_ELF)
	$(QEMU_BASE_CMDLINE) -S -gdb tcp::1234

.PHONY: qemu-gdb-listen-bin
qemu-gdb-listen-bin: $(QEMU_MUSCA_B1_BIN)
	$(QEMU_BASE_CMDLINE) -device loader,file=$<,addr=0x10000000 -S -gdb tcp::1234

.PHONY: gdb-connect
gdb-connect: $(MUSCA_ELF)
	arm-none-eabi-gdb \
	 -ex "directory $(TOCK_ROOT_DIRECTORY)" \
	 -ex "set architecture armv8-m.main" \
	 -ex "target remote localhost:1234" \
	 -ex "set confirm off" \
	 -ex "file $<"

# somehow sp is not set correctly by gdb/qemu
.PHONY: gdb-connect-elf
gdb-connect-elf: $(MUSCA_ELF)
	arm-none-eabi-gdb -q \
	 -ex "directory $(TOCK_ROOT_DIRECTORY)" \
	 -ex "target remote localhost:1234" \
	 -ex "set confirm off" \
	 -ex "set architecture armv8-m.main" \
	 -ex "file $<" \
	 -ex "load" \
	 -ex "set \$$sp = 0x30003000" \
	 -ex "b cortexv7m::hard_fault_handler_arm_v7m" 


.PHONY: flash
flash:
	@echo "Flashing to Musca-B1 is not supported yet. Please use QEMU for now."

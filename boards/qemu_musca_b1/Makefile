# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2025.

include ../Makefile.common

QEMU_CMD              := qemu-system-arm
WORKING_QEMU_VERSIONS := 8.2.7, 9.1.3, 9.2.3, 10.0.2
BROKEN_QEMU_VERSIONS  := <= 8.1.5

# Musca-B1 specific parameters
QEMU_BASE_CMDLINE := \
    $(QEMU_CMD) \
    -machine musca-b1 \
    -cpu cortex-m33 \
    -nographic \
    -semihosting


# Set this as the default local board to write apps to with tockloader
.PHONY: init
init:
	tockloader local-board set qemu_musca_b1 --binary-path qemu_musca_b1.bin

# Write the kernel to the flash file with tockloader
.PHONY: install
install: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).bin
	tockloader flash --address 0x10000000 $<

qemu_musca_b1.bin: install

.PHONY: run
run: qemu_musca_b1.bin | install
	@echo
	@echo -e "Running $$($(QEMU_CMD) --version | head -n1)"\
	  "(tested: $(WORKING_QEMU_VERSIONS); known broken: $(BROKEN_QEMU_VERSIONS)) with\n"\
          " - kernel $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf"
	@echo "To exit type C-a x"
	@echo
	$(QEMU_BASE_CMDLINE) -device loader,file=$<,addr=0x10000000

run-elf: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf
	$(QEMU_BASE_CMDLINE) -kernel '$<'

gdb-listen: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf
	$(QEMU_BASE_CMDLINE) -S -gdb tcp::1234

gdb-listen-bin: qemu_musca_b1.bin | install
	$(QEMU_BASE_CMDLINE) -device loader,file=$<,addr=0x10000000 -S -gdb tcp::1234

gdb-connect: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf
	arm-none-eabi-gdb \
	 -ex "directory $(TOCK_ROOT_DIRECTORY)" \
	 -ex "set architecture armv8-m.main" \
	 -ex "target remote localhost:1234" \
	 -ex "set confirm off" \
	 -ex "file $<"

# somehow sp is not set correctly by gdb/qemu
gdb-connect-elf: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf
	arm-none-eabi-gdb -q \
	 -ex "directory $(TOCK_ROOT_DIRECTORY)" \
	 -ex "target remote localhost:1234" \
	 -ex "set confirm off" \
	 -ex "set architecture armv8-m.main" \
	 -ex "file $<" \
	 -ex "load" \
	 -ex "set \$$sp = 0x30003000" \
	 -ex "b cortexv7m::hard_fault_handler_arm_v7m" 


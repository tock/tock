# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright OxidOS Automotive 2026.

TOCK_ROOT_DIRECTORY = ../../
PLATFORM = nucleo_u545re_q

# Path to the compiled app
# ADJUST based on the path to the tbf
APP_PATH = /mnt/c/Work/munca/libtock-c/examples/blink/build/cortex-m4/cortex-m4.tbf

include ../Makefile.common

# Target path for the final kernel ELF
KERNEL_ELF = $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM)

# J-Link Configuration
JLINK_EXE ?= JLinkExe
JLINK_DEVICE = STM32U545RE
JLINK_IF = SWD
JLINK_SPEED = 4000

# --- TARGET: binary (Builds the merged binary and unified ELF) ---
.PHONY: binary
binary: $(KERNEL_ELF)
	@echo "--- Extracting Kernel Binary ---"
	$(Q)arm-none-eabi-objcopy -O binary $(KERNEL_ELF) kernel.bin
	@echo "--- Padding and Merging App ---"
	$(Q)dd if=/dev/zero of=full_merged.bin bs=1K count=256
	$(Q)dd if=kernel.bin of=full_merged.bin conv=notrunc
	$(Q)cat $(APP_PATH) >> full_merged.bin
	@echo "--- Generating Unified ELF via Linker ---"
	# We use the linker to create a proper loadable segment.
	# The input 'full_merged.bin' becomes a section called '.data' in the ELF.
	$(Q)arm-none-eabi-ld -b binary --entry 0x0C000000 -Ttext 0x0C000000 full_merged.bin -o full_merged.elf

# --- TARGET: flash (Flashes ONLY the kernel using probe-rs) ---
.PHONY: flash
flash: $(KERNEL_ELF)
	@echo "--- Preparing Clean Kernel ELF ---"
	$(Q)arm-none-eabi-objcopy -j .text -j .ARM.exidx -j .storage -j .relocate \
	    $(KERNEL_ELF) kernel_clean.elf
	@echo "--- Flashing KERNEL only ---"
	$(Q)probe-rs download --chip STM32U545RETx --allow-erase-all kernel_clean.elf
	$(Q)probe-rs reset --chip STM32U545RETx

# --- TARGET: binary_flash (Flashes KERNEL + BLINK APP using probe-rs) ---
.PHONY: binary_flash
binary_flash: binary
	@echo "--- Preparing Clean Full ELF ---"
	# NOTE: ld -b binary names the section '.data', so we must keep that.
	$(Q)arm-none-eabi-objcopy -j .data full_merged.elf full_merged_clean.elf
	@echo "--- Flashing KERNEL + BLINK APP ---"
	$(Q)probe-rs download --chip STM32U545RETx --allow-erase-all full_merged_clean.elf
	$(Q)probe-rs reset --chip STM32U545RETx

.PHONY: install
install: flash

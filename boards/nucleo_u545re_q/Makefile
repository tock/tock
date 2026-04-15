# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright OxidOS Automotive 2026.

TOCK_ROOT_DIRECTORY = ../../
PLATFORM = nucleo_u545re_q

# Path to the compiled app (must be provided as an environment variable)
APP_PATH ?=

include ../Makefile.common

# Check that APP_PATH is provided when running targets that need it
ifeq ($(APP_PATH),)
	# Only throw the error for the 'app' and 'app_flash' targets
	ifneq ($(filter app app_flash app_flash_ocd,$(MAKECMDGOALS)),)
	$(error ERROR: APP_PATH is not set. Run with 'make app APP_PATH=/path/to/app.tbf')
	endif
endif

# Target path for the final kernel ELF
KERNEL_ELF = $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM)

# OpenOCD Configuration
OPENOCD ?= openocd
OPENOCD_OPTIONS = -f interface/stlink.cfg -f target/stm32u5x.cfg

# NOTE: STM32U5 Metadata Stripping
# The standard ELF file produced by the compiler contains metadata sections
# (like .ARM.attributes) that sometimes point to System Memory addresses
# (e.g., 0x0BFA0000). Many flashing tools, including probe-rs, will attempt
# to write to these addresses, which are protected on the STM32U5, causing
# the flash process to crash.
#
# We use 'objcopy -j' to extract ONLY the sections that belong
# in the physical Flash memory (0x0C000000). This ensures a clean,
# safe deployment to the hardware.

# --- TARGET: app (Builds the merged binary and unified ELF) ---
.PHONY: app
app: $(KERNEL_ELF)
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

# --- TARGET: flash_ocd (Flashes ONLY the kernel and WIPES apps using OpenOCD) ---
.PHONY: flash_ocd
flash_ocd: $(KERNEL_ELF)
	@echo "--- Preparing Clean Kernel ELF ---"
	$(Q)arm-none-eabi-objcopy -j .text -j .ARM.exidx -j .storage -j .relocate \
	    $(KERNEL_ELF) kernel_clean.elf
	@echo "--- Flashing KERNEL and Erasing All via OpenOCD ---"
	# We perform a mass_erase before programming to ensure old apps are deleted.
	$(Q)$(OPENOCD) $(OPENOCD_OPTIONS) -c "init; halt; stm32l4x mass_erase 0; program kernel_clean.elf verify reset exit"

# --- TARGET: app_flash (Flashes KERNEL + BLINK APP using probe-rs) ---
.PHONY: app_flash
app_flash: app
	@echo "--- Preparing Clean Full ELF ---"
	# NOTE: ld -b binary names the section '.data', so we must keep that.
	$(Q)arm-none-eabi-objcopy -j .data full_merged.elf full_merged_clean.elf
	@echo "--- Flashing KERNEL + BLINK APP ---"
	$(Q)probe-rs download --chip STM32U545RETx --allow-erase-all full_merged_clean.elf
	$(Q)probe-rs reset --chip STM32U545RETx


# --- TARGET: app_flash_ocd (Flashes KERNEL + APP using OpenOCD) ---
.PHONY: app_flash_ocd
app_flash_ocd: app
	@echo "--- Preparing Clean Full ELF ---"
	$(Q)arm-none-eabi-objcopy -j .data full_merged.elf full_merged_clean.elf
	@echo "--- Flashing KERNEL + APP via OpenOCD ---"
	$(Q)$(OPENOCD) $(OPENOCD_OPTIONS) -c "program full_merged_clean.elf verify reset exit"

.PHONY: install
install: flash

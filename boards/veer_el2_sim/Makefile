# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2022.
# Copyright (c) 2024 Antmicro <www.antmicro.com>

# Makefile for building the tock kernel for the VeeR EL2 simulation platform

TARGET=riscv32imc-unknown-none-elf
PLATFORM=veer_el2_sim
GNU_OBJCOPY ?= riscv64-unknown-elf-objcopy

include ../Makefile.common

# 'verilog' format files used in Verilator testbenches
# Any RISC-V-compatible objdump with support for 'verilog' can be used as GNU_OBJCOPY.
# llvm-objcopy doesn't support 'verilog'.
%.hex: %.elf
	@if [ -z "$(shell which ${GNU_OBJCOPY})" ]; then \
		echo "Error: No ${GNU_OBJCOPY} in $(PATH)"; \
		exit 1; \
	fi
	$(Q)$(GNU_OBJCOPY) --output-target=verilog $< $@
	$(Q)$(SHA256SUM) $@

.PHONY: release-hex
release-hex:  $(TARGET_PATH)/release/$(PLATFORM).hex

.PHONY: debug-hex
debug-hex:  $(TARGET_PATH)/debug/$(PLATFORM).hex

VEER_EL2_SRC ?= $(TARGET_DIRECTORY)/cores-veer-el2

# Target to clone the VeeR EL2 repository and switch to a fixed version.
$(VEER_EL2_SRC): | $(TARGET_DIRECTORY)
	git clone https://github.com/chipsalliance/Cores-VeeR-EL2 $(VEER_EL2_SRC)
	git -C $(VEER_EL2_SRC) switch --detach da1042557

# Target to compile the Verilator simulation program.
.PHONY: sim-prepare
sim-prepare: | $(VEER_EL2_SRC)
	# ../Makefile.common disables built-in rules and variables by setting proper MAKEFLAGS.
	# Verilator relies on the built-ins, so we set 'MAKEFLAGS' to empty for this command.
	MAKEFLAGS='' RV_ROOT=$(TARGET_DIRECTORY)/cores-veer-el2 \
	CONF_PARAMS='-set build-axi4 -set user_mode=1 -set reset_vec=0x20000000' \
	make -C $(VEER_EL2_SRC)/tools verilator-build

# Target to execute simulation.
.PHONY: sim
sim: $(TARGET_PATH)/release/$(PLATFORM).hex sim-prepare
	cp $(TARGET_PATH)/release/$(PLATFORM).hex $(VEER_EL2_SRC)/program.hex
	cd $(VEER_EL2_SRC) && ./tools/obj_dir/Vtb_top

.PHONY: sim-clean
sim-clean:
	RV_ROOT=$(VEER_EL2_SRC) make -C $(VEER_EL2_SRC)/tools clean

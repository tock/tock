# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2026.

# Makefile for building the Tock kernel for the SHAKTI C-Class Verilator sim.

include ../Makefile.common

# --- Simulation image and run -------------------------------------------------
#
# The C-Class test SoC loads a single flat image, `code.mem`, into its main
# memory at RAM_BASE, and writes UART output to `app_log` in the working
# directory. The app region (`_sapps`) is inside that same memory, so the
# kernel and the one app TBF have to be combined into that one image: the
# kernel at RAM_BASE, the TBF spliced in at APP_BASE.
#
#   make APP=/path/to/app.tbf hex
#   make APP=/path/to/app.tbf SHAKTI_BIN=/path/to/c-class/bin sim
#
# `elf2hex` here means the Berkeley/fesvr-style tool, invoked as
#
#   elf2hex <bytes-per-row> <rows> <elf> [base]
#
# which is the form used throughout the SHAKTI C-Class documentation. Note that
# SiFive's elf2hex (github.com/sifive/elf2hex) takes --bit-width/--input
# instead and is *not* a drop-in substitute; set ELF2HEX accordingly.

ELF2HEX       ?= elf2hex
ELF2HEX_WIDTH ?= 8
ELF2HEX_ROWS  ?= 4194304
RAM_BASE      ?= 0x80000000
APP_BASE      ?= 0x80100000
SIM_TIMEOUT   ?= 250

# Directory holding the C-Class Verilator build: `out` plus `boot.MSB` and
# `boot.LSB`. Built out of the c-class repository, so it lives outside this tree.
SHAKTI_BIN    ?=

# The app to boot. This board loads exactly one process.
APP           ?=

SIM_DIR    = $(TARGET_PATH)/sim
KERNEL_ELF = $(TARGET_PATH)/release/$(PLATFORM).elf

.PHONY: hex
hex: $(SIM_DIR)/code.mem

$(SIM_DIR)/code.mem: $(KERNEL_ELF) $(APP) tools/splice_tbf.py
	@test -n "$(APP)" || { \
	  echo "Set APP=/path/to/app.tbf -- this board boots exactly one process."; \
	  exit 1; }
	@test -f "$(APP)" || { echo "APP not found: $(APP)"; exit 1; }
	@mkdir -p $(SIM_DIR)
	$(ELF2HEX) $(ELF2HEX_WIDTH) $(ELF2HEX_ROWS) $(KERNEL_ELF) $$(($(RAM_BASE))) > $@
	python3 tools/splice_tbf.py $@ $(APP) $(RAM_BASE) $(APP_BASE) --width $(ELF2HEX_WIDTH)

.PHONY: sim
sim: $(SIM_DIR)/code.mem
	@test -n "$(SHAKTI_BIN)" || { \
	  echo "Set SHAKTI_BIN=/path/to/c-class/bin (holding out, boot.MSB, boot.LSB)."; \
	  exit 1; }
	@test -x "$(SHAKTI_BIN)/out" || { echo "No executable $(SHAKTI_BIN)/out"; exit 1; }
	cd $(SIM_DIR) && \
	  ln -sf $(abspath $(SHAKTI_BIN))/out . && \
	  ln -sf $(abspath $(SHAKTI_BIN))/boot.MSB . && \
	  ln -sf $(abspath $(SHAKTI_BIN))/boot.LSB . && \
	  timeout $(SIM_TIMEOUT) ./out > sim.log 2>&1 || true
	@echo "--- $(SIM_DIR)/app_log ---"
	@cat $(SIM_DIR)/app_log 2>/dev/null || { echo "(no app_log; see sim.log)"; exit 1; }

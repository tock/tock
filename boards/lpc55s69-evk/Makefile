# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2025.

# Makefile for building the tock kernel for the LPC55S69-EVK platform
#

# Include common Makefile definitions (Tock build system)
include ../Makefile.common

# --- CONFIGURATION START ---

# Detect Windows to add .exe extension
ifdef OS
   EXT = .exe
else
   EXT =
endif

# Tool definition:
LINKSERVER ?= LinkServer$(EXT)

# Device name for LinkServer:
LS_DEVICE ?= LPC55S69

# Probe selection:
# Defaults to "#1" (the first connected probe).
# IMPORTANT: We use `\#` here so Make passes the literal hash to the shell command.
# Without the backslash, Make might interpret it as a comment start.
PROBE_ID ?= \#1

# --- CONFIGURATION END ---

# The specific chip identifier (Part Number).
CHIP=LPC55S69JBD100

# Default target: `make install` triggers the `flash` target (usually maps to `program`)
.PHONY: install
install: flash


# TARGET: flash
# Usage: `make flash`
# Builds the debug version of the kernel (handled by ../Makefile.common rules) and flashes it.
#
# LinkServer Command Breakdown:
#   flash       -> Enter flash programming mode
#   --probe ""  -> Selects the probe.
#   load        -> The specific action to write the binary to flash.
#   "$<"        -> Automatic variable for the prerequisite (the .elf file).
.PHONY: flash
flash: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/debug/$(PLATFORM).elf
	$(LINKSERVER) flash --probe "$(PROBE_ID)" $(LS_DEVICE) load "$<"

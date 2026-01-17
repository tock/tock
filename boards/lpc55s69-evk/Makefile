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

# Paths to the userspace application and the kernel binaries
APP=/home/pc1/Projects/libtock-c/examples/blink/build/cortex-m7/cortex-m7.tbf
KERNEL=$(TOCK_ROOT_DIRECTORY)/target/$(TARGET)/release/$(PLATFORM).elf
KERNEL_WITH_APP=$(TOCK_ROOT_DIRECTORY)/target/$(TARGET)/release/$(PLATFORM)-app.elf

# Default target: `make install` triggers the `flash` target (usually maps to `program`)
.PHONY: install
install: flash

# Legacy target for compatibility.
# `flash-debug` is just an alias for `debug`.
.PHONY: flash-debug
flash-debug: debug


# TARGET: debug
# Usage: `make debug`
# Builds the debug version of the kernel (handled by ../Makefile.common rules) and flashes it.
#
# LinkServer Command Breakdown:
#   flash       -> Enter flash programming mode
#   --probe ""  -> Selects the probe.
#   load        -> The specific action to write the binary to flash.
#   "$<"        -> Automatic variable for the prerequisite (the .elf file).
.PHONY: debug
debug: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/debug/$(PLATFORM).elf
	$(LINKSERVER) flash --probe "$(PROBE_ID)" $(LS_DEVICE) load "$<"

# TARGET: program
# Usage: `make program`
# Builds the release version, merges the userspace app, and flashes the result.
.PHONY: program
program: release
	arm-none-eabi-objcopy --set-section-flags .apps=LOAD,ALLOC $(KERNEL) $(KERNEL_WITH_APP)
	arm-none-eabi-objcopy --update-section .apps=$(APP) $(KERNEL_WITH_APP)
	$(LINKSERVER) flash --probe "$(PROBE_ID)" $(LS_DEVICE) load "$(KERNEL_WITH_APP)"

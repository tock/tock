# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2026.

# Makefile for the nRF5340 DK kernel in non-secure state, and its secure stage

include ../../../Makefile.common

PROBE_RS ?= probe-rs
SECURE_STAGE := $(TARGET_PATH)/release/nrf5340dk-ns-secure.elf

# Build the secure stage along with the kernel.
all: secure

.PHONY: secure
secure:
	$(MAKE) -C secure

.PHONY: install
install: flash

# Write the secure stage at 0x0 and the kernel after it over the on-board
# J-Link, then reset.
.PHONY: flash
flash: $(TARGET_PATH)/release/$(PLATFORM).elf secure
	$(PROBE_RS) download --chip nRF5340_xxAA_APPONLY --verify $(SECURE_STAGE)
	$(PROBE_RS) download --chip nRF5340_xxAA_APPONLY --verify $<
	$(PROBE_RS) reset --chip nRF5340_xxAA_APPONLY

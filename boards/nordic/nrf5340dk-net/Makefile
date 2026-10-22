# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2026.

# Makefile for building the tock kernel for the nRF5340 DK network core

include ../../Makefile.common

PROBE_RS ?= probe-rs

# Default target for installing the kernel.
.PHONY: install
install: flash

# Upload the kernel to the network core over the on-board J-Link, then reset
# through the application core, whose kernel releases the network core again.
.PHONY: flash
flash: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf
	$(PROBE_RS) download --chip nRF5340_xxAA --verify $<
	$(PROBE_RS) reset --chip nRF5340_xxAA_APPONLY

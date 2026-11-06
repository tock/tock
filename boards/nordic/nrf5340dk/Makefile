# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2026.

# Makefile for building the tock kernel for the nRF5340 development kit

include ../../Makefile.common

PROBE_RS ?= probe-rs

# Default target for installing the kernel.
.PHONY: install
install: flash

# Upload the kernel over the on-board J-Link. The app-only target keeps
# probe-rs from attaching to the network core, which may be locked or held in
# reset.
.PHONY: flash
flash: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf
	$(PROBE_RS) download --chip nRF5340_xxAA_APPONLY --verify $<
	$(PROBE_RS) reset --chip nRF5340_xxAA_APPONLY

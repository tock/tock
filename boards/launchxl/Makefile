# Makefile for building the tock kernel for the launchxl development kit

TOCK_ARCH=cortex-m4
TOCK_BOARD=launchxl
TARGET=thumbv7em-none-eabi
PLATFORM=launchxl

include ../Makefile.common

$(call check_defined, UNIFLASH_CLI_BASE, See launchxl README for details)

DSLITE=$(UNIFLASH_CLI_BASE)/ccs_base/DebugServer/bin/DSLite

flash: target/$(TARGET)/release/launchxl
	@echo "Flashing target/$(TARGET)/release/launchxl"
	@$(DSLITE) flash -c jtag/cc2652r1f.ccxml -e -f target/$(TARGET)/release/launchxl


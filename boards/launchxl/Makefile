# Makefile for building the tock kernel for the launchxl development kit

TOCK_ARCH=cortex-m4
TOCK_BOARD=launchxl
TARGET=thumbv7em-none-eabi
PLATFORM=launchxl

DSLITE=$(UNIFLASH_CLI_BASE)/ccs_base/DebugServer/bin/DSLite
PROG_BASE=0x47000

include ../Makefile.common

apps/%/build/$(TOCK_ARCH)/$(TOCK_ARCH).bin:
	@make -C apps/$*

flash: guard-UNIFLASH_CLI_BASE target/$(TARGET)/release/launchxl
	@echo "Flashing target/$(TARGET)/release/launchxl"
	@$(DSLITE) flash -c jtag/cc2652r1f.ccxml -e -f target/$(TARGET)/release/launchxl

flash-%: apps/%/build/$(TOCK_ARCH)/$(TOCK_ARCH).bin guard-DSLITE
	@$(DSLITE) flash -c jtag/cc2652r1f.ccxml -e -f $<,$(PROG_BASE)

guard-%:
	@ if [ "${${*}}" = "" ]; then \
		echo "Environment variable $* not set, see readme for more details."; \
		exit 1; \
	fi

# Makefile for building the tock kernel for the OpenTitan platform

TARGET=riscv32imc-unknown-none-elf
PLATFORM=opentitan

include ../Makefile.common

flash: $(MAKEFILE_PARENT_PATH)target/$(TARGET)/release/$(PLATFORM).bin
	$(OPENTITAN_TREE)/sw/host/spiflash/spiflash --input=$(MAKEFILE_PARENT_PATH)target/$(TARGET)/release/$(PLATFORM).bin

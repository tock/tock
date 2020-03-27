# Makefile for building the tock kernel for the OpenTitan platform

TARGET=riscv32imc-unknown-none-elf
PLATFORM=opentitan

include ../Makefile.common

flash: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).bin
	$(OPENTITAN_TREE)/sw/host/spiflash/spiflash --input=$(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).bin

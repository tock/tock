# Makefile for building the tock kernel for the nRF development kit

TOCK_ARCH=cortex-m4
TARGET=thumbv7em-none-eabi
PLATFORM=acd52832

include ../Makefile.common

TOCKLOADER=tockloader

# Where in the SAM4L flash to load the kernel with `tockloader`
KERNEL_ADDRESS=0x00000

# Upload programs over uart with tockloader
ifdef PORT
  TOCKLOADER_GENERAL_FLAGS += --port $(PORT)
endif

TOCKLOADER_JTAG_FLAGS = --jlink --board nrf52dk

# Upload the kernel over JTAG
.PHONY: flash
flash: target/$(TARGET)/release/$(PLATFORM).bin
	$(TOCKLOADER) $(TOCKLOADER_GENERAL_FLAGS) flash --address $(KERNEL_ADDRESS) $(TOCKLOADER_JTAG_FLAGS) $<

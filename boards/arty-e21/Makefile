# Makefile for building the tock kernel for the HiFive1 platform

TARGET=riscv32imac-unknown-none-elf
PLATFORM=arty-e21

include ../Makefile.common

TOCKLOADER=tockloader
KERNEL_ADDRESS=0x400000

TOCKLOADER_OPENOCD_FLAGS = --openocd --board arty

.PHONY: flash-tockloader
flash-tockloader: target/$(TARGET)/release/$(PLATFORM).bin
	$(TOCKLOADER) $(TOCKLOADER_GENERAL_FLAGS) flash --address $(KERNEL_ADDRESS) $(TOCKLOADER_OPENOCD_FLAGS) $<

flash: target/$(TARGET)/release/$(PLATFORM).bin
	openocd -f openocd/arty-openocd-digilent.cfg \
		-c "init; jtagspi_init 0 openocd/bscan_spi_xc7a35t.bit; jtagspi_program $< 0x400000; shutdown"

flash-e21: core/sifive_coreip_E21_FPGA_Evaluation_v19_02_rc0.mcs
	openocd -f openocd/arty-openocd-digilent.cfg \
		-c "init; jtagspi_init 0 openocd/bscan_spi_xc7a35t.bit; jtagspi_program $< 0x0; shutdown"


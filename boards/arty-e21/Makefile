# Makefile for building the tock kernel for the HiFive1 platform

TARGET=riscv32imac-unknown-none-elf
PLATFORM=arty-e21

# Need this to be different for now
RUSTFLAGS_FOR_CARGO_LINKING ?= -C link-arg=-Tlayout.ld -C linker=riscv32-unknown-elf-ld

TOOLCHAIN ?= /Users/bradjc/git/riscv-rust-toolchain/toolchain/bin/riscv32-unknown-elf

include ../Makefile.common

flash-jlink: target/$(TARGET)/release/$(PLATFORM).elf
	openocd -f arty-openocd-jlink.cfg \
		-c "flash protect 0 64 last off; program $<; resume 0x40400000; exit"


flash: target/$(TARGET)/release/$(PLATFORM).bin
	openocd -f arty-openocd-digilent.cfg \
		-c "init; jtagspi_init 0 bscan_spi_xc7a35t.bit; jtagspi_program $< 0x400000; shutdown"


flash-e21: sifive_coreip_E21_FPGA_Evaluation_v1p0_rc2.mcs
	openocd -f arty-openocd-digilent.cfg \
		-c "init; jtagspi_init 0 bscan_spi_xc7a35t.bit; jtagspi_program $< 0x0; shutdown"


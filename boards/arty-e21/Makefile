# Makefile for building the tock kernel for the HiFive1 platform

TARGET=riscv32imac-unknown-none-elf
PLATFORM=arty-e21

# Need this to be different for now
RUSTFLAGS_FOR_CARGO_LINKING ?= -C link-arg=-Tlayout.ld -C linker=riscv32-unknown-elf-ld

TOOLCHAIN ?= /Users/bradjc/git/riscv-rust-toolchain/toolchain/bin/riscv32-unknown-elf

include ../Makefile.common

flash: target/$(TARGET)/release/$(PLATFORM).elf
	openocd -f arty-openocd.cfg \
		-c "flash protect 0 64 last off; program $<; resume 0x40400000; exit"


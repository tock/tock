# Makefile for building the tock kernel for the HiFive1 platform

TARGET=riscv32imac-unknown-none-elf
PLATFORM=hifive1

include ../Makefile.common

flash: target/$(TARGET)/release/$(PLATFORM).elf
	openocd \
		-c "source [find board/sifive-hifive1.cfg]; flash protect 0 64 last off; program $<; resume 0x20400000; exit"

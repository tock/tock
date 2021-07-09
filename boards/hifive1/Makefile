# Makefile for building the tock kernel for the HiFive1 platform

TARGET=riscv32imac-unknown-none-elf
PLATFORM=hifive1
QEMU ?= qemu-system-riscv32

include ../Makefile.common

# Default target for installing the kernel.
.PHONY: install
install: flash

flash: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf
	openocd \
		-c "source [find board/sifive-hifive1-revb.cfg]; program $<; resume 0x20000000; exit"

qemu: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf
	$(QEMU) -M sifive_e,revb=true -kernel $^  -nographic

qemu-app: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf
	$(QEMU) -M sifive_e,revb=true -kernel $^ -device loader,file=$(APP),addr=0x20040000 -nographic


TOCKLOADER=tockloader
TOCKLOADER_JTAG_FLAGS = --jlink --board hifive1b
KERNEL_ADDRESS = 0x20010000

# upload kernel over JTAG
.PHONY: flash
flash-jlink: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).bin
	$(TOCKLOADER) $(TOCKLOADER_GENERAL_FLAGS) flash --address $(KERNEL_ADDRESS) $(TOCKLOADER_JTAG_FLAGS) $<

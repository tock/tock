# Makefile for building the tock kernel for the OpenTitan platform

TARGET=riscv32imc-unknown-none-elf
PLATFORM=opentitan

include ../Makefile.common

qemu: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf
	qemu-system-riscv32 -M opentitan -kernel $^ -bios $(OPENTITAN_BOOT_ROM) -nographic -serial mon:stdio

qemu-app: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf
	qemu-system-riscv32 -M opentitan -kernel $^ -bios $(OPENTITAN_BOOT_ROM) -device loader,file=$(APP),addr=0x20030000 -nographic -serial mon:stdio

flash: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).bin
	$(OPENTITAN_TREE)/sw/host/spiflash/spiflash --input=$(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).bin

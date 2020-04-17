# Makefile for building the tock kernel for the OpenTitan platform

TARGET=riscv32imc-unknown-none-elf
PLATFORM=opentitan

include ../Makefile.common

# Passing a feature to cargo through `FEATURES` environment variable.
ifneq ($(FEATURES),)
	CARGO_FLAGS += --features=$(FEATURES)
endif

qemu: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf
	$(call check_defined, OPENTITAN_BOOT_ROM)
	qemu-system-riscv32 -M opentitan -kernel $^ -bios $(OPENTITAN_BOOT_ROM) -nographic -serial mon:stdio

qemu-app: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf
	$(call check_defined, OPENTITAN_BOOT_ROM)
	qemu-system-riscv32 -M opentitan -kernel $^ -bios $(OPENTITAN_BOOT_ROM) -device loader,file=$(APP),addr=0x20030000 -nographic -serial mon:stdio

flash: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).bin
	$(OPENTITAN_TREE)/build-out/sw/host/spiflash/spiflash --input=$(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).bin

flash-app: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf
	riscv32-none-elf-objcopy --update-section .apps=$(APP) $^ $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM)-app.elf
	$(OBJCOPY) --output-target=binary $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM)-app.elf $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM)-app.bin
	$(OPENTITAN_TREE)/build-out/sw/host/spiflash/spiflash --input=$(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM)-app.bin

# Makefile for building the tock kernel for the OpenTitan platform

DEFAULT_BOARD_CONFIGURATION=fpga_cw310
TARGET=riscv32imc-unknown-none-elf
PLATFORM=earlgrey-cw310
FLASHID=--dev-id="0403:6010"
RISC_PREFIX ?= riscv64-elf
QEMU ?= ../../../tools/qemu/build/qemu-system-riscv32


include ../../Makefile.common

# Pass OpenTitan board configuration option in `BOARD_CONFIGURATION` through
# Cargo `--features`. Please see `Cargo.toml` for available options.
ifneq ($(BOARD_CONFIGURATION),)
	CARGO_FLAGS += --features=$(BOARD_CONFIGURATION)
else
	CARGO_FLAGS += --features=$(DEFAULT_BOARD_CONFIGURATION)
endif

# Default target for installing the kernel.
.PHONY: install
install: flash

qemu: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf
	$(call check_defined, OPENTITAN_BOOT_ROM)
	$(QEMU) -M opentitan -kernel $^ -bios $(OPENTITAN_BOOT_ROM) -nographic -serial mon:stdio


qemu-gdb: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf
	$(call check_defined, OPENTITAN_BOOT_ROM)
	$(QEMU) -s -S -M opentitan -kernel $^ -bios $(OPENTITAN_BOOT_ROM) -nographic -serial mon:stdio

qemu-app: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf
	$(call check_defined, OPENTITAN_BOOT_ROM)
	$(QEMU) -M opentitan -kernel $^ -bios $(OPENTITAN_BOOT_ROM) -device loader,file=$(APP),addr=0x20030000 -nographic -serial mon:stdio

qemu-app-gdb : $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf
	$(call check_defined, OPENTITAN_BOOT_ROM)
	$(QEMU) -s -S -M opentitan -kernel $^ -bios $(OPENTITAN_BOOT_ROM) -device loader,file=$(APP),addr=0x20030000 -nographic -serial mon:stdio

flash: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).bin
	$(OPENTITAN_TREE)/util/fpga/cw310_loader.py --firmware $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).bin

flash-app: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf
	$(RISC_PREFIX)-objcopy --update-section .apps=$(APP) $^ $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM)-app.elf
	$(RISC_PREFIX)-objcopy --output-target=binary $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM)-app.elf $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM)-app.bin
	$(OPENTITAN_TREE)/util/fpga/cw310_loader.py --firmware $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM)-app.bin

test:
ifneq ($(OPENTITAN_TREE),)
	$(error "Running on QEMU, use test-hardware to run on hardware")
endif
	mkdir -p $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/deps/
	$(Q)cp test_layout.ld $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/deps/layout.ld
	$(Q)cp ../../kernel_layout.ld $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/
	$(Q)RUSTFLAGS="$(RUSTC_FLAGS_TOCK)" $(CARGO) test $(CARGO_FLAGS_TOCK_NO_BUILD_STD) $(NO_RUN) --bin $(PLATFORM) --release

test-hardware:
ifeq ($(OPENTITAN_TREE),)
	$(error "Please ensure that OPENTITAN_TREE is set")
endif
	mkdir -p $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/deps/
	$(Q)cp test_layout.ld $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/deps/layout.ld
	$(Q)cp ../../kernel_layout.ld $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/
	$(Q)RUSTFLAGS="$(RUSTC_FLAGS_TOCK)" $(CARGO) test $(CARGO_FLAGS_TOCK_NO_BUILD_STD) $(NO_RUN) --bin $(PLATFORM) --release --features=hardware_tests

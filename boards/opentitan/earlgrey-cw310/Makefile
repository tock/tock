# Makefile for building the tock kernel for the OpenTitan platform

DEFAULT_BOARD_CONFIGURATION=fpga_cw310
TARGET=riscv32imc-unknown-none-elf
PLATFORM=earlgrey-cw310
FLASHID=--dev-id="0403:6010"
RISC_PREFIX ?= riscv64-elf
QEMU ?= ../../../tools/qemu/build/qemu-system-riscv32
ELF2TAB ?= elf2tab


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

rsa.tbf:
	$(OPENTITAN_TREE)/bazelisk.sh build //sw/otbn/crypto:rsa
	$(eval ELF := $(shell $(OPENTITAN_TREE)/bazelisk.sh cquery //sw/otbn/crypto:rsa --output starlark --starlark:expr="' '.join([f.path for f in target.files.to_list() if f.basename.endswith('.elf')])"))
	$(ELF2TAB) \
		--verbose \
		--package-name "otbn-rsa" \
		--kernel-minor 0 --kernel-major 2 \
		--app-heap 0 --kernel-heap 0 --stack 0 \
		--output-file rsa.tab \
		$(OPENTITAN_TREE)/$(ELF)
	tar -xf rsa.tab rsa.elf.tbf && mv rsa.elf.tbf rsa.tbf && touch rsa.tbf

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
	$(OPENTITAN_TREE)/bazelisk.sh run //sw/host/opentitantool -- \
		--rcfile= \
		--interface=cw310 \
		--conf=$(OPENTITAN_TREE)/sw/host/opentitantool/config/opentitan_cw310.json \
		bootstrap --protocol=eeprom $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).bin


flash-app: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf
	$(RISC_PREFIX)-objcopy --update-section .apps=$(APP) $^ $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM)-app.elf
	$(RISC_PREFIX)-objcopy --output-target=binary $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM)-app.elf $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM)-app.bin
	$(OPENTITAN_TREE)/util/fpga/cw310_loader.py --firmware $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM)-app.bin

verilator: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).bin
	$(call check_defined, OPENTITAN_TREE)
	srec_cat $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).bin \
		--binary --offset 0 --byte-swap 8 --fill 0xff \
		-within $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).bin \
		-binary -range-pad 8 --output binary.64.vmem --vmem 64
	$(OPENTITAN_TREE)/build/lowrisc_dv_chip_verilator_sim_0.1/sim-verilator/Vchip_sim_tb \
		--meminit=rom,$(OPENTITAN_TREE)/build-out/sw/device/lib/testing/test_rom/test_rom_sim_verilator.scr.39.vmem \
		--meminit=flash,./binary.64.vmem \
		--meminit=otp,$(OPENTITAN_TREE)/build-out/sw/device/otp_img/otp_img_sim_verilator.vmem

test:
ifneq ($(OPENTITAN_TREE),)
	$(error "Running on QEMU, use test-hardware to run on hardware")
endif
	mkdir -p $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/deps/
	$(Q)cp test_layout.ld $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/deps/layout.ld
	$(Q)cp ../../kernel_layout.ld $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/
	$(Q)RUSTFLAGS="$(RUSTC_FLAGS_TOCK)" $(CARGO) test $(CARGO_FLAGS_TOCK_NO_BUILD_STD) $(NO_RUN) --bin $(PLATFORM) --release
# Test layout should be removed so that following apps (non-test) load the correct layout.
	$(Q)rm $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/deps/layout.ld

test-hardware:
ifeq ($(OPENTITAN_TREE),)
	$(error "Please ensure that OPENTITAN_TREE is set")
endif
	mkdir -p $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/deps/
	$(Q)cp test_layout.ld $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/deps/layout.ld
	$(Q)cp ../../kernel_layout.ld $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/
	$(Q)RUSTFLAGS="$(RUSTC_FLAGS_TOCK)" $(CARGO) test $(CARGO_FLAGS_TOCK_NO_BUILD_STD) $(NO_RUN) --lib --release --features=hardware_tests
	$(Q)rm $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/deps/layout.ld

test-verilator:
	$(call check_defined, OPENTITAN_TREE)

	mkdir -p $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/deps/
	$(Q)cp test_layout.ld $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/deps/layout.ld
	$(Q)cp ../../kernel_layout.ld $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/
	$(Q)RUSTFLAGS="$(RUSTC_FLAGS_TOCK)" VERILATOR="yes" OBJCOPY=${OBJCOPY} $(CARGO) test $(CARGO_FLAGS_TOCK_NO_BUILD_STD) $(NO_RUN) --bin $(PLATFORM) --release --features=hardware_tests --features sim_verilator
	$(Q)rm $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/deps/layout.ld

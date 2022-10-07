# Makefile for building the tock kernel for the OpenTitan platform

DEFAULT_BOARD_CONFIGURATION=fpga_cw310
TARGET=riscv32imc-unknown-none-elf
PLATFORM=earlgrey-cw310
FLASHID=--dev-id="0403:6010"
RISC_PREFIX ?= riscv64-linux-gnu
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
	$(QEMU) -M opentitan -kernel $^ -nographic -serial mon:stdio

qemu-gdb: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf
	$(QEMU) -s -S -M opentitan -kernel $^ -nographic -serial mon:stdio

qemu-app: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf
	$(QEMU) -M opentitan -kernel $^ -device loader,file=$(APP),addr=0x20030000 -nographic -serial mon:stdio

qemu-app-gdb : $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf
	$(QEMU) -s -S -M opentitan -kernel $^ -device loader,file=$(APP),addr=0x20030000 -nographic -serial mon:stdio

flash: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).bin
	$(OPENTITAN_TREE)/util/fpga/cw310_loader.py --firmware $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).bin

flash-app: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf
	$(RISC_PREFIX)-objcopy --update-section .apps=$(APP) $^ $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM)-app.elf
	$(RISC_PREFIX)-objcopy --output-target=binary $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM)-app.elf $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM)-app.bin
	$(OPENTITAN_TREE)/util/fpga/cw310_loader.py --firmware $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM)-app.bin

verilator: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf
	$(call check_defined, OPENTITAN_TREE)
#	Make a copy so we dont modify the original elf when linkng apps
	$(RISC_PREFIX)-objcopy $^ $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM)_verilator.elf
ifneq ($(APP),)
		$(info [CW-130: Verilator]: Linking App)
		$(RISC_PREFIX)-objcopy --update-section .apps=$(APP) $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM)_verilator.elf $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM)_verilator.elf
endif
	$(info [CW-130: Verilator]: Starting)
	$(RISC_PREFIX)-objcopy --output-target=binary $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM)_verilator.elf $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM)_verilator.bin
	srec_cat $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM)_verilator.bin \
		--binary --offset 0 --byte-swap 8 --fill 0xff \
		-within $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM)_verilator.bin\
		-binary -range-pad 8 --output binary.64.vmem --vmem 64
	$(OPENTITAN_TREE)/bazel-out/k8-fastbuild/bin/hw/build.verilator_real/sim-verilator/Vchip_sim_tb \
		--meminit=rom,$(OPENTITAN_TREE)/bazel-out/k8-fastbuild-ST-97f470ee3b14/bin/sw/device/lib/testing/test_rom/test_rom_sim_verilator.scr.39.vmem \
		--meminit=flash,./binary.64.vmem \
		--meminit=otp,$(OPENTITAN_TREE)/bazel-out/k8-fastbuild/bin/hw/ip/otp_ctrl/data/rma_image_verilator.vmem

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
	$(Q)RUSTFLAGS="$(RUSTC_FLAGS_TOCK)" $(CARGO) test $(CARGO_FLAGS_TOCK_NO_BUILD_STD) $(NO_RUN) --bin $(PLATFORM) --release --features=hardware_tests
	$(Q)rm $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/deps/layout.ld

test-verilator:
	$(call check_defined, OPENTITAN_TREE)

	mkdir -p $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/deps/
	$(Q)cp test_layout.ld $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/deps/layout.ld
	$(Q)cp ../../kernel_layout.ld $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/
	$(Q)RUSTFLAGS="$(RUSTC_FLAGS_TOCK)" VERILATOR="yes" OBJCOPY=${OBJCOPY} $(CARGO) test $(CARGO_FLAGS_TOCK_NO_BUILD_STD) $(NO_RUN) --bin $(PLATFORM) --release --features=hardware_tests --features sim_verilator
	$(Q)rm $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/deps/layout.ld

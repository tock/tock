# Makefile for building the tock kernel for the OpenTitan platform

DEFAULT_BOARD_CONFIGURATION=fpga_cw310
TARGET=riscv32imc-unknown-none-elf
PLATFORM=earlgrey-cw310
FLASHID=--dev-id="0403:6010"
RISC_PREFIX ?= riscv64-linux-gnu
QEMU ?= ../../../tools/qemu/build/qemu-system-riscv32
# Override for the entry point
# This offset is calculated (from the linker file) as:
# ORIGIN(rom) + size_manifest = 0x20000000 + 0x400
QEMU_ENTRY_POINT=0x20000400
# Latest supported commmit of OpenTitan
OPENTITAN_SUPPORTED_SHA := d072ac505f82152678d6e04be95c72b728a347b8


include ../../Makefile.common

# Pass OpenTitan board configuration option in `BOARD_CONFIGURATION` through
# Cargo `--features`. Please see `Cargo.toml` for available options.
ifneq ($(BOARD_CONFIGURATION),)
	CARGO_FLAGS += --features=$(BOARD_CONFIGURATION)
else
	CARGO_FLAGS += --features=$(DEFAULT_BOARD_CONFIGURATION)
endif

.PHONY: ot-check
OPENTITAN_ACTUAL_SHA := $(shell cd $(OPENTITAN_TREE); git show --pretty=format:"%H" --no-patch)
# TODO: Theres mix and match of tab/space indenting in the block below,
# should be changed to using `RECIPEPREFIX` make syntax when CI supports
# make version > 3.81 (macos)
ot-check:
ifeq ($(OPENTITAN_TREE),)
	$(error "Please ensure that OPENTITAN_TREE is set")
endif
ifneq ($(OPENTITAN_ACTUAL_SHA), $(OPENTITAN_SUPPORTED_SHA))
    ifeq ($(SKIP_OT_VERSION_CHECK), yes)
		$(warning Skipping trivial version check)
    else
		$(error Please ensure to build the correct version \
		of OpenTitan, latest supported is <$(OPENTITAN_SUPPORTED_SHA)>, \
		you are on <$(OPENTITAN_ACTUAL_SHA)>)
    endif
endif


# Default target for installing the kernel.
.PHONY: install
install: flash

qemu: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf
	$(QEMU) -M opentitan -kernel $^ -nographic -serial mon:stdio -global driver=riscv.lowrisc.ibex.soc,property=resetvec,value=${QEMU_ENTRY_POINT}

qemu-gdb: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf
	$(QEMU) -s -S -M opentitan -kernel $^ -nographic -serial mon:stdio -global driver=riscv.lowrisc.ibex.soc,property=resetvec,value=${QEMU_ENTRY_POINT}

qemu-app: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf
	$(QEMU) -M opentitan -kernel $^ -device loader,file=$(APP),addr=0x20030000 -nographic -serial mon:stdio -global driver=riscv.lowrisc.ibex.soc,property=resetvec,value=${QEMU_ENTRY_POINT}

qemu-app-gdb : $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf
	$(QEMU) -s -S -M opentitan -kernel $^ -device loader,file=$(APP),addr=0x20030000 -nographic -serial mon:stdio -global driver=riscv.lowrisc.ibex.soc,property=resetvec,value=${QEMU_ENTRY_POINT}

flash: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).bin
	$(OPENTITAN_TREE)/bazel-bin/sw/host/opentitantool/opentitantool.runfiles/lowrisc_opentitan/sw/host/opentitantool/opentitantool --interface=cw310 bootstrap $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).bin

flash-app: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf
	$(RISC_PREFIX)-objcopy --update-section .apps=$(APP) $^ $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM)-app.elf
	$(RISC_PREFIX)-objcopy --output-target=binary $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM)-app.elf $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM)-app.bin
	$(OPENTITAN_TREE)/bazel-bin/sw/host/opentitantool/opentitantool.runfiles/lowrisc_opentitan/sw/host/opentitantool/opentitantool --interface=cw310 bootstrap $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM)-app.bin

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
		--meminit=rom,$(OPENTITAN_TREE)/bazel-out/k8-fastbuild-ST-2cc462681f62/bin/sw/device/lib/testing/test_rom/test_rom_sim_verilator.39.scr.vmem \
		--meminit=flash,./binary.64.vmem \
		--meminit=otp,$(OPENTITAN_TREE)/bazel-out/k8-fastbuild/bin/hw/ip/otp_ctrl/data/img_rma.24.vmem

test:
ifneq ($(OPENTITAN_TREE),)
	$(error "Running on QEMU, use test-hardware to run on hardware")
endif
	mkdir -p $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/deps/
	$(Q)cp test_layout.ld $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/deps/layout.ld
	$(Q)cp ../../kernel_layout.ld $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/
	$(Q)RUSTFLAGS="$(RUSTC_FLAGS_TOCK)" QEMU_ENTRY_POINT=${QEMU_ENTRY_POINT} $(CARGO) test $(CARGO_FLAGS_TOCK_NO_BUILD_STD)  $(NO_RUN) --bin $(PLATFORM) --release
# 	Test layout should be removed so that following apps (non-test) load the correct layout.
	$(Q)rm $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/deps/layout.ld

test-hardware: ot-check
	mkdir -p $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/deps/
	$(Q)cp test_layout.ld $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/deps/layout.ld
	$(Q)cp ../../kernel_layout.ld $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/
	$(Q)RUSTFLAGS="$(RUSTC_FLAGS_TOCK)" OBJCOPY=${OBJCOPY} TOCK_ROOT_DIRECTORY=${TOCK_ROOT_DIRECTORY} TARGET=${TARGET} $(CARGO) test $(CARGO_FLAGS_TOCK_NO_BUILD_STD) $(NO_RUN) --bin $(PLATFORM) --release --features=hardware_tests

test-verilator: ot-check
	mkdir -p $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/deps/
	$(Q)cp test_layout.ld $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/deps/layout.ld
	$(Q)cp ../../kernel_layout.ld $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/
	$(Q)RUSTFLAGS="$(RUSTC_FLAGS_TOCK)" VERILATOR="yes" OBJCOPY=${OBJCOPY} TOCK_ROOT_DIRECTORY=${TOCK_ROOT_DIRECTORY} TARGET=${TARGET} $(CARGO) test $(CARGO_FLAGS_TOCK_NO_BUILD_STD) $(NO_RUN) --bin $(PLATFORM) --release --features=hardware_tests --features sim_verilator

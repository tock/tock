# Makefile for building the Tock kernel for the qemu-system-riscv32 `virt`
# platform / machine type.

TARGET=riscv32imac-unknown-none-elf
PLATFORM=qemu_rv32_virt

include ../Makefile.common

WORKING_QEMU_VERSION=7.0.0

# Run the kernel inside a qemu-riscv32-system "virt" machine type simulation
#
# Peripherals attached by default:
# - 16550 UART (attached to stdio by default)
# - VirtIO EntropySource (default backend /dev/random)
#
# By default a VirtIO NetworkCard is _not_ attached, since creating a TAP device
# on the host will require root or further system configuration. The
# configuration options to enable the network card are included as comments.
#
# Requires that a qemu-riscv32-system binary is in the user's PATH. The tested &
# verified QEMU version is printed along with the one used. No actual version
# check is performed given the simulation might work with different version,
# though should at leat work on the tested one.
.PHONY: run
run: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).bin
	@echo
	@echo -e "Running $$(qemu-system-riscv32 --version | head -n1)" \
	  "(tested: $(WORKING_QEMU_VERSION)) with\n  - kernel $^"
	@echo "To exit type C-a x"
	@echo
	qemu-system-riscv32 \
	  -machine virt \
	  -bios $^ \
	  -global virtio-mmio.force-legacy=false \
	  -device virtio-rng-device \
	  -nographic
	  @# attaching a TAP network device requires proper permissions
	  @# to create a tuntap device or access to an existing device
	  @# -netdev tap,id=n0,script=no,downscript=no
	  @# -device virtio-net-device,netdev=n0

# Same as `run`, but load an application specified by $(APP) into the respective
# memory location.
.PHONY: run-app
run-app: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf
	@echo
	@echo -e "Running $$(qemu-system-riscv32 --version | head -n1)" \
	  "(tested: $(WORKING_QEMU_VERSION))"\
	  "with\n  - kernel $^\n  - app $(APP)"
	@echo "To exit type C-a x"
	@echo
	qemu-system-riscv32 \
	  -machine virt \
	  -bios $^ \
	  -global virtio-mmio.force-legacy=false \
	  -device virtio-rng-device \
	  -device loader,file=$(APP),addr=0x80100000 \
	  -nographic

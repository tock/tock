# Makefile for building the Tock kernel for the qemu-system-riscv32 `virt`
# platform / machine type.

TARGET   = riscv32imac-unknown-none-elf
PLATFORM = qemu_rv32_virt

include ../Makefile.common

QEMU_CMD             := qemu-system-riscv32
WORKING_QEMU_VERSION := 7.0.0

# Whether a VirtIO network device shall be attached to the QEMU
# machine, and which backend should be used. The following options are
# available:
#
# - NETDEV: SLIRP
#
#   Use the QEMU userspace slirp network backend. This causes QEMU to
#   behave as a NAT-router and gateway to the VM, transparently
#   routing any outgoing traffic through the host's userspace network
#   sockets. This option also accepts an optional NETDEV_SLIRP_ARGS
#   which is appended to the provided string.
#
# - NETDEV: TAP
#
#   Creates a TAP network interface to act as a layer-2 Ethernet
#   connection between the guest interface and the host. Must have the
#   proper permissions to let QEMU create the tap interface on the
#   host. Use SUDO-TAP instead to run QEMU through `sudo`.
NETDEV            ?= NONE
ifneq ($(NETDEV_SLIRP_ARGS),)
  NETDEV_SLIRP_ARGS := ,$(NETDEV_SLIRP_ARGS)
else
  NETDEV_SLIRP_ARGS :=
endif

ifeq ($(NETDEV),NONE)
  QEMU_NETDEV_CMDLINE = ""
else ifeq ($(NETDEV),SLIRP)
  QEMU_NETDEV_CMDLINE = \
    -netdev user,id=n0,net=192.168.1.0/24,dhcpstart=192.168.1.255$(NETDEV_SLIRP_ARGS) \
    -device virtio-net-device,netdev=n0
else ifneq (,$(filter $(NETDEV),TAP SUDO-TAP))
  QEMU_NETDEV_CMDLINE = \
    -netdev tap,id=n0,script=no,downscript=no \
    -device virtio-net-device,netdev=n0
  ifeq ($(NETDEV),SUDO-TAP)
    QEMU_CMD := sudo $(QEMU_CMD)
  endif
else
  $(error Invalid argument provided for variable NETDEV)
endif

# Peripherals attached by default:
# - 16550 UART (attached to stdio by default)
# - VirtIO EntropySource (default backend /dev/random)
QEMU_BASE_CMDLINE := \
  $(QEMU_CMD) \
    -machine virt \
    -semihosting \
    -global virtio-mmio.force-legacy=false \
    -device virtio-rng-device \
    $(QEMU_NETDEV_CMDLINE) \
    -nographic

# Some helpful instructions & information. Requires that a qemu-riscv32-system
# binary is in the user's PATH. The tested & verified QEMU version is printed
# along with the one used. No actual version check is performed given the
# simulation might work with different version, though should at leat work on
# the tested one.
.PHONY: qemu-usage-instructions
qemu-usage-instructions:
	@echo
	@echo -e "Running $$(qemu-system-riscv32 --version | head -n1)" \
	  "(tested: $(WORKING_QEMU_VERSION)) with\n" \
          " - kernel $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf"
	@test "\"$(APP)\" != \"\"" && echo -e "  - app $(APP)"
	@echo "To exit type C-a x"
	@echo

# Run the kernel inside a qemu-riscv32-system "virt" machine type simulation
.PHONY: run
run: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf qemu-usage-instructions
	$(QEMU_BASE_CMDLINE) \
	  -bios $<

# Same as `run`, but load an application specified by $(APP) into the respective
# memory location.
.PHONY: run-app
run-app: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf qemu-usage-instructions
	$(QEMU_BASE_CMDLINE) \
	  -bios $< \
	  -device loader,file=$(APP),addr=0x80100000

# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2022.

# Makefile for building the Tock kernel for the qemu-system-riscv32 `virt`
# platform / machine type.

include ../Makefile.common

QEMU_CMD              := qemu-system-riscv32
WORKING_QEMU_VERSIONS := 8.2.7, 9.1.3, 9.2.3, 10.0.2
BROKEN_QEMU_VERSIONS  := <= 8.1.5

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
#   To forward TCP port 1234 on the emulated Tock device (having IP
#   192.168.1.50) to the host port 5678, set the following variable:
#
#       NETDEV_SLIRP_ARGS=hostfwd=tcp::5678-192.168.1.50:1234
#
# - NETDEV: TAP
#
#   Creates a TAP network interface to act as a layer-2 Ethernet
#   connection between the guest interface and the host. Must have the
#   proper permissions to let QEMU create the tap interface on the
#   host. Use SUDO-TAP instead to run QEMU through `sudo`.
NETDEV            ?= NONE
ifneq ($(NETDEV_SLIRP_ARGS),)
  NETDEV_SLIRP_ARGS_INT := ,$(NETDEV_SLIRP_ARGS)
else
  NETDEV_SLIRP_ARGS_INT :=
endif

ifeq ($(NETDEV),NONE)
  QEMU_NETDEV_CMDLINE =
else ifeq ($(NETDEV),SLIRP)
  QEMU_NETDEV_CMDLINE = \
    -netdev user,id=n0,net=192.168.1.0/24,dhcpstart=192.168.1.255$(NETDEV_SLIRP_ARGS_INT) \
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
    -global driver=riscv-cpu,property=smepmp,value=true \
    -global virtio-mmio.force-legacy=false \
    -device virtio-rng-device \
    -device virtio-keyboard-device \
    $(QEMU_NETDEV_CMDLINE)

# Run the kernel inside a qemu-riscv32-system "virt" machine type simulation
.PHONY: run
run: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf
	@echo
	@echo -e "Running $$($(QEMU_CMD) --version | head -n1)"\
	  "(tested: $(WORKING_QEMU_VERSIONS); known broken: $(BROKEN_QEMU_VERSIONS)) with\n"\
          " - kernel $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf"
	@echo "To exit type C-a x"
	@echo
	$(QEMU_BASE_CMDLINE) \
	  -bios $< \
	  -nographic

# Same as `run`, but with a `virtio-gpu` video device attached, showing the
# screen output in a QEMU window
.PHONY: run-gui
run-gui: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf
	@echo
	@echo -e "Running $$($(QEMU_CMD) --version | head -n1)"\
	  "(tested: $(WORKING_QEMU_VERSIONS); known broken: $(BROKEN_QEMU_VERSIONS)) with\n"\
          " - kernel $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf"
	@echo "To exit type C-a x"
	@echo
	$(QEMU_BASE_CMDLINE) \
	  -bios $< \
	  -device virtio-gpu-device \
	  -serial stdio

# Same as `run`, but load an application specified by $(APP) into the respective
# memory location.
.PHONY: run-app
run-app: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf
	@echo
	@echo -e "Running $$($(QEMU_CMD) --version | head -n1)"\
	  "(tested: $(WORKING_QEMU_VERSIONS); known broken: $(BROKEN_QEMU_VERSIONS)) with\n"\
          " - kernel $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf\n"\
	  " - app $(APP)"
	@echo "To exit type C-a x"
	@echo
	$(QEMU_BASE_CMDLINE) \
	  -bios $< \
	  -device loader,file=$(APP),addr=0x80100000 \
	  -nographic

# Same as `run-app`, but with a `virtio-gpu` video device attached, showing the
# screen output in a QEMU window
.PHONY: run-app-gui
run-app-gui: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf
	@echo
	@echo -e "Running $$($(QEMU_CMD) --version | head -n1)"\
	  "(tested: $(WORKING_QEMU_VERSIONS); known broken: $(BROKEN_QEMU_VERSIONS)) with\n"\
          " - kernel $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).elf\n"\
	  " - app $(APP)"
	@echo "To exit type C-a x"
	@echo
	$(QEMU_BASE_CMDLINE) \
	  -bios $< \
	  -device loader,file=$(APP),addr=0x80100000 \
	  -device virtio-gpu-device \
	  -serial stdio

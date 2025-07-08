QEMU RISC-V 32 bit `virt` Platform
==================================

This board crate targets the QEMU RISC-V 32 bit `virt` platform. It
can utilize paravirtualized peripherals through the VirtIO
transport. Currently supported periphals of this board are:

- the primary 16550-compatible UART
- VirtIO-based network adapters
- VirtIO-based random number generators

While this target does not feature many peripherals for now, it represents a
stable QEMU target for using Tock in a virtualized RISC-V environment. This can
be useful for CI and other purposes. In the future, this target can be extended
to support VirtIO peripherals.

Starting from at least QEMU v7.0.0 up to and including v7.1.0, QEMU cotained a
bug which caused spurious memory access faults raised by the emulated Physical
Memory Protection (PMP), part of the emulated RISC-V CPU core. Therefore, **this
board requires at least QEMU v7.2.0** to function properly. Symptomps of the
aforementioned bug are crashes of userspace processes with a memory-access fault
reported by the kernel.

Running QEMU
------------

To run the board in QEMU, `qemu-system-riscv32` must be started with the
`-machine virt` argument. The Tock kernel expects to be loaded as the BIOS by
passing `-bios $TOCK_KERNEL.bin`, such that it runs in RISC-V machine mode and
has full control over the virtual board. `-nographic` can be used to suppress
QEMU's graphical interface.

The [`Makefile`] further contains two targets for running this board's kernel in
QEMU standalone, or with a single app. These can be executed through the
**`run`** and **`run-app`** targets, respectively.

- **`run`**: Start Tock on an emulated QEMU board without an app:

  ```
  tock/boards/qemu_rv32_virt $ make run
      Finished release [optimized + debuginfo] target(s) in 0.05s
     text    data     bss     dec     hex filename
    64880      12   11248   76140   1296c tock/target/riscv32imac-unknown-none-elf/release/qemu_rv32_virt
  a9de4df9486d724e6bf6a3423af669903dfd2bd1fd65c1dd867ddf9d7bcbec9b  tock/target/riscv32imac-unknown-none-elf/release/qemu_rv32_virt.bin

  Running QEMU emulator version 7.0.0 (tested: 7.0.0) with
    - kernel tock/target/riscv32imac-unknown-none-elf/release/qemu_rv32_virt.bin
  To exit type C-a x

  qemu-system-riscv32 \
    -machine virt \
    -bios tock/target/riscv32imac-unknown-none-elf/release/qemu_rv32_virt.bin \
    -global virtio-mmio.force-legacy=false \
    -device virtio-rng-device \
    -nographic
  QEMU RISC-V 32-bit "virt" machine, initialization complete.
  Entering main loop.
  ```

- **`run-app`**: Start Tock on an emulated QEMU board with an app:

  ```
  tock/boards/qemu_rv32_virt $ make run-app APP=$PATH_TO_APP.tbf
  ```

Through the **`NETDEV`** environment variable, QEMU can be instructed to attach
a VirtIO-based network adapter to the target. The following options are available:

- `NETDEV=NONE` (default): Do not expose a network adapter to the guest.

- `NETDEV=SLIRP`: Use QEMU's userspace networking capabilities (through
  `libslirp`), which provides the target with an emulated network and a gateway
  bridging outgoing TCP and UDP connections onto sockets of the host operating
  system. `NETDEV_SLIRP_ARGS` can be used to pass further arguments to the
  `netdev`, for instance to forward ports from host to guest. For example, to
  forward the TCP port `8080` to the guest at `192.168.1.50:80`, use the
  following command line:

  ```
  $ make run NETDEV=SLIRP NETDEV_SLIRP_ARGS=hostfwd=tcp::8080-192.168.1.50:80
  ```

- `NETDEV=TAP`: Create a TAP network device on the host and expose the
  corresponding remote end to the guest's VirtIO network card. This establishes
  a layer-2 link between the host and guest. This option assumes that QEMU has
  the necessary permissions to use (or create) the device on the host. The
  interface will be unconfigured and needs to be made active and be assigned an
  IP address manually.

- `NETDEV=SUDO-TAP`: Like `TAP`, but run QEMU as root through `sudo`. This will
  likely prompt for a password.

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
      Finished `release` profile [optimized + debuginfo] target(s) in 0.08s
     text    data     bss     dec     hex filename
    87552      48   41940  129540   1fa04 target/riscv32imac-unknown-none-elf/release/qemu_rv32_virt

  Running QEMU emulator version 10.0.2 (tested: 8.2.7, 9.1.3; known broken: <= 8.1.5, >= 9.2.3) with
    - kernel target/riscv32imac-unknown-none-elf/release/qemu_rv32_virt.elf
  To exit type C-a x

  qemu-system-riscv32 \
    -machine virt \
    -semihosting \
    -global driver=riscv-cpu,property=smepmp,value=true \
    -global virtio-mmio.force-legacy=false \
    -device virtio-rng-device  \
    -nographic \
    -bios target/riscv32imac-unknown-none-elf/release/qemu_rv32_virt.elf
  QEMU RISC-V 32-bit "virt" machine, initialization complete.
  - Found VirtIO EntropySource device, enabling RngDriver
  - VirtIO NetworkCard device not found, disabling EthernetTapDriver
  Entering main loop.
  tock$
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

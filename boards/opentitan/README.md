OpenTitan RISC-V Board
=================

- https://opentitan.org/

OpenTitan is the first open source project building a transparent, high-quality reference design and integration guidelines for silicon root of trust (RoT) chips.

Tock currently supports the OpenTitan snapshot-20191101-2 release on a Nexys Video FPGA, as described here: https://docs.opentitan.org/doc/ug/getting_started_fpga/index.html.

You can get started with OpenTitan using either the Nexys Video FPGA board or simulation. See the OpenTitan [getting started](https://docs.opentitan.org/doc/ug/getting_started/index.html) for more details.

Programming
-----------

Tock on OpenTitan requires lowRISC/opentitan@4429c362900713c059fbd870db140e0058e1c0eb or newer. In general it is recommended that users start with the latest OpenTitan bitstream and if that results in issues try the one mentioned above.

For more information you can follow the [OpenTitan development flow](https://docs.opentitan.org/doc/ug/getting_started_fpga/index.html#testing-the-demo-design) to flash the image.

First setup the development board using the steps here: https://docs.opentitan.org/doc/ug/getting_started_fpga/index.html. You need to make sure the boot ROM is working and that your machine can communicate with the OpenTitan ROM. You will need to use the `PROG` USB port on the board for this.

To use `make flash` you first need to clone the OpenTitan repo and build the `spiflash` tool.

In the OpenTitan repo build the `spiflash` program.

```shell
make -C sw/host/spiflash clean all
```

Export the `OPENTITAN_TREE` environment variable to point to the OpenTitan tree.

```shell
export OPENTITAN_TREE=/home/opentitan/
```

Back in the Tock directory run `make flash`

If everything works you should see something like this on the console. If you need help getting console access check the [testing the design](https://docs.opentitan.org/doc/ug/getting_started_fpga/index.html#testing-the-demo-design) section in the OpenTitan documentation.

```
bootstrap: DONE!
Jump!
OpenTitan initialisation complete. Entering main loop
```

You can also just use the `spiflash` program manually to download the image to the board if you don't want to use `make flash`.

```shell
./sw/host/spiflash/spiflash --input=../../target/riscv32imc-unknown-none-elf/release/opentitan.bin
```

NOTE: You will need to download the Tock binary after every power cycle.

### Compiling the Kernel for FPGA or Verilator

Opentitan is supported on both an FPGA and in Verilator. Slightly different
versions of the EarlGrey chip implementation are required for the different
platforms. By default the kernel is compiled for the FPGA. To compile for
Verilator, run:

```shell
make BOARD_CONFIGURATION=sim_verilator
```

To explicitly specify the FPGA, run:

```shell
make BOARD_CONFIGURATION=fpga_nexysvideo
```

Programming Apps
----------------

Tock apps for OpenTitan must be included in the Tock binary file flashed with the steps mentioned above.

Apps are built out of tree. Currently [libtock-rs](https://github.com/tock/libtock-rs) apps work well while [libtock-c](https://github.com/tock/libtock-c) apps require a special branch and complex work arounds. It is recommended that libtock-rs apps are used.

Once an app is built and a tbf file is generated, you can use `riscv32-none-elf-objcopy` with `--update-section` to create an ELF image with the
apps included.

```shell
$ riscv32-oe-elf-objcopy \
    --update-section .apps=<...>/libtock-rs/target/riscv32imc-unknown-none-elf/tab/opentitan/hello_world/rv32imc.tbf \
    <...>/tock/target/riscv32imc-unknown-none-elf/release/opentitan.elf\
    <...>/tock/target/riscv32imc-unknown-none-elf/release/opentitan-app.elf
```

You will then need to convert this new elf to a binary file.

```shell
$ riscv32-oe-elf-objcopy \
    --output-target=binary \
    <...>/tock/target/riscv32imc-unknown-none-elf/release/opentitan-app.elf \
    <...>/tock/target/riscv32imc-unknown-none-elf/release/opentitan-app.bin

```

The OpenTitan Makefile can also handle this process automatically. Follow the steps above but instead run the `flash-app` make target.

```shell
$ make flash-app APP=<...> OPENTITAN_TREE=/home/opentitan/
```

You will need to have the GCC version of RISC-V 32-bit objcopy installed as the LLVM one doesn't support updating sections.


Running in QEMU
---------------

The OpenTitan application can be run in the QEMU emulation platform for RISC-V, allowing quick and easy testing.

Unfortunately you need QEMU 5.1, which at the time of writing is unlikely to be available in your distro. Luckily Tock can build QEMU for you. From the top level of the Tock source just run `make ci-setup-qemu` and follow the steps.

QEMU can be started with Tock using the `qemu` make target:

```shell
$ make OPENTITAN_BOOT_ROM=<path_to_opentitan>/sw/device/boot_rom/boot_rom_fpga_nexysvideo.elf qemu
```

Where OPENTITAN_BOOT_ROM is set to point to the OpenTitan ELF file. This is usually located at `sw/device/boot_rom/boot_rom_fpga_nexysvideo.elf` in the OpenTitan build output.

QEMU can be started with Tock and a userspace app with the `qemu-app` make target:

```shell
$ make OPENTITAN_BOOT_ROM=<path_to_opentitan/sw/device/boot_rom/boot_rom_fpga_nexysvideo.elf> APP=/path/to/app.tbf qemu-app
```

The TBF must be compiled for the OpenTitan board which is, at the time of writing,
supported for Rust userland apps using libtock-rs. For example, you can build
the Hello World example app from the libtock-rs repository by running:

```
$ cd [LIBTOCK-RS-DIR]
$ make flash-opentitan
$ tar xf target/riscv32imac-unknown-none-elf/tab/opentitan/hello_world.tab
$ cd [TOCK_ROOT]/boards/opentitan
$ make APP=[LIBTOCK-RS-DIR]/rv32imac.tbf qemu-app
```

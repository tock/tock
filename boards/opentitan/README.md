OpenTitan RISC-V Board
=================

- https://opentitan.org/

OpenTitan is the first open source project building a transparent, high-quality reference design and integration guidelines for silicon root of trust (RoT) chips.

Tock currently supports the OpenTitan snapshot-20191101-2 release on a Nexys Video FPGA, as described here: https://docs.opentitan.org/doc/ug/getting_started_fpga/index.html.

You can get started with OpenTitan using either the Nexys Video FPGA board or simulation. See the OpenTitan [getting started](https://docs.opentitan.org/doc/ug/getting_started/index.html) for more details.

Programming
-----------

Tock on OpenTitan requires lowRISC/opentitan@0e5d819d61b5bf56f8453ad877eb10e3c52fc542 or newer.

For more information you can follow the [OpenTitan development flow](https://docs.opentitan.org/doc/ug/getting_started_fpga/index.html#testing-the-demo-design) to flash the image.

First setup the development board using the steps here: https://docs.opentitan.org/doc/ug/getting_started_fpga/index.html. You need to make sure the boot ROM is working and that your machine can communicate with the OpenTitan ROM. You will need to use the `PROG` USB port on the board for this.

To use `make flash` you first need to clone the OpenTitan repo and build the `spiflash` tool.

In the OpenTitan repo build the `spiflash` program.

```shell
make -C sw/host/spiflash clean all
```

Export the `OPENTITAN_TREE` enviroment variable to point to the OpenTitan tree.

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

Tock applications currenlty don't work on OpenTitan.

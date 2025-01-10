Cypress CY8CPROTO-062-4343W
===========================

<img src="https://www.infineon.com/export/sites/default/_images/product/evaluation-boards/cypress-boards/CY8CPROTO-062-4343W_0.jpg_1361197165.jpg" width="40%">

The [Cypress CY8CPROTO-062-4343W](https://www.infineon.com/cms/en/product/evaluation-boards/cy8cproto-062-4343w/) is a prototyping board based on the PSoC 62xA SoC.

## Getting started

1. Download the [Infineon Customized OpenOCD](https://github.com/Infineon/openocd/releases/latest)
2. Set OPENOCD_ROOT to the directory from extracted archive.
3. Connect the computer to the `KitProg3` USB connector.
4. Inside the board directory, run the following command (with root privilegies in case of failure):
   `${OPENOCD_ROOT}/bin/openocd -s ${OPENOCD_ROOT}/scripts`

## Flashing the kernel

The kernel can be programmed by going inside the board's directory and running:
```bash
$ make flash
```

## Flashing an app

Apps are built out-of-tree. Once an app is built, you must add the path to the generated TBF in the Makefile (APP variable), then run:
```bash
$ make program
```

This will generate a new ELF file that can be deployed on the CY8CPROTO-062-4343W via gdb and OpenOCD.

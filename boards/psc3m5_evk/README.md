Cypress CY8CPROTO-062-4343W
===========================

<img src="https://www.infineon.com/export/sites/default/_images/product/evaluation-boards/cypress-boards/CY8CPROTO-062-4343W_0.jpg_1361197165.jpg" width="40%">

The [Cypress CY8CPROTO-062-4343W](https://www.infineon.com/cms/en/product/evaluation-boards/cy8cproto-062-4343w/) is a prototyping board based on the PSoC 62xA SoC.

## Getting started

Install `probe-rs`.

```
cargo install probe-rs-tools

# on macOS:
brew install probe-rs
```

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

This will generate a new ELF file that can be deployed on the CY8CPROTO-062-4343W via gdb and probe-rs.

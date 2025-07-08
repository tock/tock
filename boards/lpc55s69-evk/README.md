NXP LPCXpresso55S69 Development Board
======================================

For more details about the board [visit the NXP board website](https://www.nxp.com/design/design-center/software/development-software/mcuxpresso-software-and-tools-/lpcxpresso-boards/lpcxpresso55s69-development-board:LPC55S69-EVK). More details about the chip can be found [here](https://www.nxp.com/products/LPC55S6x)

## Flashing the kernel

### Prerequisites

The primary tool used for flashing this board is `probe-rs`. You will need to install it. The recommended way is using `cargo`, the Rust package manager. More details can be found [here](https://probe.rs/docs/getting-started/installation/)

```bash
$ cargo install probe-rs
```

### Compiling and Flashing

To compile the Tock kernel and flash it to the board, navigate to the `boards/lpc55s69-evk` directory and run `make flash-debug` or `make flash`. This command compiles the kernel either in debug mode or in release mode and uses `probe-rs` to flash the binary file to the chip.

```bash
$ make flash-debug
        or
$ make flash
```

The expected output of the command is looking like this:

```bash
$ make flash-debug
    Finished `dev` profile [optimized + debuginfo] target(s) in 8.95s
  text   data   bss   dec   hex filename
  409B   32    8220 12350  303e /home/USER/tock/target/thumbv8em.main-none-eabihf/debug/lpc55s69-evk
0b7602b822e4507274cd6dd2a47879cee487d8981b83186a9931adc2269880bf  /home/USER/tock/target/thumbv8em.main-none-eabihf/debug/lpc55s69-evk.bin
probe-rs run --chip LPC55S69JBD100 /home/USER/tock/targetthumbv8m.main-none-eadihf/debug/lpc55s69-evk.elf
```


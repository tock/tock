# TockOS port for Teensy 3.2

This directory is an experimental port of the Tock embedded operating system to
the Teensy 3.2.

## Compiling

To compile the kernel, simply run `make` in this directory. You must
have the prerequiste build tools installed, as detailed in the
[Tock getting started guide](https://github.com/helena-project/tock/blob/master/doc/Getting_Started.md).

## Programming the Teensy

Connect the Teensy via USB to your computer, and run `make program` from the
root directory. You should see a prompt telling you to press the reset button on
your board. Once you press the button, `teensy-loader-cli` will flash the kernel
onto the board using the Teensy's builtin HalfKay bootloader.

## Blink

The `boards/teensy3.2/src/tests` directory contains tests which can be run instead of running
the normal kernel main loop. To run `blink` from the kernel, edit
`tests/mod.rs` to the following:

```rust
// Set this function to run whatever test you desire. Test functions are named XXX_test by convention.
pub fn test() {
    blink::blink_test();
}

// Set this to true to make the kernel run the test instead of main.
pub const TEST: bool = true;
```

Then run `make program` and the kernel will be compiled and flashed to your
Teensy. You should see the orange LED blinking!

To get a blink with UART console output on TX0, run `print::print_test()` instead.

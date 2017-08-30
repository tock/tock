Running Your First Tock App
===========================

This guide will help you get the `blink` app running on top of Tock kernel.


Setup
-----

You need to be able to compile and load Tock.
See the [getting started README](../Getting_Started.md) on how to get setup.

You also need hardware that supports Tock.


Instructions
------------

1. **Compile Tock**. In the root of the Tock directory, compile the kernel for
your hardware platform. You can find a list of boards by running `make list`.

    ```bash
    cd boards/imix
    make
    ```

    This will create binaries of the Tock kernel. Tock is compiled with
    Cargo, a package manager for Rust applications. The first time Tock is built
    all of the crates must be compiled. On subsequent builds, crates that haven't
    changed will not have to be rebuilt and the compilation will be faster.


2. **Load the Tock Kernel**. The next step is to program the Tock kernel onto
your hardware. To do this, run:

    ```bash
    make program  # Load code via bootloader
      -- or --    # Check the README in your board folder
    make flash    # Load code via jtag
    ```

    in the board directory. Now you have the kernel loaded onto the hardware.
    The kernel configures the hardware and provides drivers for many hardware
    resources, but does not actually include any application logic. For that, we
    need to load an application.

    Note, you only need to program the kernel once. Loading applications does
    not alter the kernel, and applications can be re-programed without
    re-programming the kernel.

3. **Load an Application**. For this introduction, we will program the blink
app. The app can be found in the `userland/examples` directory, and is
compiled and loaded much like the kernel is.

    ```bash
    cd userland/examples/blink
    make program
    ```

    When the `make` command finishes you should see the LEDs on the board blinking.
    Congratulations! You have just programmed your first Tock application.

Running Your First Tock App
===========================

This guide will help you get the `blink` app running on top of Tock kernel.

Setup
-----

You need to be able to compile and load Tock.
See the [getting started README](../Getting_Started.md) on how to get setup.

You also need [hardware](https://tockos.org/hardware) that supports Tock.


Instructions
------------

1. **Compile Tock**. In the root of the Tock directory, compile the kernel for
your hardware platform. You can find a list of boards by running `make list`.
For example if your board is `imix` then:

    ```bash
    cd boards/imix
    make
    ```

    If you have another board just replace imix with `<your-board>`

    This will create binaries of the Tock kernel. Tock is compiled with
    Cargo, a package manager for Rust applications. The first time Tock is built
    all of the crates must be compiled. On subsequent builds, crates that haven't
    changed will not have to be rebuilt and the compilation will be faster.


2. **Load the Tock Kernel**. The next step is to program the Tock kernel onto
your hardware. See the [getting started README](../Getting_Started.md) how the
kernel is installed on your board two options are supported: `program` and
`flash`

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

3. **Load an Application**. First, we need to remove any applications already
on the board. Note that Tockloader by default will install any application in
addition to whatever is already installed on the board.

    ```bash
    tockloader erase-apps
    ```

    For this introduction, we will program the blink app. Tockloader supports
    installing apps from a repository, so installing the blink app is simple:

    ```bash
    tockloader install blink
    ```

    Your specific board may require additional arguments, please see the readme
    in the `boards/` folder for more details.

    We can also compile the blink app and load our compiled version. The basic C
    version of blink is located in the
    [libtock-c](https://github.com/tock/libtock-c) repository. Clone that
    repository, then navigate to `examples/blink`. From there, you should be
    able to compile it and install it by:

    ```bash
    make
    tockloader install
    ```

    When the blink app is installed you should see the LEDs on the board
    blinking. Congratulations! You have just programmed your first Tock
    application.

OpenTitan RISC-V Board
======================

- https://opentitan.org/

OpenTitan is an open source project building a transparent,
high-quality reference design and integration guidelines for
silicon root of trust (RoT) chips.

Tock currently supports OpenTitan on the ChipWhisperer
CW310 FPGA board. For more details on the boards see:
https://docs.opentitan.org/doc/ug/fpga_boards/

You can get started with OpenTitan using either the ChipWhisperer CW310
board or a simulation. See the OpenTitan
[getting started guide](https://opentitan.org/guides/getting_started/index.html)
for more details.

Supported version
-----------------

The OpenTitan project is producing its first chip, _Earlgrey_.
You should use either an FPGA
[bitstream](https://storage.googleapis.com/opentitan-bitstreams/earlgrey_es/bitstream-edf5e35f5d50a5377641c90a315109a351de7635.tar.gz)
or simulator built from that version of the OpenTitan codebase.

Programming
-----------

The latest supported version (commit SHA) of OpenTitan is specified in the
[OPENTITAN_SUPPORTED_SHA](https://github.com/tock/tock/blob/master/boards/opentitan/earlgrey-cw310/Makefile)
make variable found in `boards/opentitan/earlgrey-cw310/Makefile`

In *general* it is recommended that users start with the commit specified by `OPENTITAN_SUPPORTED_SHA` as newer
versions **have not been** tested.

> Note: when building, you can pass in `SKIP_OT_VERSION_CHECK=yes` to skip the trivial OpenTitan version check, this maybe useful when developing or testing across multiple versions of OpenTitan.

Setup
-----

### Setup OpenTitan

You can follow the steps at https://opentitan.org/book/doc/getting_started/index.html
for a guide to setup the OpenTitan repo.

There are more details for setting up dependencies available at
https://opentitan.org/book/doc/getting_started/unofficial/index.html

The quick steps for setting up the OpenTitan repo are

```shell
git clone https://github.com/lowRISC/opentitan.git
cd opentitan

# Use the OpenTitan_SHA currently supported by Tock
git checkout <OpenTitan_SHA>
pip3 install --user -r python-requirements.txt
```

ChipWhisper CW310
-----------------

To use `make flash` you first need to clone the OpenTitan repo and ensure that
the Python dependencies are installed.

Then you need to build the OpenTitan tools:

```shell
./bazelisk.sh build //sw/host/opentitantool
```

You might need to run these commands to get it to work

```shell
ln -s /usr/bin/ld.lld /usr/sbin/ld.lld
ln -s /usr/bin/gcc /usr/sbin/gcc
```

Next connect to the board's serial with a second terminal:

```shell
screen /dev/ttyACM1 115200,cs8,-ixon,-ixoff
```

Then you need to flash the bitstream with:


```shell
./bazel-bin/sw/host/opentitantool/opentitantool.runfiles/lowrisc_opentitan/sw/host/opentitantool/opentitantool --interface=cw310 fpga load-bitstream lowrisc_systems_chip_earlgrey_cw310_0.1.bit.orig
```

After which you should see some output in the serial window.

Then in the Tock board directory export the `OPENTITAN_TREE` environment
variable to point to the OpenTitan tree.

```shell
export OPENTITAN_TREE=/home/opentitan/
```

then you can run `make flash` or `make test-hardware` to use the board.

Verilator
---------

Opentitan is supported on both an FPGA and in Verilator. Slightly different
versions of the EarlGrey chip implementation are required for the different
platforms. By default the kernel is compiled for the FPGA.

### Setting up Verilator

For a full guide see the official [OpenTitan Verilator documentation](https://docs.opentitan.org/doc/ug/getting_started_verilator/)

A quick summary on how to do this is included below though

### Build Boot (test) Rom/OTP Image and FuseSOC

Build **only the targets** we care about. To speed up the building, multi-thread the build process with `--jobs x` where x is the thread count.

```shell
# To build the test-ROM
./bazelisk.sh build //sw/device/lib/testing/test_rom:test_rom

# To build OTP
./bazelisk.sh build //hw/ip/otp_ctrl/...

# To build FuseSOC
./bazelisk.sh build //hw:verilator
```

### Test Verilator

You can use the following to automatically build the relevant targets and run a quick test with

```shell
./bazelisk.sh test --test_output=streamed //sw/device/tests:uart_smoketest_sim_verilator
```

or manually with

```shell
bazel-out/k8-fastbuild/bin/hw/build.verilator_real/sim-verilator/Vchip_sim_tb \
                                    --meminit=rom,./bazel-out/k8-fastbuild-ST-97f470ee3b14/bin/sw/device/lib/testing/test_rom/test_rom_sim_verilator.scr.39.vmem \
                                    --meminit=otp,./bazel-out/k8-fastbuild/bin/hw/ip/otp_ctrl/data/rma_image_verilator.vmem

# Read the output, you want to attach screen to UART, for example
# "UART: Created /dev/pts/4 for uart0. Connect to it with any terminal program, "

screen /dev/pts/4

# Wait a few minutes
# You should eventually see messages in screen
# Once you see "Test ROM complete, jumping to flash!" you know it works, note at this point we haven't provided flash image (so it ends here).
```

At this point Opentitan on Verilator should be ready to go!

### Build and Run Tock

You can also use the Tock Make target to automatically build Tock and run it with Verilator (within `boards/opentitan/earlgrey-cw310`) run:

```shell
make BOARD_CONFIGURATION=sim_verilator verilator
```
The above command should **compile relevant targets and start Verilator simulation**.

However, to manually compile Tock for Verilator, run:

```shell
make BOARD_CONFIGURATION=sim_verilator
```

You will then need to generate a vmem file (must be at the TOP_DIR of tock to execute the following):

```shell
srec_cat \
    target/riscv32imc-unknown-none-elf/release/earlgrey-cw310.bin \
    --binary --offset 0 --byte-swap 8 --fill 0xff \
    -within target/riscv32imc-unknown-none-elf/release/earlgrey-cw310.bin\
    -binary -range-pad 8 --output binary.64.vmem --vmem 64
```

And Verilator can be run with:

```shell
${OPENTITAN_TREE}/bazel-out/k8-fastbuild/bin/hw/build.verilator_real/sim-verilator/Vchip_sim_tb \
    --meminit=rom,${OPENTITAN_TREE}/bazel-out/k8-fastbuild-ST-97f470ee3b14/bin/sw/device/lib/testing/test_rom/test_rom_sim_verilator.scr.39.vmem \
    --meminit=flash,./binary.64.vmem \
    --meminit=otp,${OPENTITAN_TREE}/bazel-out/k8-fastbuild/bin/hw/ip/otp_ctrl/data/rma_image_verilator.vmem
```

In both cases expect Verilator to run for **tens of minutes** before you see anything.

Programming Apps
----------------

Tock apps for OpenTitan must be included in the Tock binary file flashed with
the steps mentioned above.

**Apps are built out of tree.**

The OpenTitan Makefile can also handle this process automatically. Follow
the steps above but instead run the `flash-app` make target.

```shell
make flash-app APP=<...> OPENTITAN_TREE=/home/opentitan/
```

You will need to have the GCC version of [RISC-V 32-bit objcopy](https://github.com/riscv-collab/riscv-gnu-toolchain/blob/master/README.md) installed as
the LLVM one doesn't support updating sections.

### Programming Apps in Verilator

A **single app** in `.tbf (tock binary format)` can be bundled and loaded with the kernel into Verilator with:

```shell
make APP=<...> BOARD_CONFIGURATION=sim_verilator verilator
```

### Libtock-C App Verilator Example

To load a libtock-c app, we can do the following to load the `c_hello` sample app:

**Build app:**
```shell
git clone https://github.com/tock/libtock-c.git
cd libtock-c/examples/c_hello/
make RISCV=1
```
**Load and Run:**

Now, in the Opentitan board directory in tock (`tock/boards/opentitan/earlgrey-cw310`)

```shell
make APP=<PATH_TO_LIBTOCK-C>/examples/c_hello/build/rv32imc/rv32imc.0x20030080.0x10005000.tbf BOARD_CONFIGURATION=sim_verilator verilator
```

Note: be sure to use the correct `tbf` file here, that is,
> use: `rv32imc.0x20030080.0x10005000.tbf`

as this falls within the supported app flash region (0x20030080) and ram (0x10005000) regions for Opentitan.

**App Output:**

To see the output, when Verilator loads up it will show the endpoint for the
pseudoterminal slave, something like `UART: Created /dev/pts/7 for uart0`.

```console
$ screen /dev/pts/7
.
...
OpenTitan initialisation complete. Entering main loop
Hello World!
```

Running in QEMU
---------------

The OpenTitan application can be run in the QEMU emulation platform for
RISC-V, allowing quick and easy testing. This is also a good option for
those who can't afford the FPGA development board.

Unfortunately you need QEMU 7.2, which at the time of writing is unlikely
to be available in your distro. Luckily Tock can build QEMU for you. From
the top level of the Tock source just run `make ci-setup-qemu` and
follow the steps.

QEMU can be started with Tock using the `qemu` make target:

```shell
make qemu
```

QEMU can be started with Tock and a userspace app with the `qemu-app` make
target:

```shell
make APP=/path/to/app.tbf qemu-app
```

The TBF must be compiled for the OpenTitan board. For example, you can build
the Hello World example app from the libtock-rs repository by running:

```shell
cd "$libtock_rs_dir"
make opentitan EXAMPLE=console
cd "${tock_dir}/boards/opentitan/earlgrey-cw310"
make APP=$"{libtock_rs_dir}/target/tbf/opentitan/console.tbf" qemu-app
```

QEMU GDB Debugging [**earlgrey-cw310**]
------------------

GDB can be used for debugging with QEMU. This can be useful when debugging a particular application/kernel. 

Start by installing the respective version of gdb.

**Arch**:

```shell
sudo pacman -S riscv32-elf-gdb
```
**Ubuntu**:
```shell
sudo apt-get install gdb-multiarch
```

In the board directory, QEMU can be started in a suspended state with gdb ready to be connected. 

```shell
make qemu-gdb
```

or with an app ready to be loaded.

```shell
make APP=/path/to/app.tbf qemu-app-gdb
```

In a separate shell, start gdb

**Arch**

```console
$ riscv32-elf-gdb [/path/to/tock.elf]
> target remote:1234            #1234 is the specified default port
```

**Ubuntu**

```console
$ gdb-multiarch [/path/to/tock.elf]
> set arch riscv
> target remote:1234            #1234 is the specified default port
```

Once attached, standard gdb functionality is available. Additional debug symbols can be added with.
```console
add-symbol-file <tock.elf>
add-symbol-file <app.elf>
```

Unit tests
----------
The Tock OpenTitan boards include automated unit tests to test the kernel.

To run the unit tests on QEMU, just run:

```shell
make test
```

in the specific board directory.

To run the test on hardware use the following steps to build the OTBN binary and run it on hardware:

**Note: You will need to have **Vivado 2020.2 Lab Edition** installed to be able to build `rsa.elf`. See here for an [installation guide](https://docs.opentitan.org/doc/getting_started/install_vivado/) from the OpenTitan docs. Once installed source the settings.sh file. Can be done with:**

```shell
source <path_to_installation>/Xilinx/Vivado_Lab/2020.2/settings64.sh
```
We can now build the `rsa.elf` with:
```shell
cd "${OPENTITAN_TREE}"
# Build OTBN Binary
./bazelisk.sh build //sw/device/tests:otbn_rsa_test

# Package binary as a Tock app
elf2tab --verbose -n "otbn-rsa" --kernel-minor 0 --kernel-major 2 --disable --app-heap 0 --kernel-heap 0 --stack 0 ./bazel-out/k8-fastbuild-ST-2cc462681f62/bin/sw/otbn/crypto/rsa.elf

# Run on hardware
cd "${tock_dir}/boards/opentitan/earlgrey-cw310"
make APP="${OPENTITAN_TREE}/bazel-out/sw/otbn/rsa.tbf" test-hardware
```

### For Verilator

To load the OTBN binary and run it on Verilator, use:

```shell
elf2tab --verbose -n "otbn-rsa" --kernel-minor 0 --kernel-major 2 --disable --app-heap 0 --kernel-heap 0 --stack 0 ./bazel-out/k8-fastbuild-ST-2cc462681f62/bin/sw/otbn/crypto/rsa.elf

make APP="${OPENTITAN_TREE}/bazel-out/k8-fastbuild-ST-2cc462681f62/bin/sw/otbn/crypto/rsa.tbf" BOARD_CONFIGURATION=sim_verilator test-verilator
```

The output on a CW310 should look something like this:

```
OpenTitan initialisation complete. Entering main loop
check run AES128 ECB...
aes_test passed (ECB Enc Src/Dst)
aes_test passed (ECB Dec Src/Dst)
aes_test passed (ECB Enc In-place)
aes_test passed (ECB Dec In-place)
    [ok]
check run AES128 CBC...
aes_test passed (CBC Enc Src/Dst)
aes_test passed (CBC Dec Src/Dst)
aes_test passed (CBC Enc In-place)
aes_test passed (CBC Dec In-place)
    [ok]
check run AES128 CTR...
aes_test CTR passed: (CTR Enc Ctr Src/Dst)
aes_test CTR passed: (CTR Dec Ctr Src/Dst)
    [ok]
check run CSRNG Entropy 32...
Entropy32 test: first get Ok(())
Entropy test: obtained all 8 values. They are:
[00]: 11358ec6
[01]: cad739e8
[02]: 236b897e
[03]: 707c0162
[04]: 2627c579
[05]: 86b6562c
[06]: a8e0e4f8
[07]: 4b298bcd
    [ok]
check hmac load binary...
    [ok]
check hmac check verify...
    [ok]
start multi alarm test...
    [ok]
check otbn run binary...
    [ok]
start TicKV append key test...
---Starting TicKV Tests---
Key: [18, 52, 86, 120, 154, 188, 222, 240] with value [16, 32, 48] was added
Now retrieving the key
Key: [18, 52, 86, 120, 154, 188, 222, 240] with value [16, 32, 48, 0] was retrieved
Removed Key: [18, 52, 86, 120, 154, 188, 222, 240]
Try to read removed key: [18, 52, 86, 120, 154, 188, 222, 240]
Unable to find key: [18, 52, 86, 120, 154, 188, 222, 240]
Let's start a garbage collection
Finished garbage collection
---Finished TicKV Tests---
    [ok]
trivial assertion...
    [ok]
```

The tests can also be run on Verilator with:

```shell
make BOARD_CONFIGURATION=sim_verilator test-verilator
```

Note that the Verilator tests can take hours to complete.

OpenTitan RISC-V Board
======================

- https://opentitan.org/

OpenTitan is the first open source project building a transparent,
high-quality reference design and integration guidelines for
silicon root of trust (RoT) chips.

Tock currently supports OpenTitan on the ChipWhisperer
CW310 FPGA board. For more details on the boards see:
https://docs.opentitan.org/doc/ug/fpga_boards/

You can get started with OpenTitan using either the, ChipWhisperer CW310
board or a simulation. See the OpenTitan
[getting started](https://docs.opentitan.org/doc/ug/getting_started/index.html)
for more details.

Programming
-----------

Tock on OpenTitan requires
lowRISC/opentitan@199d45626f8a7ae2aef5d9ff73793bf9a4233711 or newer. In
general it is recommended that users start with the latest OpenTitan bitstream
and if that results in issues try the one mentioned above.

For more information you can follow the
[OpenTitan development flow](https://docs.opentitan.org/doc/ug/getting_started_fpga/index.html#testing-the-demo-design)
to flash the image.

First setup the development board using the steps here:
https://docs.opentitan.org/doc/ug/getting_started_fpga/index.html.
You need to make sure the boot ROM is working and that your machine can
communicate with the OpenTitan ROM. You will need to use the `PROG` USB
port on the board for this.

ChipWhisper CW310
-----------------

To use `make flash` you first need to clone the OpenTitan repo and ensure that
the Python dependencies are installed.

```shell
python3 pip install -r python-requirements.txt
```

Next connect to the board's serial with a second terminal:

```shell
screen /dev/ttyACM1 115200,cs8,-ixon,-ixoff
```

Then you need to flash the bitstream with:


```shell
./util/fpga/cw310_loader.py --bitstream lowrisc_systems_chip_earlgrey_cw310_0.1.bit --set-pll-defaults
```

After which you should see some output in the serial window.

Then in the Tock board directoty export the `OPENTITAN_TREE` enviroment
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

## Setting up Verilator

For a full guide see the official OpenTitan documentation: https://docs.opentitan.org/doc/ug/getting_started_verilator/

A quick summary on how to do this is included below though

### Build FuseSoc

```shell
git clone https://github.com/lowRISC/opentitan.git
cd opentitan
git checkout <OpenTitan_SHA>
pip3 install --user -r python-requirements.txt

LANG="en_US.UTF-8" fusesoc --cores-root . run --flag=fileset_top --target=sim --setup --build lowrisc:dv:chip_verilator_sim
```
### Build Boot Rom/OTP Image
Build only the targets we care about.
```shell
ninja -C build-out sw/device/lib/testing/test_rom/test_rom_export_sim_verilator
ninja -C build-out sw/device/otp_img/otp_img_sim_verilator.vmem
```

### Test Verilator

```shell
build/lowrisc_dv_chip_verilator_sim_0.1/sim-verilator/Vchip_sim_tb \
    --meminit=rom,./build-out/sw/device/lib/testing/test_rom/test_rom_sim_verilator.scr.39.vmem \
    --meminit=otp,./build-out/sw/device/otp_img/otp_img_sim_verilator.vmem

# Read the output, you want to attach screen to UART
screen /dev/pts/4

# Wait a few minutes
# You should eventually see messages in screen
# Once you see "waiting for SPI input..." you know it works
```

### Build and Run Tock

To compile Tock for Verilator, run:

```shell
make BOARD_CONFIGURATION=sim_verilator
```

You will then need to generate a vmem file:

```shell
srec_cat \
    target/riscv32imc-unknown-none-elf/release/earlgrey-cw310.bin \
    --binary --offset 0 --byte-swap 8 --fill 0xff \
    -within target/riscv32imc-unknown-none-elf/release/earlgrey-cw310.bin\
    -binary -range-pad 8 --output binary.64.vmem --vmem 64
```

And Verilator can be run with:

```shell
${OPENTITAN_TREE}/build/lowrisc_dv_chip_verilator_sim_0.1/sim-verilator/Vchip_sim_tb \
    --meminit=rom,${OPENTITAN_TREE}/build-out/sw/device/boot_rom/boot_rom_sim_verilator.scr.39.vmem \
    --meminit=flash,./binary.64.vmem \
    --meminit=otp,${OPENTITAN_TREE}/build-bin/sw/device/otp_img/otp_img_sim_verilator.vmem
````

You can also use the Tock Make target to automatically build Tock and run it with Verilator

```shell
make BOARD_CONFIGURATION=sim_verilator verilator
```

In both cases expect Verilator to run for tens of minutes before you see anything.

Programming Apps
----------------

Tock apps for OpenTitan must be included in the Tock binary file flashed with
the steps mentioned above.

Apps are built out of tree.

The OpenTitan Makefile can also handle this process automatically. Follow
the steps above but instead run the `flash-app` make target.

```shell
$ make flash-app APP=<...> OPENTITAN_TREE=/home/opentitan/
```

You will need to have the GCC version of RISC-V 32-bit objcopy installed as
the LLVM one doesn't support updating sections.

Running in QEMU
---------------

The OpenTitan application can be run in the QEMU emulation platform for
RISC-V, allowing quick and easy testing. This is also a good option for
those who can't afford the FPGA development board.

Unfortunately you need QEMU 6.1, which at the time of writing is unlikely
to be avaliable in your distro. Luckily Tock can build QEMU for you. From
the top level of the Tock source just run `make ci-setup-qemu` and
follow the steps.

QEMU can be started with Tock using the `qemu` make target:

```shell
$ make OPENTITAN_BOOT_ROM=<path_to_opentitan>/sw/device/boot_rom/boot_rom_fpga_nexysvideo.elf qemu
```

Where OPENTITAN_BOOT_ROM is set to point to the OpenTitan ELF file. This is
usually located at `sw/device/boot_rom/boot_rom_fpga_nexysvideo.elf` in the
OpenTitan build output. Note that the `make ci-setup-qemu` target will also
download a ROM file.

QEMU can be started with Tock and a userspace app with the `qemu-app` make
target:

```shell
$ make OPENTITAN_BOOT_ROM=<path_to_opentitan/sw/device/boot_rom/boot_rom_fpga_nexysvideo.elf> APP=/path/to/app.tbf qemu-app
```

The TBF must be compiled for the OpenTitan board. For example, you can build
the Hello World example app from the libtock-rs repository by running:

```
$ cd [LIBTOCK-RS-DIR]
$ make flash-opentitan
$ tar xf target/riscv32imac-unknown-none-elf/tab/opentitan/hello_world.tab
$ cd [TOCK_ROOT]/boards/opentitan
$ make APP=[LIBTOCK-RS-DIR]/rv32imac.tbf qemu-app
```

QEMU GDB Debugging [**earlgrey-cw310**]
------------------

GDB can be used for debugging with QEMU. This can be useful when debugging a particular application/kernel. 

Start by installing the respective version of gdb.

**Arch**:
```shell
$ sudo pacman -S riscv32-elf-gdb    
```
**Ubuntu**:
```shell
$ sudo apt-get install gdb-multiarch
```

In the board directory, QEMU can be started in a suspended state with gdb ready to be connected. 
```shell
$ make OPENTITAN_BOOT_ROM=<path_to_opentitan/sw/device/boot_rom/boot_rom_fpga_nexysvideo.elf> qemu-gdb
```
or with an app ready to be loaded.
```shell
$ make OPENTITAN_BOOT_ROM=<path_to_opentitan/sw/device/boot_rom/boot_rom_fpga_nexysvideo.elf> APP=/path/to/app.tbf qemu-app-gdb
```
In a seperate shell, start gdb

**Arch**
```shell
$ riscv32-elf-gdb [/path/to/tock.elf]
> target remote:1234            #1234 is the specified default port
```

**Ubuntu**
```shell
$ gdb-multiarch [/path/to/tock.elf]
> set arch riscv
> target remote:1234            #1234 is the specified default port
```

Once attached, standard gdb functionality is avaliable. Additional debug symbols can be added with.
```
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

To run the test on hardware use these commands to build the OTBN binary and run it on hardware:

```shell
elf2tab --verbose -n "otbn-rsa" --kernel-minor 0 --kernel-major 2 --app-heap 0 --kernel-heap 0 --stack 0 ${OPENTITAN_TREE}/build-out/sw/otbn/rsa.elf
OPENTITAN_TREE=<...> APP=${OPENTITAN_TREE}/build-out/sw/otbn/rsa.tbf make test-hardware
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
Now retriving the key
Key: [18, 52, 86, 120, 154, 188, 222, 240] with value [16, 32, 48, 0] was retrived
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

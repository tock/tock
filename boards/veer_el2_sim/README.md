# VeeR EL2 simulation target

This target uses a Verilator simulation model of the VeeR EL2 core. The core
comes with a predefined testbench that instantiates the core and a simple
interface that alows the software to print text to `stdout` and `console.log`.

Tock uses non-default configuration of VeeR EL2 in order to enable User mode
and change the address of reset vector. This readme explains the steps to do this,
and the `Makefile` of this target also performs the configuration.

The simulation program loads software to the memory and executes a fixed number
of instructions. It also produces a log file with instruction trace (`exec.log`).

For more information about the core and the Verilator testbench see
[the readme of the chip](https://github.com/tock/tock/tree/master/chips/veer_el2)
and [visit the VeeR EL2 repository](https://github.com/chipsalliance/Cores-VeeR-EL2).

This was tested with Verilator 5.006, which can be installed as explained
[in the doc](https://verilator.org/guide/latest/install.html).

## Running simulation in Verilator

In order to compile Tock and start simulation, run:

    make -C boards/veer_el2_sim sim

The expected output is:

    VerilatorTB: Start of sim

    mem_signature_begin = 00000000
    mem_signature_end   = 00000000
    mem_mailbox         = D0580000
    VeeR EL2 initialisation complete.
    Entering main loop.

## Running simulation in Verilator with applications

### Building Tock

In order to compile Tock, run:

    make -C boards/veer_el2_sim

### Building an application

    git clone https://github.com/tock/libtock-c.git
    make -C libtock-c/examples/c_hello -j$(nproc)

### Providing verilog file for simulation

The testbench for Verilator requires a single file with the program
(`program.hex`), so it's necessary to combine the kernel and applications into
a single binary first.

You can use Tockloader to create a binary file representing the flash with the
kernel, and then install the application:

    tockloader flash --board veer_el2_sim --flash-file ./veer_el2_sim.bin --address 0x20000000 ./target/riscv32imc-unknown-none-elf/release/veer_el2_sim.bin
    tockloader install --board veer_el2_sim --arch rv32imc --flash-file ./veer_el2_sim.bin libtock-c/examples/c_hello/build/c_hello.tab
    riscv64-unknown-elf-objcopy --change-addresses 0x20000000 -I binary -O verilog veer_el2_sim.bin program.hex

Now `program.hex` is ready to be used in simulation.

### Starting simulation in Verilator

Clone VeeR EL2:

    git clone https://github.com/chipsalliance/Cores-VeeR-EL2.git
    cd Cores-VeeR-EL2
    git switch --detach da1042557

Increase the maximum number of cycles in simulation:

    sed -i 's/parameter MAX_CYCLES = 2_000_000;/parameter MAX_CYCLES = 10_000_000;/g' testbench/tb_top.sv

There's a testbench that can be built using these commands:

    export RV_ROOT=$(pwd)
    make -C tools CONF_PARAMS='-set build-axi4 -set user_mode=1 -set reset_vec=0x20000000' verilator-build

The program to run should be placed in the current working directory and named
`program.hex`:

    cp ../program.hex .

In order to start the simulation, run:

    ./tools/obj_dir/Vtb_top

The output should look like this:

    VerilatorTB: Start of sim

    mem_signature_begin = 00000000
    mem_signature_end   = 00000000
    mem_mailbox         = D0580000
    VeeR EL2 initialisation complete.
    Entering main loop.
    Hello World!

The execution trace will be located in `exec.log`.

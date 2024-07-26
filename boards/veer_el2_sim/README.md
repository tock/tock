# VeeR EL2 simulation target

For more information about the core and the Verilator testbench [visit the VeeR EL2 repository](https://github.com/chipsalliance/Cores-VeeR-EL2).
This was tested with Verilator 5.006, which can be installed as explained [in the doc](https://verilator.org/guide/latest/install.html).

There's also a repository dedicated to [running Tock on VeeR EL2 in simulation](https://github.com/chipsalliance/VeeR-EL2-tock-example)

## Building software

### Building Tock

In order to compile Tock and convert it to the format used by Verilator, run:

    make -C boards/veer_el2_sim debug -j$(nproc)
    riscv64-unknown-elf-objcopy -O verilog target/riscv32imc-unknown-none-elf/debug/veer_el2_sim.elf kernel.hex

### Building an application

    git clone https://github.com/tock/libtock-c.git
    make -C libtock-c/examples/c_hello TOCK_TARGETS='rv32imc|rv32imc.0x80030080.0x80070000|0x80030080|0x80070000' -j$(nproc)
    riscv64-unknown-elf-objcopy -I binary -O verilog libtock-c/examples/c_hello/build/rv32imc/rv32imc.0x80030080.0x80070000.tbf c_hello.hex

### Output files

As the testbench for Verilator requires a single file with the program (`program.hex`), the kernel and the application need to be combined:

    sed -i 's/@00000000/@80030000/g' c_hello.hex
    cat kernel.hex c_hello.hex > program.hex

Now `program.hex` is ready to be used in simulation.

## Running simulation in Verilator

Clone VeeR EL2:

    git clone https://github.com/chipsalliance/Cores-VeeR-EL2.git
    cd Cores-VeeR-EL2
    git switch --detach da1042557

Increase the maximum number of cycles in simulation:

    sed -i 's/parameter MAX_CYCLES = 2_000_000;/parameter MAX_CYCLES = 10_000_000;/g' testbench/tb_top.sv

There's a testbench that can be built using these commands:

    export RV_ROOT=$(pwd)
    make -C tools USER_MODE=1 verilator-build

The program to run should be placed in the current working directory and named `program.hex`:

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

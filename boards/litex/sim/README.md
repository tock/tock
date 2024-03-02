LiteX SoC in a Verilated Simulation
============================================

This board is targeting a
[verilated](https://www.veripool.org/wiki/verilator) SoC bitstream
built using [LiteX](https://github.com/enjoy-digital/litex).

Since LiteX is a SoC builder, the individual generated SoCs can differ
significantly depending on the release and configuration options
used. This board definition currently targets and has been tested with
- [the LiteX SoC generator, revision
  a6d9955c9d3065](https://github.com/enjoy-digital/litex/tree/a6d9955c9d3065)
- using the included
  [`litex_sim`](https://github.com/enjoy-digital/litex/blob/a6d9955c9d3065/litex/tools/litex_sim.py)
- built around a VexRiscv-CPU with PMP, hardware multiplication and
  compressed instruction support (named `TockSecureIMC`)
- featuring a TIMER0 with 64-bit wide hardware uptime
- using the following configuration options:

  ```
  --csr-data-width=32
  --integrated-rom-size=0x100000
  --integrated-main-ram-size=0x10000000
  --cpu-variant=tock+secure+imc
  --with-ethernet
  --timer-uptime
  --with-gpio
  --rom-init $PATH_TO_TOCK_BINARY
  ```

The `tock+secure+imc` is a custom VexRiscv CPU variant, based on the
build infrastructure in
[pythondata-cpu-vexriscv](https://github.com/litex-hub/pythondata-cpu-vexriscv),
which is patched to introduce a CPU with a Physical Memory Protection
(PMP) unit with Top of Range (TOR) addressing support, hardware
multiplication and compressed instruction support (such that it is
compatible with the `rv32imc` arch).

The [`tock-litex`](https://github.com/lschuermann/tock-litex)
repository contains helpful instructions for how to set up the local
LiteX development and simulation environment.

Many bitstream customizations can be represented in the Tock board by
simply changing the variables in
`src/litex_generated_constants.rs`. To support a different set of FPGA
cores and perform further modifications, the `src/main.rs` file will
have to be modified.

This board makes assumptions about the generated LiteX SoC, such as
CSR locations in memory. The companion repository
[tock-litex](https://github.com/lschuermann/tock-litex) provides
access to an environment with the required LiteX Python packages in
their targeted versions. This board currently targets the release
[2024011101](https://github.com/lschuermann/tock-litex/releases/tag/2024011101)
of `tock-litex`.


Building the SoC / Running the simulation
-----------------------------------------

Please refer to the [LiteX
documentation](https://github.com/enjoy-digital/litex/wiki/) for
instructions on how to install and use the LiteX SoC generator.

Once LiteX is installed, running the simulation:

```
$ cd $PATH_TO_LITEX/litex/tools
$ ./litex_sim.py <configuration options above>
```

This command will build the SoC Verilog files and a LiteX ROM BIOS.
Afterwards, it will use Verilator to create a C++ simulation program,
which is compiled and run. The script may ask for an administrator
password to create a `tap0` network device on the host if built with
Ethernet support. The Tock kernel is included as the integrated ROM
(loaded at address `0x0`) by using the `--rom-init
$PATH_TO_TOCK_BINARY`.

If everything works you should be greeted by the Tock kernel:
```
...
[xgmii_ethernet] loaded (0xa51090)
[clocker] loaded
[clocker] sys_clk: freq_hz=1000000, phase_deg=0
Verilated LiteX+VexRiscv: initialization complete, entering main loop.
```

### Running with Applications

By its nature, this simulated board does not feature any persistent storage. For
this reason, Tockloader is not able to interact with a running LiteX Simulator
instance directly. However, Tockloader includes a flash-file support mode which
supports operating on a binary file representing a device's flash. This can be
used to combine the kernel and applications in a single binary, which can then
be loaded into the simulation using the above method (passed to the `--rom-init`
parameter). An example of this is illustrated below:

```
$ tockloader flash \
    --board litex_sim \
    --flash-file ./litex_sim_flash.bin \
    -a 0x0 \
    ./tock/target/riscv32imc-unknown-none-elf/release/litex_sim.bin
[INFO   ] Using settings from KNOWN_BOARDS["litex_sim"]
[INFO   ] Operating on flash file "./litex_sim_flash.bin".
[INFO   ] Limiting flash size to 0x100000 bytes.
[STATUS ] Flashing binary to board...
[INFO   ] Finished in 0.000 seconds
$ tockloader install \
    --board litex_sim \
    --arch rv32imc \
    --flash-file ./litex_sim_flash.bin \
    ./libtock-c/examples/c_hello/build/c_hello.tab
[INFO   ] Using settings from KNOWN_BOARDS["litex_sim"]
[INFO   ] Operating on flash file "./litex_sim_flash.bin".
[INFO   ] Limiting flash size to 0x100000 bytes.
[STATUS ] Installing app on the board...
[INFO   ] Found sort order:
[INFO   ]   App "c_hello" at address 0x80060
[INFO   ] Finished in 0.002 seconds
```

Debugging
---------

LiteX makes it easy to generate vastly different SoCs. Prior to
creating an issue on GitHub, please verify that all variables in
`src/litex_generated.rs` correspond to the values provided to LiteX or
generated by it. The respective file paths for the source of each
value is included as a comment.

It is possible to extend the VexRiscv-CPU with a debug port, which is
exposed on the Wishbone bus of the SoC. This can then be used to
attach a GDB debugger via the Network using the Etherbone
(`--with-etherbone`) core. For more information refer to the LiteX
documentation and [this quickstart
guide](https://github.com/timvideos/litex-buildenv/wiki/Debugging).

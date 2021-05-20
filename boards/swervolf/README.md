SweRVolf
========

SweRVolf is a FuseSoC-based SoC for the SweRV RISC-V core.

This can be used to run the RISC-V compliance tests, Zephyr OS
or other software in simulators or on FPGA boards. Focus is on
portability, extendability and ease of use; to allow SweRV users
to quickly get software running, modify the SoC to their needs or
port it to new target devices.

https://github.com/chipsalliance/Cores-SweRVolf

Running
-------

FuseSoC can be used to run SweRVolf on either an FPGA board or in a simulation
enviroment.

Tock has been tested on the simulation enviroment.

Running in simulation with EH1 core
-----------------------------------
For full details in setting up the FuseSoC simulator see: https://github.com/chipsalliance/Cores-SweRVolf#prerequisites

The quick steps to setup are shown below:

First install verilator

```shell
sudo pacman -S verilator
```

Then ensure that the Python package fusesoc is installed

```shell
pip install fusesoc
```

Finally the Tock build system can build the simulator.

```shell
make setup-sim
```

If the simulator built correctly you should see: "SweRV+FuseSoC rocks"

Then to run Tock

```shell
make sim
```

NOTE: The Verilator simulation can be slow. Below are some rough estimates
of time when running on a standard x64 laptop.

Boot, hardware initalise, first print: 20 seconds
Boot, hardware initalise, panic: 30 seconds

The below diff below can be used to increase the simulation speed, with no
functionality impact.

```diff
diff --git a/arch/rv32i/src/lib.rs b/arch/rv32i/src/lib.rs
index 994de0a6c..87347e40b 100644
--- a/arch/rv32i/src/lib.rs
+++ b/arch/rv32i/src/lib.rs
@@ -81,18 +81,18 @@ pub extern "C" fn _start() {

             // INITIALIZE MEMORY

-            // Start by initializing .bss memory. The Tock linker script defines
-            // `_szero` and `_ezero` to mark the .bss segment.
-            la a0, {sbss}               // a0 = first address of .bss
-            la a1, {ebss}               // a1 = first address after .bss
-
-          bss_init_loop:
-            beq  a0, a1, bss_init_done  // If a0 == a1, we are done.
-            sw   zero, 0(a0)            // *a0 = 0. Write 0 to the memory location in a0.
-            addi a0, a0, 4              // a0 = a0 + 4. Increment pointer to next word.
-            j bss_init_loop             // Continue the loop.
-
-          bss_init_done:
+          //   // Start by initializing .bss memory. The Tock linker script defines
+          //   // `_szero` and `_ezero` to mark the .bss segment.
+          //   la a0, {sbss}               // a0 = first address of .bss
+          //   la a1, {ebss}               // a1 = first address after .bss
+
+          // bss_init_loop:
+          //   beq  a0, a1, bss_init_done  // If a0 == a1, we are done.
+          //   sw   zero, 0(a0)            // *a0 = 0. Write 0 to the memory location in a0.
+          //   addi a0, a0, 4              // a0 = a0 + 4. Increment pointer to next word.
+          //   j bss_init_loop             // Continue the loop.
+
+          // bss_init_done:


             // Now initialize .data memory. This involves coping the values right at the
```

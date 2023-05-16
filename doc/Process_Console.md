Process Console
 ===============
 
 `process_console` is a capsule that implements a small shell over the UART that allows
 a terminal to inspect the kernel and control userspace processes.

<!-- npm i -g markdown-toc; markdown-toc -i Process_Console.md -->

<!-- toc -->

- [Setup](#setup)
- [Using Process Console](#using-process-console)
- [Commands](#commands)
  * [`help`](#help)
  * [`list`](#list)
    + [`list` Command Fields](#list-command-fields)
  * [`status`](#status)
  * [`start` and `stop`](#start-and-stop)
  * [`terminate` and `boot`](#terminate-and-boot)
  * [`fault`](#fault)
  * [`panic`](#panic)
  * [`reset`](#reset)
  * [`kernel`](#kernel)
  * [`process`](#process)
  * [`commands history`](#commands-history)

<!-- tocstop -->

Setup
 -----

 Here is how to add `process_console` to a board's `main.rs` (the example is taken from the microbit's implementation of the Process console):
 ```rust
 let process_printer = components::process_printer::ProcessPrinterTextComponent::new()
        .finalize(components::process_printer_text_component_static!());

 let _process_console = components::process_console::ProcessConsoleComponent::new(
        board_kernel,
        uart_mux,
        mux_alarm,
        process_printer,
        Some(reset_function),
    )
    .finalize(components::process_console_component_static!(
        nrf52833::rtc::Rtc
    ));
 let _ = _process_console.start();
 ```

> Note: Using the process console might require allocating more stack to the kernel. This is done by modifying the `STACK_MEMORY` variable from the board's `main.rs`.

 Using Process Console
 --------------------

 With this capsule properly added to a board's `main.rs` and the Tock kernel
 loaded to the board, make sure there is a serial connection to the board.
 Likely, this just means connecting a USB cable from a computer to the board.
 Next, establish a serial console connection to the board. An easy way to do
 this is to run:

 ```shell
 $ tockloader listen
[INFO   ] No device name specified. Using default name "tock".
[INFO   ] No serial port with device name "tock" found.
[INFO   ] Found 2 serial ports.
Multiple serial port options found. Which would you like to use?
[0]	/dev/ttyS4 - n/a
[1]	/dev/ttyACM0 - "BBC micro:bit CMSIS-DAP" - mbed Serial Port

Which option? [0] 1
[INFO   ] Using "/dev/ttyACM0 - "BBC micro:bit CMSIS-DAP" - mbed Serial Port".
[INFO   ] Listening for serial output.

tock$ 
 ```
 Commands
 --------

 This module provides a simple text-based console to inspect and control
 which processes are running. The console has eleven commands:
  - [`help`](#help) - prints the available commands and arguments
  - [`list`](#list) - lists the current processes with their IDs and running state
  - [`status`](#status) - prints the current system status
  - [`start n`](#start-and-stop) - starts the stopped process with name n
  - [`stop n`](#start-and-stop) - stops the process with name n
  - [`terminate n`](#terminate-and-boot) - terminates the running process with name n, moving it to the Terminated state
  - [`boot n`](#terminate-and-boot) - tries to restart a Terminated process with name n
  - [`fault n`](#fault) - forces the process with name n into a fault state
  - [`panic`](#panic) - causes the kernel to run the panic handler
  - [`reset`](#reset) - causes the board to reset
  - [`kernel`](#kernel) - prints the kernel memory map
  - [`process n`](#process) - prints the memory map of process with name n
  - [`commands history`](#commands-history) - scrolls through inserted user commands

 For the examples below we will have 2 processes on the board: `blink` (which will blink all the LEDs that are 
 connected to the kernel), and `c_hello` (which prints 'Hello World' when the console is started). Also, a micro:bit v2 board was used as support for the commands, so the results may vary on other devices.
 We will assume that the user has a serial connection to the board, either by using tockloader or another serial port software.
 With that console open, you can issue commands. 
 
 ### `help`
- To get a list of the available commands, use the `help` command:
 ```text
     tock$ help
     Welcome to the process console.
     Valid commands are: help status list stop start fault boot terminate process kernel panic
 ```

 ### `list`
- To see all of the processes on the board, use `list`:
  
```text
    tock$ list
    PID    Name                Quanta  Syscalls  Restarts  Grants  State
    0      blink                    0     26818         0   1/14   Yielded
    1      c_hello                  0         8         0   1/14   Yielded
```
  #### `list` Command Fields

 - `PID`: The identifier for the process. This can change if the process
   restarts.
 - `Name`: The process name.
 - `Quanta`: How many times this process has exceeded its allotted time
   quanta.
 - `Syscalls`: The number of system calls the process has made to the kernel.
 - `Restarts`: How many times this process has crashed and been restarted by
   the kernel.
 - `Grants`: The number of grants that have been initialized for the process
   out of the total number of grants defined by the kernel.
 - `State`: The state the process is in.

  ### `status`
  - To get a general view of the system, use the `status` command: 

```text
    tock$ status
    Total processes: 2
    Active processes: 2
    Timeslice expirations: 0
 ```
  ### `start` and `stop`
  - You can control processes with the `start` and `stop` commands:

 ```text
     tock$ stop blink
     Process blink stopped
     tock$ list
     PID    Name                Quanta  Syscalls  Restarts  Grants  State
     2      blink                    0     22881         1   1/14   StoppedYielded
     1      c_hello                  0         8         0   1/14   Yielded
     tock$ start blink
     Process blink resumed.
     tock$ list
     PID    Name                Quanta  Syscalls  Restarts  Grants  State
     2      blink                    0     23284         1   1/14   Yielded
     1      c_hello                  0         8         0   1/14   Yielded
 ```
  ### `terminate` and `boot`
  - You can kill a process with `terminate` and then restart it with `boot`:

```text
    tock$ terminate blink
    Process blink terminated
    tock$ list
    PID    Name                Quanta  Syscalls  Restarts  Grants  State
    2      blink                    0     25640         1   0/14   Terminated
    1      c_hello                  0         8         0   1/14   Yielded
    tock$ boot blink
    tock$ list
    PID    Name                Quanta  Syscalls  Restarts  Grants  State
    3      blink                    0       251         2   1/14   Yielded
    1      c_hello                  0         8         0   1/14   Yielded
```
### `fault`
  - To force a process into a fault state, you should use the `fault` command:

```text
    tock$ fault blink
    panicked at 'Process blink had a fault', kernel/src/process_standard.rs:412:17
      Kernel version 899d73cdd

    ---| No debug queue found. You can set it with the DebugQueue component.

    ---| Cortex-M Fault Status |---
    No Cortex-M faults detected.

    ---| App Status |---
    𝐀𝐩𝐩: blink   -   [Faulted]
    Events Queued: 0   Syscall Count: 2359   Dropped Upcall Count: 0
    Restart Count: 0
    Last Syscall: Yield { which: 1, address: 0x0 }
    Completion Code: None


    ╔═══════════╤══════════════════════════════════════════╗
    ║  Address  │ Region Name    Used | Allocated (bytes)  ║
    ╚0x20006000═╪══════════════════════════════════════════╝
                │ Grant Ptrs      112
                │ Upcalls         320
                │ Process         920
      0x20005AB8 ┼───────────────────────────────────────────
                │ ▼ Grant          24
      0x20005AA0 ┼───────────────────────────────────────────
                │ Unused
      0x200049FC ┼───────────────────────────────────────────
                │ ▲ Heap            0 |   4260               S
      0x200049FC ┼─────────────────────────────────────────── R
                │ Data            508 |    508               A
      0x20004800 ┼─────────────────────────────────────────── M
                │ ▼ Stack         232 |   2048          
      0x20004718 ┼───────────────────────────────────────────
                │ Unused
      0x20004000 ┴───────────────────────────────────────────
                .....
      0x00040800 ┬─────────────────────────────────────────── F
                │ App Flash      1996                        L
      0x00040034 ┼─────────────────────────────────────────── A
                │ Protected        52                        S
      0x00040000 ┴─────────────────────────────────────────── H

      R0 : 0x00000001    R6 : 0x000406B0
      R1 : 0x00000000    R7 : 0x20004000
      R2 : 0x0000000B    R8 : 0x00000000
      R3 : 0x0000000B    R10: 0x00000000
      R4 : 0x200047AB    R11: 0x00000000
      R5 : 0x200047AB    R12: 0x20004750
      R9 : 0x20004800 (Static Base Register)
      SP : 0x20004778 (Process Stack Pointer)
      LR : 0x00040457
      PC : 0x0004045E
    YPC : 0x0004045E

    APSR: N 0 Z 0 C 1 V 0 Q 0
          GE 0 0 0 0
    EPSR: ICI.IT 0x00
          ThumbBit true 

    Total number of grant regions defined: 14
      Grant  0 : --          Grant  5 : --          Grant 10 : --        
      Grant  1 : --          Grant  6 : --          Grant 11 : --        
      Grant  2 0x0: 0x20005aa0  Grant  7 : --          Grant 12 : --        
      Grant  3 : --          Grant  8 : --          Grant 13 : --        
      Grant  4 : --          Grant  9 : --        

    Cortex-M MPU
      Region 0: [0x20004000:0x20005000], length: 4096 bytes; ReadWrite (0x3)
        Sub-region 0: [0x20004000:0x20004200], Enabled
        Sub-region 1: [0x20004200:0x20004400], Enabled
        Sub-region 2: [0x20004400:0x20004600], Enabled
        Sub-region 3: [0x20004600:0x20004800], Enabled
        Sub-region 4: [0x20004800:0x20004A00], Enabled
        Sub-region 5: [0x20004A00:0x20004C00], Disabled
        Sub-region 6: [0x20004C00:0x20004E00], Disabled
        Sub-region 7: [0x20004E00:0x20005000], Disabled
      Region 1: Unused
      Region 2: [0x00040000:0x00040800], length: 2048 bytes; UnprivilegedReadOnly (0x2)
        Sub-region 0: [0x00040000:0x00040100], Enabled
        Sub-region 1: [0x00040100:0x00040200], Enabled
        Sub-region 2: [0x00040200:0x00040300], Enabled
        Sub-region 3: [0x00040300:0x00040400], Enabled
        Sub-region 4: [0x00040400:0x00040500], Enabled
        Sub-region 5: [0x00040500:0x00040600], Enabled
        Sub-region 6: [0x00040600:0x00040700], Enabled
        Sub-region 7: [0x00040700:0x00040800], Enabled
      Region 3: Unused
      Region 4: Unused
      Region 5: Unused
      Region 6: Unused
      Region 7: Unused

    To debug, run `make debug RAM_START=0x20004000 FLASH_INIT=0x4005d`
    in the app's folder and open the .lst file.

    𝐀𝐩𝐩: c_hello   -   [Yielded]
    Events Queued: 0   Syscall Count: 8   Dropped Upcall Count: 0
    Restart Count: 0
    Last Syscall: Yield { which: 1, address: 0x0 }
    Completion Code: None


    ╔═══════════╤══════════════════════════════════════════╗
    ║  Address  │ Region Name    Used | Allocated (bytes)  ║
    ╚0x20008000═╪══════════════════════════════════════════╝
                │ Grant Ptrs      112
                │ Upcalls         320
                │ Process         920
      0x20007AB8 ┼───────────────────────────────────────────
                │ ▼ Grant          76
      0x20007A6C ┼───────────────────────────────────────────
                │ Unused
      0x20006A04 ┼───────────────────────────────────────────
                │ ▲ Heap            0 |   4200               S
      0x20006A04 ┼─────────────────────────────────────────── R
                │ Data            516 |    516               A
      0x20006800 ┼─────────────────────────────────────────── M
                │ ▼ Stack         128 |   2048          
      0x20006780 ┼───────────────────────────────────────────
                │ Unused
      0x20006000 ┴───────────────────────────────────────────
                .....
      0x00041000 ┬─────────────────────────────────────────── F
                │ App Flash      1996                        L
      0x00040834 ┼─────────────────────────────────────────── A
                │ Protected        52                        S
      0x00040800 ┴─────────────────────────────────────────── H

      R0 : 0x00000001    R6 : 0x00040C50
      R1 : 0x00000000    R7 : 0x20006000
      R2 : 0x00000000    R8 : 0x00000000
      R3 : 0x00000000    R10: 0x00000000
      R4 : 0x00040834    R11: 0x00000000
      R5 : 0x20006000    R12: 0x200067F0
      R9 : 0x20006800 (Static Base Register)
      SP : 0x200067D0 (Process Stack Pointer)
      LR : 0x00040A3F
      PC : 0x00040A46
    YPC : 0x00040A46

    APSR: N 0 Z 0 C 1 V 0 Q 0
          GE 0 0 0 0
    EPSR: ICI.IT 0x00
          ThumbBit true 

    Total number of grant regions defined: 14
      Grant  0 : --          Grant  5 : --          Grant 10 : --        
      Grant  1 : --          Grant  6 : --          Grant 11 : --        
      Grant  2 : --          Grant  7 : --          Grant 12 : --        
      Grant  3 : --          Grant  8 : --          Grant 13 : --        
      Grant  4 0x1: 0x20007a6c  Grant  9 : --        

    Cortex-M MPU
      Region 0: [0x20006000:0x20007000], length: 4096 bytes; ReadWrite (0x3)
        Sub-region 0: [0x20006000:0x20006200], Enabled
        Sub-region 1: [0x20006200:0x20006400], Enabled
        Sub-region 2: [0x20006400:0x20006600], Enabled
        Sub-region 3: [0x20006600:0x20006800], Enabled
        Sub-region 4: [0x20006800:0x20006A00], Enabled
        Sub-region 5: [0x20006A00:0x20006C00], Enabled
        Sub-region 6: [0x20006C00:0x20006E00], Disabled
        Sub-region 7: [0x20006E00:0x20007000], Disabled
      Region 1: Unused
      Region 2: [0x00040800:0x00041000], length: 2048 bytes; UnprivilegedReadOnly (0x2)
        Sub-region 0: [0x00040800:0x00040900], Enabled
        Sub-region 1: [0x00040900:0x00040A00], Enabled
        Sub-region 2: [0x00040A00:0x00040B00], Enabled
        Sub-region 3: [0x00040B00:0x00040C00], Enabled
        Sub-region 4: [0x00040C00:0x00040D00], Enabled
        Sub-region 5: [0x00040D00:0x00040E00], Enabled
        Sub-region 6: [0x00040E00:0x00040F00], Enabled
        Sub-region 7: [0x00040F00:0x00041000], Enabled
      Region 3: Unused
      Region 4: Unused
      Region 5: Unused
      Region 6: Unused
      Region 7: Unused

    To debug, run `make debug RAM_START=0x20006000 FLASH_INIT=0x4085d`
    in the app's folder and open the .lst file.
```
### `panic`
  - You can also force a kernel panic with the `panic` command:

```text
    tock$ panic

    panicked at 'Process Console forced a kernel panic.', capsules/src/process_console.rs:959:29
      Kernel version 899d73cdd

    ---| No debug queue found. You can set it with the DebugQueue component.

    ---| Cortex-M Fault Status |---
    No Cortex-M faults detected.

    ---| App Status |---
    𝐀𝐩𝐩: blink   -   [Yielded]
    Events Queued: 0   Syscall Count: 1150   Dropped Upcall Count: 0
    Restart Count: 0
    Last Syscall: Yield { which: 1, address: 0x0 }
    Completion Code: None


    ╔═══════════╤══════════════════════════════════════════╗
    ║  Address  │ Region Name    Used | Allocated (bytes)  ║
    ╚0x20006000═╪══════════════════════════════════════════╝
                │ Grant Ptrs      112
                │ Upcalls         320
                │ Process         920
      0x20005AB8 ┼───────────────────────────────────────────
                │ ▼ Grant          24
      0x20005AA0 ┼───────────────────────────────────────────
                │ Unused
      0x200049FC ┼───────────────────────────────────────────
                │ ▲ Heap            0 |   4260               S
      0x200049FC ┼─────────────────────────────────────────── R
                │ Data            508 |    508               A
      0x20004800 ┼─────────────────────────────────────────── M
                │ ▼ Stack         232 |   2048          
      0x20004718 ┼───────────────────────────────────────────
                │ Unused
      0x20004000 ┴───────────────────────────────────────────
                .....
      0x00040800 ┬─────────────────────────────────────────── F
                │ App Flash      1996                        L
      0x00040034 ┼─────────────────────────────────────────── A
                │ Protected        52                        S
      0x00040000 ┴─────────────────────────────────────────── H

      R0 : 0x00000001    R6 : 0x000406B0
      R1 : 0x00000000    R7 : 0x20004000
      R2 : 0x00000004    R8 : 0x00000000
      R3 : 0x00000004    R10: 0x00000000
      R4 : 0x200047AB    R11: 0x00000000
      R5 : 0x200047AB    R12: 0x20004750
      R9 : 0x20004800 (Static Base Register)
      SP : 0x20004778 (Process Stack Pointer)
      LR : 0x00040457
      PC : 0x0004045E
    YPC : 0x0004045E

    APSR: N 0 Z 0 C 1 V 0 Q 0
          GE 0 0 0 0
    EPSR: ICI.IT 0x00
          ThumbBit true 

    Total number of grant regions defined: 14
      Grant  0 : --          Grant  5 : --          Grant 10 : --        
      Grant  1 : --          Grant  6 : --          Grant 11 : --        
      Grant  2 0x0: 0x20005aa0  Grant  7 : --          Grant 12 : --        
      Grant  3 : --          Grant  8 : --          Grant 13 : --        
      Grant  4 : --          Grant  9 : --        

    Cortex-M MPU
      Region 0: [0x20004000:0x20005000], length: 4096 bytes; ReadWrite (0x3)
        Sub-region 0: [0x20004000:0x20004200], Enabled
        Sub-region 1: [0x20004200:0x20004400], Enabled
        Sub-region 2: [0x20004400:0x20004600], Enabled
        Sub-region 3: [0x20004600:0x20004800], Enabled
        Sub-region 4: [0x20004800:0x20004A00], Enabled
        Sub-region 5: [0x20004A00:0x20004C00], Disabled
        Sub-region 6: [0x20004C00:0x20004E00], Disabled
        Sub-region 7: [0x20004E00:0x20005000], Disabled
      Region 1: Unused
      Region 2: [0x00040000:0x00040800], length: 2048 bytes; UnprivilegedReadOnly (0x2)
        Sub-region 0: [0x00040000:0x00040100], Enabled
        Sub-region 1: [0x00040100:0x00040200], Enabled
        Sub-region 2: [0x00040200:0x00040300], Enabled
        Sub-region 3: [0x00040300:0x00040400], Enabled
        Sub-region 4: [0x00040400:0x00040500], Enabled
        Sub-region 5: [0x00040500:0x00040600], Enabled
        Sub-region 6: [0x00040600:0x00040700], Enabled
        Sub-region 7: [0x00040700:0x00040800], Enabled
      Region 3: Unused
      Region 4: Unused
      Region 5: Unused
      Region 6: Unused
      Region 7: Unused

    To debug, run `make debug RAM_START=0x20004000 FLASH_INIT=0x4005d`
    in the app's folder and open the .lst file.

    𝐀𝐩𝐩: c_hello   -   [Yielded]
    Events Queued: 0   Syscall Count: 8   Dropped Upcall Count: 0
    Restart Count: 0
    Last Syscall: Yield { which: 1, address: 0x0 }
    Completion Code: None


    ╔═══════════╤══════════════════════════════════════════╗
    ║  Address  │ Region Name    Used | Allocated (bytes)  ║
    ╚0x20008000═╪══════════════════════════════════════════╝
                │ Grant Ptrs      112
                │ Upcalls         320
                │ Process         920
      0x20007AB8 ┼───────────────────────────────────────────
                │ ▼ Grant          76
      0x20007A6C ┼───────────────────────────────────────────
                │ Unused
      0x20006A04 ┼───────────────────────────────────────────
                │ ▲ Heap            0 |   4200               S
      0x20006A04 ┼─────────────────────────────────────────── R
                │ Data            516 |    516               A
      0x20006800 ┼─────────────────────────────────────────── M
                │ ▼ Stack         128 |   2048          
      0x20006780 ┼───────────────────────────────────────────
                │ Unused
      0x20006000 ┴───────────────────────────────────────────
                .....
      0x00041000 ┬─────────────────────────────────────────── F
                │ App Flash      1996                        L
      0x00040834 ┼─────────────────────────────────────────── A
                │ Protected        52                        S
      0x00040800 ┴─────────────────────────────────────────── H

      R0 : 0x00000001    R6 : 0x00040C50
      R1 : 0x00000000    R7 : 0x20006000
      R2 : 0x00000000    R8 : 0x00000000
      R3 : 0x00000000    R10: 0x00000000
      R4 : 0x00040834    R11: 0x00000000
      R5 : 0x20006000    R12: 0x200067F0
      R9 : 0x20006800 (Static Base Register)
      SP : 0x200067D0 (Process Stack Pointer)
      LR : 0x00040A3F
      PC : 0x00040A46
    YPC : 0x00040A46

    APSR: N 0 Z 0 C 1 V 0 Q 0
          GE 0 0 0 0
    EPSR: ICI.IT 0x00
          ThumbBit true 

    Total number of grant regions defined: 14
      Grant  0 : --          Grant  5 : --          Grant 10 : --        
      Grant  1 : --          Grant  6 : --          Grant 11 : --        
      Grant  2 : --          Grant  7 : --          Grant 12 : --        
      Grant  3 : --          Grant  8 : --          Grant 13 : --        
      Grant  4 0x1: 0x20007a6c  Grant  9 : --        

    Cortex-M MPU
      Region 0: [0x20006000:0x20007000], length: 4096 bytes; ReadWrite (0x3)
        Sub-region 0: [0x20006000:0x20006200], Enabled
        Sub-region 1: [0x20006200:0x20006400], Enabled
        Sub-region 2: [0x20006400:0x20006600], Enabled
        Sub-region 3: [0x20006600:0x20006800], Enabled
        Sub-region 4: [0x20006800:0x20006A00], Enabled
        Sub-region 5: [0x20006A00:0x20006C00], Enabled
        Sub-region 6: [0x20006C00:0x20006E00], Disabled
        Sub-region 7: [0x20006E00:0x20007000], Disabled
      Region 1: Unused
      Region 2: [0x00040800:0x00041000], length: 2048 bytes; UnprivilegedReadOnly (0x2)
        Sub-region 0: [0x00040800:0x00040900], Enabled
        Sub-region 1: [0x00040900:0x00040A00], Enabled
        Sub-region 2: [0x00040A00:0x00040B00], Enabled
        Sub-region 3: [0x00040B00:0x00040C00], Enabled
        Sub-region 4: [0x00040C00:0x00040D00], Enabled
        Sub-region 5: [0x00040D00:0x00040E00], Enabled
        Sub-region 6: [0x00040E00:0x00040F00], Enabled
        Sub-region 7: [0x00040F00:0x00041000], Enabled
      Region 3: Unused
      Region 4: Unused
      Region 5: Unused
      Region 6: Unused
      Region 7: Unused

    To debug, run `make debug RAM_START=0x20006000 FLASH_INIT=0x4085d`
    in the app's folder and open the .lst file.
```

### `reset`
  - You can also reset the board with the `reset` command:

```text
    tock$ reset
```

### `kernel`
  - You can view the kernel memory map with the `kernel` command:

```text
    tock$ kernel
    Kernel version: 2.1 (build 899d73cdd)

    ╔═══════════╤══════════════════════════════╗
    ║  Address  │ Region Name    Used (bytes)  ║
    ╚0x20003DAC═╪══════════════════════════════╝
                │   BSS         11692
      0x20001000 ┼─────────────────────────────── S
                │   Relocate        0            R
      0x20001000 ┼─────────────────────────────── A
                │ ▼ Stack        4096            M
      0x20000000 ┼───────────────────────────────
                .....
      0x0001A000 ┼─────────────────────────────── F
                │   RoData      27652            L
      0x000133FC ┼─────────────────────────────── A
                │   Code        78844            S
      0x00000000 ┼─────────────────────────────── H

```
### `process`
  - You can also view the memory map for a process with the `process` command:

```text
    tock$ process c_hello
    𝐀𝐩𝐩: c_hello   -   [Yielded]
    Events Queued: 0   Syscall Count: 8   Dropped Upcall Count: 0
    Restart Count: 0
    Last Syscall: Yield { which: 1, address: 0x0 }
    Completion Code: None


    ╔═══════════╤══════════════════════════════════════════╗
    ║  Address  │ Region Name    Used | Allocated (bytes)  ║
    ╚0x20008000═╪══════════════════════════════════════════╝
                │ Grant Ptrs      112
                │ Upcalls         320
                │ Process         920
      0x20007AB8 ┼───────────────────────────────────────────
                │ ▼ Grant          76
      0x20007A6C ┼───────────────────────────────────────────
                │ Unused
      0x20006A04 ┼───────────────────────────────────────────
                │ ▲ Heap            0 |   4200               S
      0x20006A04 ┼─────────────────────────────────────────── R
                │ Data            516 |    516               A
      0x20006800 ┼─────────────────────────────────────────── M
                │ ▼ Stack         128 |   2048          
      0x20006780 ┼───────────────────────────────────────────
                │ Unused
      0x20006000 ┴───────────────────────────────────────────
                .....
      0x00041000 ┬─────────────────────────────────────────── F
                │ App Flash      1996                        L
      0x00040834 ┼─────────────────────────────────────────── A
                │ Protected        52                        S
      0x00040800 ┴─────────────────────────────────────────── H

```

### `commands history`
 - You can use the up and down arrows to scroll through the command history and to view the previous commands you have run.
 - If you inserted more commands than the command history can hold, oldest commands will be overwritten.
 - You can view the commands in bidirectional order, `up arrow` for oldest commands and `down arrow` for newest.
 - If the user custom size for the history is set to `0`, the history will be disabled and the rust compiler will be able to optimize the binary file by removing dead code.
 - If you are typing a command and accidentally press the `up arrow` key, you can press `down arrow` in order to retrieve the command you were typing.
 - If you scroll through the history and you want to edit a command and accidentally press the `up` or `down` arrow key, scroll to the bottom of the history and you will get back to the command you were typing.

  Here is how to add a custom size for the command `history` used by the ProcessConsole structure to keep track of the typed commands, in the `main.rs` of boards:
 ```rust
 const COMMAND_HISTORY_LEN : usize = 30;

 /// ...
 
 pub struct Platform {
    // other fields
    
    pconsole: &'static capsules::process_console::ProcessConsole<
        'static,
        COMMAND_HISTORY_LEN,
        // or { capsules::process_console::DEFAULT_COMMAND_HISTORY_LEN }
        // for the deafult behaviour
        VirtualMuxAlarm<'static, nrf52840::rtc::Rtc<'static>>,
        components::process_console::Capability,
    >,
    
    // other fields
}

  /// ...

  let _process_console = components::process_console::ProcessConsoleComponent::new(
          board_kernel,
          uart_mux,
          mux_alarm,
          process_printer,
          Some(reset_function),
      )
      .finalize(components::process_console_component_static!(
          nrf52833::rtc::Rtc,
          COMMAND_HISTORY_LEN // or nothing for the default behaviour
      ));

  /// ...
 ```
> Note: In order to disable any functionality for the command history set the `COMMAND_HISTORY_LEN` to `0` or `1` (the history will be disabled for a size of `1`, because the first position from the command history is reserved for accidents by pressing `up` or `down` arrow key.
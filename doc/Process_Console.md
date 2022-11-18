 Process Console
 ===============
 
 `process_console` is a capsule that implements a text console over the UART that allows
 a terminal to inspect and control userspace processes.

 Protocol
 --------

 This module provides a simple text-based console to inspect and control
 which processes are running. The console has eleven commands:
  - `help` - prints the available commands and arguments
  - `status` - prints the current system status
  - `list` - lists the current processes with their IDs and running state
  - `stop n` - stops the process with name n
  - `start n` - starts the stopped process with name n
  - `fault n` - forces the process with name n into a fault state
  - `terminate n` - terminates the running process with name n, moving to the Terminated state
  - `boot n` - tries to boot a Terminated process with name n
  - `panic` - causes the kernel to run the panic handler
  - `process n` - prints the memory map of process with name n
  - `kernel` - prints the kernel memory map

 ### `list` Command Fields:

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

 Setup
 -----

 You need a device that provides the `hil::uart::UART` trait. This code
 connects a `ProcessConsole` directly up to USART0:

 ```rust
 # use kernel::{capabilities, hil, static_init};
 # use capsules::process_console::ProcessConsole;

 pub struct Capability;
 unsafe impl capabilities::ProcessManagementCapability for Capability {}

 let pconsole = static_init!(
     ProcessConsole<usart::USART>,
     ProcessConsole::new(&usart::USART0,
                  115200,
                  &mut console::WRITE_BUF,
                  &mut console::READ_BUF,
                  &mut console::COMMAND_BUF,
                  kernel,
                  Capability));
 hil::uart::UART::set_client(&usart::USART0, pconsole);

 pconsole.start();
 ```

 Using ProcessConsole
 --------------------

 With this capsule properly added to a board's `main.rs` and that kernel
 loaded to the board, make sure there is a serial connection to the board.
 Likely, this just means connecting a USB cable from a computer to the board.
 Next, establish a serial console connection to the board. An easy way to do
 this is to run:

 ```shell
 $ tockloader listen
 ```

 With that console open, you can issue commands. For example, to see all of
 the processes on the board, use `list`:

 ```text
 $ tockloader listen
 Using "/dev/cu.usbserial-c098e513000c - Hail IoT Module - TockOS"

 Listening for serial output.
 Initialization complete. Entering main loop
 Hello World!
 list
 PID    Name    Quanta  Syscalls  Restarts Grants  State
 00     blink        0       113         0  1/12   Yielded
 01     c_hello      0         8         0  3/12   Yielded
 ```

 To get the welcome message, use the `help` command:
 ```text
 help
 Welcome to the process console.
 Valid commands are: help status list stop start fault boot terminate process kernel panic
 ```

 To get a general view of the system, use the `status` command:

 ```text
 status
 Total processes: 2
 Active processes: 2
 Timeslice expirations: 0
 ```

 You can control processes with the `start` and `stop` commands:

 ```text
 stop blink
 Process blink stopped
 ```

 To force a process into a fault state, you should use the `fault` command:

```text
fault blink
Process blink now faulted
```

You can change the Termination status with `terminate` and `boot` commands:

```text
terminate blink
Process blink terminated
```

You can also force a kernel panic with the `panic` command:

```text
panic
panicked at 'Process Console forced a kernel panic.', capsules/src/process_console.rs:728:13
	Kernel version 41623783b

---| Debug buffer not empty. Flushing. May repeat some of last message(s):
Entering main loop.

---| No debug queue found. You can set it with the DebugQueue component.

---| RISC-V Machine State |---
Last cause (mcause): Machine timer interrupt (interrupt=1, exception code=0x00000007)
Last value (mtval):  0x00000000

System register dump:
 mepc:    0x8000ACC0    mstatus:     0x00000088
 mcycle:  0xEAA35783    minstret:    0xEAA38155
 mtvec:   0x80000100
 mstatus: 0x00000088
  uie:    false  upie:   false
  sie:    false  spie:   false
  mie:    true   mpie:   true
  spp:    false
 mie:   0x00000808   mip:   0x00000080
  usoft:  false               false 
  ssoft:  false               false 
  msoft:  true                false 
  utimer: false               false 
  stimer: false               false 
  mtimer: false               true  
  uext:   false               false 
  sext:   false               false 
  mext:   true                false 
```

You can view the kernel memory map with the `kernel` command:

```text
kernel
Kernel version: 2.1 (build 41623783b)
tock$ 
 ╔═══════════╤══════════════════════════════╗
 ║  Address  │ Region Name    Used (bytes)  ║
 ╚0x80202D10═╪══════════════════════════════╝
             │   BSS          3328
  0x8020200C ┼─────────────────────────────── S
             │   Relocate       12            R
  0x80202000 ┼─────────────────────────────── A
             │ ▼ Stack        8192            M
  0x80200000 ┼───────────────────────────────
             .....
  0x80012B5C ┼─────────────────────────────── F
             │   RoData      20730            L
  0x8000DA62 ┼─────────────────────────────── A
             │   Code        55906            S
  0x80000000 ┼─────────────────────────────── H
```
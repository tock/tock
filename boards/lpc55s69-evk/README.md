NXP LPCXpresso55S69 Development Board
======================================

For more details about the board [visit the NXP board website](https://www.nxp.com/design/design-center/software/development-software/mcuxpresso-software-and-tools-/lpcxpresso-boards/lpcxpresso55s69-development-board:LPC55S69-EVK). More details about the chip can be found [here](https://www.nxp.com/products/LPC55S6x)

## Flashing the kernel

### Prerequisites

The primary tool used for flashing this board is `LinkServer`. You will need to install it. The recommended way is using the [NXP official website](https://www.nxp.com/design/design-center/software/development-software/mcuxpresso-software-and-tools-/linkserver-for-microcontrollers:LINKERSERVER).

Add to PATH: Ensure the LinkServer executable is in your system `PATH`.

To verify the installation, run:

```bash
$ LinkServer --version
```

### Compiling and Flashing

To compile the Tock kernel and flash it to the board, navigate to the `boards/lpc55s69-evk` directory and run `make flash-debug` or `make flash`. 

```bash
# Flash the Debug build
$ make debug

# Flash the Release build
$ make install
```

The Makefile is configured to auto-detect your OS (Windows/Linux) and the first connected probe. If you have multiple probes connected, you can specify one by ID:

```bash
$ make debug PROBE_ID="#2"
```

The expected output of the command is looking like this:

```bash
$ make flash-debug
INFO: Selected device LPC55S69:LPCXpresso55S69
INFO: Selected probe #1 OSA0AQJR (LPC-LINK2 CMSIS-DAP V5.224)
Ns: LinkServer RedlinkMulti Driver v25.12 (Dec  8 2025 18:34:00 - crt_emu_cm_redlink.exe build 1160)
Pc: (  0) Reading remote configuration
Wc(03). No cache support.
Nc: Found generic directory XML file in C:\Users\User\AppData\Local\Temp\tmpbupiyfhi\crt_directory.xml
Pc: (  5) Remote configuration complete
Nc: Reconnected to existing LinkServer process.
Nc: Probe Firmware: LPC-LINK2 CMSIS-DAP V5.224 (NXP Semiconductors)
Nc: Serial Number:  OSA0AQJR
Nc: VID:PID:  1FC9:0090
Nc: USB Path: \\?\hid#vid_1fc9&pid_0090&mi_00#7&341d2066&0&0000#{4d1e55b2-f16f-11cf-88cb-001111000030}
Nc: Using memory from core 0 after searching for a good core
Pc: ( 30) Emulator Connected
Nc: processor is in secure mode
Pc: ( 40) Debug Halt
Pc: ( 50) CPU ID
Nc: debug interface type      = CoreSight DP (DAP DP ID 6BA02477) over SWD TAP 0
Nc: processor type            = Cortex-M33 (CPU ID 00000D21) on DAP AP 0
Nc: number of h/w breakpoints = 8
Nc: number of flash patches   = 0
Nc: number of h/w watchpoints = 4
Nc: Probe(0): Connected&Reset. DpID: 6BA02477. CpuID: 00000D21. Info: <None>
Nc: Debug protocol: SWD. RTCK: Disabled. Vector catch: Disabled.
Ns: Content of CoreSight Debug ROM(s):
Nc: RBASE E00FE000: CID B105100D PID 0000095000 ROM (type 0x1)
Nc: ROM 1 E00FF000: CID B105100D PID 04000BB4C9 ROM (type 0x1)
Nc: ROM 2 E000E000: CID B105900D PID 04000BBD21 CSt ARM ARMv8-M type 0x0 Misc - Undefined
Nc: ROM 2 E0001000: CID B105900D PID 04000BBD21 CSt ARM DWTv2 type 0x0 Misc - Undefined
Nc: ROM 2 E0002000: CID B105900D PID 04000BBD21 CSt ARM FPBv2 type 0x0 Misc - Undefined
Nc: ROM 2 E0000000: CID B105900D PID 04000BBD21 CSt ARM ITMv2 type 0x43 Trace Source - Bus
Nc: ROM 1 E0040000: CID B105900D PID 04000BBD21 CSt type 0x11 Trace Sink - TPIU
Nc: NXP: LPC55S69
Nc: DAP stride is 1024 bytes (256 words)
Nc: Inspected v.2 On chip Flash memory using IAP lib LPC55xx.cfx
Nc: Image 'LPC55xx Dec  8 2025 17:21:59'
Nc: Opening flash driver LPC55xx.cfx
Nc: VECTRESET requested, but not supported on ARMv8-M CPUs. Using SOFTRESET instead.
Nc: Using SOFT reset to run the flash driver
Nc: Flash variant 'LPC55xx (630KB)' detected (630KB = 1260*512 at 0x0)
Nc: Closing flash driver LPC55xx.cfx
Nc: Inspected v.2 On chip Flash memory using IAP lib LPC55xx_S.cfx
Nc: Image 'LPC55xx (Secure) Dec  8 2025 17:21:35'
Nc: Opening flash driver LPC55xx_S.cfx
Nc: VECTRESET requested, but not supported on ARMv8-M CPUs. Using SOFTRESET instead.
Nc: Using SOFT reset to run the flash driver
Nc: Flash variant 'LPC55xx (630KB) (Secure)' detected (630KB = 1260*512 at 0x10000000)
Nc: Closing flash driver LPC55xx_S.cfx
Pc: ( 65) Chip Setup Complete
Pc: ( 70) License Check Complete
Nt: Loading 'tmp60vf72mz.axf' ELF 0x00000000 len 0x19000
Nc: Opening flash driver LPC55xx.cfx
Nc: VECTRESET requested, but not supported on ARMv8-M CPUs. Using SOFTRESET instead.
Nc: Using SOFT reset to run the flash driver
Nc: Flash variant 'LPC55xx (630KB)' detected (630KB = 1260*512 at 0x0)
Pb: 1 of 1 (  0) Writing sectors 0-199 at 0x00000000 with 102400 bytes
Ps: (  0) at 00000000: 0 bytes - 0/102400
Ps: ( 97) at 00000000: 99328 bytes - 99328/102400
Wc: op BlankCheck (0x18400, 0x0, 0x6) status 0x1 - driver reported driver error - INTIAP driver rc 105 (0x69)
Ps: (100) at 00018400: 3072 bytes - 102400/102400
Nc: 00019000 done 100% (102400 out of 102400)
Nc: Sectors written: 6, unchanged: 194, total: 200
Nc: Closing flash driver LPC55xx.cfx
Pb: (100) Finished writing Flash successfully.
Nt: Loaded 0x19000 bytes in 538ms (about 190kB/s)
Nt: Loading 'tmp60vf72mz.axf' ELF 0x0001FFD4 len 0x2C
Nc: Opening flash driver LPC55xx.cfx (already resident)
Nc: VECTRESET requested, but not supported on ARMv8-M CPUs. Using SOFTRESET instead.
Nc: Using SOFT reset to run the flash driver
Nc: Flash variant 'LPC55xx (630KB)' detected (630KB = 1260*512 at 0x0)
Pb: 1 of 1 (  0) Writing sectors 255-255 at 0x0001FFD4 with 44 bytes
Ps: (  0) at 0001FE00: 0 bytes - 0/512
Ps: (100) at 0001FE00: 512 bytes - 512/512
Nc: Sectors written: 0, unchanged: 1, total: 1
Nc: Closing flash driver LPC55xx.cfx
Pb: (100) Finished writing Flash successfully.
Nt: Loaded 0x2C bytes in 207ms (about 0kB/s)
Nt: Reset target (system)
Nc: Starting execution using system reset with a stall address
Nc: Retask read watchpoint 1 at 0x50000040 to use for boot ROM stall
Nc: Boot ROM stalled accessing address 0x50000040 (restoring watchpoint 1)
Ns: Stopped (Was Reset)  [Reset from Unknown]: Watchpoint (Temp) #1 - read watchpoint at 0x50000040 Data watch for bootloader stall

Ns: restart on reset
```

### Flashing app

Applications are built separately from the main project. After building an application execute:
```bash
tockloader install
```

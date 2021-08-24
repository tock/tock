New in 2.0
==========

* Many core kernel APIs have been redesigned and rewritten.

  - There are two new userspace system calls: `AllowReadOnly` and `Exit`. The
    old `Allow` system call has been renamed to `AllowReadWrite`.
    `AllowReadOnly` provides a mechanism for userspace to share a read-only
    buffer (e.g., constant data stored in flash) to the kernel. `Exit` allows a
    process to complete execution and request that the kernel either terminate
    or restart it.

  - The system call ABI has been rewritten. System calls can now return up to 4
    registers of values to userspace on return. The ABI defines the format and
    structure of the allowed return types. TRD104 documents the new ABI.

  - The calling semantics and requirements for system calls is more clearly
    defined, especially with respect to calls to the Allow system calls and how
    buffers are managed. Furthermore, the lifetime of upcalls passed to the
    kernel with the `subscribe` system call has been defined. To enforce that
    the kernel doesn't maintain references to upcalls that it shouldn't (so
    userspace can reclaim any resources they require), upcalls are now managed
    by the core kernel. The changes to these calling semantics are documented in
    TRD104.

  - Several types in the kernel have changed names, to better reflect their
    actual use and behavior.

	  - `AppSlice` is now `ReadOnlyProcessBuffer` and `ReadWriteProcessBuffer`.

    - `Callback` is now `Upcall` (to distinguish upcalls from the kernel to
      userspace from general softare callbacks). `Upcall`s are now stored in a
      special block of memory in grant regions and are managed by the kernel
      rather than drivers. This allows the kernel to enforce their swapping
      semantics. #2639

    - `Platform` is now `SyscallDriverLookup` and `Chip` is now split into
      `Chip` for chip-specific operations and `KernelResources` for kernel
      operations.

    - `Driver` is now `SyscallDriver`.

* The kernel namespace has been reorganized.

  - https://github.com/tock/tock/pull/2659 reorganizes the kernel namespace. The
    actual abstractions and types exported were not changed, but their places in
    the namespace were.

  - Almost everything is now exported as `kernel::module::Type` rather than
    `kernel::Type`.

  - `/common` is split up into `/utilities` and `/collections`

* There is increased chip and board support.

  - RISC-V support has been extended to support progress and revisions to
    support microcontroller-type systems, including support for EPMP memory
    protection.

  - There is support for ARM CortexM0+ and CortexM7.

  - Board support adds:

	  - Nano RP2040 Connect
	  - Clue nRF52840
	  - BBC Micro:bit v2
	  - WeAct F401CCU6 Core Board
    - i.MX RT 1052 Evaluation Kit
	  - Teensy 4.0
	  - Pico Explorer Base
	  - Rapsberry Pi Pico
	  - LiteX on Digilent Arty A-7
	  - Verilated LiteX simulation
	  - ESP32-C3-DevKitM-1


* Major HIL changes

  - All HILs have changed significantly, to be in line with the new types within
    the kernel.

  - `ReturnCode` has been removed from the kernel. HILs that used to return
    `ReturnCode` now return `Result<(), ErrorCode>`, so that `Ok` indicates a
    success result. #2508

  - There is a draft of a TRD describing guidelines for HIL design, which
    enumerates 13 principles HIL traits should follow.

  - The SPI, I2C, and CRC HILs have changed in how they handle buffers. SPI and
    I2C now correctly return buffers on error cases, and CRC now relies on
    `LeasableBuffer` to compute a CRC over a large block of memory.

  - Digest has been extended to support multiple digest algorithms: in addition
    to HMAC SHA256 it now supports SHA224, SHA256, SHA384, SHA512, HMAC SHA384
    and HMAC SHA512.

  - The Time HIL has been updated to better support `dyn` references when
    needed, by adding a `ConvertTicks` trait. This change is documented in TRD
    105 (which, when finalized, obsoletes 101).

  - Blanket implementations for UART trait groups have been added. Now, if a
    structure implements both `uart::Transmit` and `uart::Receive`, it will
    automatically implement `uart::UartData`.

  - New HILs added:

    - key/value store
    - 8080 bus (for LCDs)
    - text screen
    - screen
    - touch

* In-kernel virtualizers for the following HILs have been added: AES, RNG, SHA

* The kernel now checks whether loaded processes are compiled for the running
  kernel version. Because 2.0 changes the user/kernel ABI, processes compiled
  for Tock 1.x will not run correctly on a Tock 2.x kernel and vice versa. If
  the kernel detects that a process is compiled for the wrong kernel version it
  stops loading processes.

* There have been changes to kernel internals and the build system to reduce
  code size. For example, kernel code that was highly replicated in
  monomorphized functions has been factored out (#2648).

* All system call driver capsules that do not support use by multiple processes
  now use grant regions to store state and explicitly forbid access from
  multiple processes (e.g., #2518).

* The process console has been improved and can now display memory maps for the
  kernel and processes.

* Added `tools/stack_analysis.sh` and `make stack-analysis` for analyzing stack
  size.

* Improvements to `tools/print_tock_memory_usage.sh` for displaying code size.

* Transitioned uses of deprecated `llvm_asm!()` to `asm!()` macro for better
  compile-time checking (#2449, #2363).

* Make it possible for boards to avoid using code space for peripherals they do
  not use (e.g., #2069).

* Bug fixes.


New in 1.5
==========

* Major HIL Changes

  None

* Loading and Restarting Processes Improvements

  Processes can now fault and be restarted by the kernel, and
  [#1565](https://github.com/tock/tock/pull/1565) allows a board configuration
  file to specify the restart policy that the kernel should use.

  Process discovery, parsing, and creation was also overhauled in
  [#1480](https://github.com/tock/tock/pull/1480) to remove `unsafe` from the
  TBF header parsing code. This allows `process::load_processes()` to return
  errors if process loading fails. Boards now need to handle the `Result` return
  type.


New in 1.4
==========

* Major HIL Changes

  Three HILs have been revised to better support embedded devices and clean up
  the interface for users of the HILs.

  - [#1211](https://github.com/tock/tock/pull/1211) revamps the UART interface
    to separate the transmit and receive paths.

  - [#1297](https://github.com/tock/tock/pull/1297) breaks the GPIO HIL into
    component subtraits so GPIO users can be specific about the features they
    need from GPIO pins.

  - [#1345](https://github.com/tock/tock/pull/1345) clearly defines the
    differences between counters, alarms, and timers.

* Start on RISC-V Support

  [#1323](https://github.com/tock/tock/pull/1317),
  [#1323](https://github.com/tock/tock/pull/1323), and
  [#1345](https://github.com/tock/tock/pull/1345) add architecture support and
  boards to Tock for the RISC-V architecture.

* Update Userland-Kernel Boundary Interface

  [#1318](https://github.com/tock/tock/pull/1318) updates the interface for
  switching to and returning from userspace to be less Cortex-M specific. The
  functions are more general and do not assume values are passed on the stack.


New in 1.2
==========

* Kernel debug module

  - [#1036](https://github.com/tock/tock/pull/1036),
    [#1029](https://github.com/tock/tock/pull/1029), and
    [#997](https://github.com/tock/tock/pull/997) change `debug::panic`'s
    signature. First, instead of taking a single LED, `panic` takes a slice of LEDs
    as its first argument. Second, the Rust now uses a `PanicInfo` struct to pass
    along information about where a panic occured, and `debug::panic` adopts the
    same structure. Third, architecture specific assembly code was removed
    from the kernel crate (including the debug module), requiring `debug::panic` to
    take in a particlar implementation of the `nop` instruction. Finally,
    `debug::panic` takes a reference to the process array (it is permissible to
    pass an empty array instead, but you won't get any information about process
    state on panic).

    Boards most likely call `debug::panic` from their `panic_fmt` function:

    ```rust
    #[lang = "panic_fmt"]
    pub unsafe extern "C" fn panic_fmt(args: Arguments, file: &'static str, line: u32) -> ! {
            let led = ...;
            let writer = ...;
            debug::panic(led, writer, args, file, line)
    }
    ```

    should now be:
    ```rust
    use core::panic::PanicInfo;
    ...
    #[panic_implementation]
    pub unsafe extern "C" fn panic_fmt(pi: &PanicInfo) -> ! {[lang = "panic_fmt"]
        let led = ...;
        let writer = ...;

        debug::panic(&mut [led], writer, pi, &cortexm4::support::nop, &PROCESSES)
    ```

  - [#1046](https://github.com/tock/tock/pull/1046) changes how the debug module
    in the kernel crate is structured. Instead of being a pseudo-process, debug
    is now treated more like a capsule, and needs a UART object to be passed to
    it. This means that `main.rs` needs to be updated to correctly set this up.

    First, if the debug UART bus is shared with console (or anything else), and
    this is likely the case, then a UART mux needs to be created. This is going to
    look slightly different depending on the underlying MCU, but for the SAM4L
    this looks like:

    ```rust
    let uart_mux = static_init!(
        MuxUart<'static>,
        MuxUart::new(
            &sam4l::usart::USART0, // Choose the correct UART HW bus
            &mut capsules::virtual_uart::RX_BUF,
            115200
        )
    );
    hil::uart::UART::set_client(&sam4l::usart::USART0, uart_mux);
    ```

    With the mux created, a user of the UART bus must be defined. This will be
    passed to the debug module.

    ```rust
    let debugger_uart = static_init!(UartDevice, UartDevice::new(uart_mux, false));
    debugger_uart.setup();
    ```

    The following is the actual debug module, and must be created to use the
    `debug!()` macro. If debug is sharing a UART bus then the above mux and
    device is necessary, but if it is on a dedicated UART bus then that UART
    module can be passed in here instead.

    ```rust
    let debugger = static_init!(
        kernel::debug::DebugWriter,
        kernel::debug::DebugWriter::new(
            debugger_uart, // Replace with just a HW UART if no sharing is needed.
            &mut kernel::debug::OUTPUT_BUF,
            &mut kernel::debug::INTERNAL_BUF,
        )
    );
    hil::uart::UART::set_client(debugger_uart, debugger);
    ```

    Finally, to get around Rust sharing rules, we need to create this wrapper:

    ```rust
    let debug_wrapper = static_init!(
        kernel::debug::DebugWriterWrapper,
        kernel::debug::DebugWriterWrapper::new(debugger)
    );
    kernel::debug::set_debug_writer_wrapper(debug_wrapper);
    ```



* Reorganization of the kernel crate: The kernel crate has been restructured to
enable many improvements to Tock, and to move to a more consistent design between
the kernel crate and other parts of Tock. This change has happened through several
pull requests:
[#975](https://github.com/tock/tock/pull/975),
[#1044](https://github.com/tock/tock/pull/1044),
[#1109](https://github.com/tock/tock/pull/1109),
[#1111](https://github.com/tock/tock/pull/1111),
[#1113](https://github.com/tock/tock/pull/1113),
[#1115](https://github.com/tock/tock/pull/1115),
[#1171](https://github.com/tock/tock/pull/1171), and
[#1191](https://github.com/tock/tock/pull/1191).

    The primary motivation for this is
    making the kernel crate architecture agnostic, so that Tock can be ported
    non Cortex-M platforms ([#985](https://github.com/tock/tock/issues/985)).

    A part of this reorganization is the introduction of Capabilities, or
    a compile-time access control mechanism in Tock based on being able to
    forbid `unsafe` code. Capabilities restrict what code in Tock can call
    certain sensitive functions, like `load_processes()`.


  - The `Chip` in main.rs has to be instantiated with `static_init!` to
    ensure it has a long enough lifetime. Now:

    ```rust
    let chip = static_init!(sam4l::chip::Sam4l, sam4l::chip::Sam4l::new());
    ```

  - Capabilities need to be created. Creating a capability requires the ability
    to call `unsafe`, so capsules cannot create capabilities, and instead must
    be passed the capability if they need access to protected functions.

    ```rust
    let process_management_capability =
        create_capability!(capabilities::ProcessManagementCapability);
    let main_loop_capability = create_capability!(capabilities::MainLoopCapability);
    ```

  - There is now a `Kernel` struct that needs to be instantiated by the board.
    `Kernel` has a method for the kernel's main loop, instead of a global
    function in the kernel's base module. Board configurations (i.e. each
    board's `main.rs`) as a result need to instantiate a statically allocate
    this new struct.

    ```rust
    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(&PROCESSES));

    board_kernel.kernel_loop(&hail, chip, Some(&hail.ipc), &main_loop_capability);
    ```

  - `load_processes` takes the `Kernel` struct as an additional first argument,
    the `chip` as a new second argument, and the required capability as the
    last argument. Creating a `Process` (which `load_processes()` does) requires
    a reference to the chip because the process object needs to have access to
    architecture-specific context switching functions, as well as chip-specific
    MPU functions.

    ```rust
     kernel::procs::load_processes(
        board_kernel,
        chip,
        &_sapps as *const u8,
        &mut APP_MEMORY,
        &mut PROCESSES,
        FAULT_RESPONSE,
        &process_management_capability,
    );
    ```

  - Creating a grant requires a capability, as not just any code should be able
    to allocate memory in the grant regions.

    ```rust
    let memory_allocation_cap = create_capability!(capabilities::MemoryAllocationCapability);
    ```

    To use:

    ```rust
    board_kernel.create_grant(&memory_allocation_cap)
    ```

    Creating grants is now handled through the main `Kernel` struct so that it
    can check that no grants are created after processes are setup in memory,
    since grants require space allocated in process memory.



* [#1032](https://github.com/tock/tock/pull/1032) updates the ADC HIL to
  explicitly specify the resolution of the sample and to clarify that samples
  are always left-aligned in the `u16` buffer. Previously, the only ADC
  implementation happened to be 12 bits and left-aligned, which callers only
  assumed. It also added a method to (if possible) report the reference
  voltage, which can be used to convert raw ADC samples to absolute voltages.
  Implementers of the ADC HIL must implement two new methods:

  ```rust
  /// Function to ask the ADC how many bits of resolution are in the samples
  /// it is returning.
  fn get_resolution_bits(&self) -> usize;

  /// Function to ask the ADC what reference voltage it used when taking the
  /// samples. This allows the user of this interface to calculate an actual
  /// voltage from the ADC reading.
  ///
  /// The returned reference voltage is in millivolts, or `None` if unknown.
  fn get_voltage_reference_mv(&self) -> Option<usize>;
  ```

* UART HIL Refinements: This release saw several updates to the UART HIL,
  summarized in the [UART HIL tracking issue](https://github.com/tock/tock/issues/1072).

  - [#1073](https://github.com/tock/tock/pull/1073) removes `initialize` from
    the UART HIL. Implementations will need to disentangle board-specific
    initialization code, such as enabling the peripheral or assigning pins,
    from UART configuration code, such as baud rate or parity. Initialization
    is no longer part of the UART HIL and should be performed by the top-level
    board before passing the UART object to any other code. UART configuration
    is now controlled by the new `configure` HIL method:

    ```rust
    /// Configure UART
    ///
    /// Returns SUCCESS, or
    ///
    /// - EOFF: The underlying hardware is currently not available, perhaps
    ///         because it has not been initialized or in the case of a shared
    ///         hardware USART controller because it is set up for SPI.
    /// - EINVAL: Impossible parameters (e.g. a `baud_rate` of 0)
    /// - ENOSUPPORT: The underlying UART cannot satisfy this configuration.
    fn configure(&self, params: UARTParameters) -> ReturnCode;
    ```

* [#1145](https://github.com/tock/tock/pull/1145) rewrites the HILs
  for random number generation. There are now two HILs, `entropy` and
  `rng` (random number generation).  They differ in the guarantees they give
  about the bits they produce. The `entropy` traits guarantee high entropy bits:
  1 bit of entropy per bit generated, such that every bit generated has an
  equal chance of being 0 or 1 and is independent of any other bit produced
  by the trait: that observing the stream of bits provides zero
  information on what the future bits will be. Entropy's guarantees make
  it suitable for use in security and cryptography. The `rng` traits
  provide bits that are assured to satisfy all standard NIST randomness
  tests, but do not promise that future bits cannot be guessed from
  past ones. E.g., the bits are random but not robust against an adversary.

  It also adds library components for converting between different entropy
  sources as well as converting an entropy source into a random number
  generator (but *not* a random number generator into an entropy source!).
  Any software that needs entropy for security or cryptography should use
  an `entropy` trait and not an `rng` trait.

* Updates to linker and toolchain: As of
  [#993](https://github.com/tock/tock/pull/993) and
  [#1031](https://github.com/tock/tock/pull/1031), the Tock kernel no longer
  requires GCC for compilation, and entirely uses the LLVM toolchain.


  - Boards now need to explicitly define room for the kernel stack. Something
    like the following should be in the board's main.rs:

    ```rust
    /// Dummy buffer that causes the linker to reserve enough space for the stack.
    #[no_mangle]
    #[link_section = ".stack_buffer"]
    pub static mut STACK_MEMORY: [u8; 0x1000] = [0; 0x1000];
    ```

  - There are numerous changes to the shared board linker file. Individual boards
    need to be updated to not use variables, and instead define the entire `MEMORY`
    section:

    ```
    /* Memory Spaces Definitions, 448K flash, 64K ram */
    /* Bootloader is at address 0x00000000 */
    MEMORY
    {
      rom (rx)  : ORIGIN = 0x00010000, LENGTH = 0x00020000
      prog (rx) : ORIGIN = 0x00030000, LENGTH = 0x00040000
      ram (rwx) : ORIGIN = 0x20000000, LENGTH = 0x00020000
    }

    MPU_MIN_ALIGN = 8K;
    ```

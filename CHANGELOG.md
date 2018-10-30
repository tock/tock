Since 1.2
=========

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
        UartMux<'static>,
        UartMux::new(
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

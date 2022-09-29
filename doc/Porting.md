Porting Tock
============

This guide covers how to port Tock to a new platform.

_It is a work in progress. Comments and pull requests are appreciated!_

<!-- npm i -g markdown-toc; markdown-toc -i Porting.md -->

<!-- toc -->

- [Overview](#overview)
- [Is Tock a Good Fit for my Hardware?](#is-tock-a-good-fit-for-my-hardware)
- [Crate Details](#crate-details)
  * [`arch` Crate](#arch-crate)
  * [`chip` Crate](#chip-crate)
    + [Tips and Tools](#tips-and-tools)
  * [`board` Crate](#board-crate)
    + [Component Creation](#component-creation)
    + [Board Support](#board-support)
      - [`panic!`s (aka `io.rs`)](#panics-aka-iors)
      - [Board Cargo.toml, build.rs](#board-cargotoml-buildrs)
      - [Board Makefile](#board-makefile)
        * [Getting the built kernel onto a board](#getting-the-built-kernel-onto-a-board)
      - [Board README](#board-readme)
    + [Loading Apps](#loading-apps)
    + [Common Pitfalls](#common-pitfalls)
- [Adding a Platform to Tock Repository](#adding-a-platform-to-tock-repository)

<!-- tocstop -->

Overview
--------

At a high level, to port Tock to a new platform you will need to create a new
"board" as a crate, as well as potentially add additional "chip" and "arch"
crates. The board crate specifies the exact resources available on a hardware
platform by stitching capsules together with the chip crates (e.g. assigning
pins, setting baud rates, allocating hardware peripherals etc.). The chip crate
implements the peripheral drivers (e.g. UART, GPIO, alarms, etc.) for a specific
microcontroller by implementing the traits found in `kernel/src/hil`. If your
platform uses a microcontroller already supported by Tock then you can use the
existing chip crate. The arch crate implements the low-level code for a specific
hardware architecture (e.g. what happens when the chip first boots and how
system calls are implemented).


Is Tock a Good Fit for my Hardware?
-----------------------------------

Before porting Tock to a new platform or microcontroller, you should determine
if Tock is a good fit. While we do not have an exact rubric, there are some
requirements that we generally look for:

- Must have requirements:

  - Memory protection support. This is generally the MPU on Cortex-M platforms
    or the PMP on RISC-V platforms.
  - At least 32-bit support. Tock is not designed for 16-bit platforms.
  - Enough RAM and flash to support userspace applications. "Enough" is
    underspecified, but generally boards should have at least 64 kB of RAM and
    128 kB of flash.

- Generally expected requirements:

  - The platform should be 32-bit. Tock may support 64-bit in the future.
  - The platform should be single core. A multicore CPU is OK, but the
    expectation is that only one core will be used with Tock.


Crate Details
-------------

This section includes more details on what is required to implement each type of
crate for a new hardware platform.

### `arch` Crate

Tock currently supports the ARM Cortex-M0, Cortex-M3, and Cortex M4, and the
riscv32imac architectures. There is not much architecture-specific code in Tock,
the list is pretty much:

 - Syscall entry/exit
 - Interrupt configuration
 - Top-half interrupt handlers
 - MPU configuration (if appropriate)
 - Power management configuration (if appropriate)

It would likely be fairly easy to port Tock to another ARM Cortex M
(specifically the M0+, M23, M4F, or M7) or another riscv32 variant. It will
probably be more work to port Tock to other architectures. While we aim to be
architecture agnostic, this has only been tested on a small number of
architectures.

If you are interested in porting Tock to a new architecture, it's likely best
to reach out to us via email or Slack before digging in too deep.


### `chip` Crate

The `chip` crate is specific to a particular microcontroller, but should attempt
to be general towards a family of microcontrollers. For example, support for the
`nRF58240` and `nRF58230` microcontrollers is shared in the `chips/nrf52` and
`chips/nrf5x` crates. This helps reduce duplicated code and simplifies adding
new specific microcontrollers.

The `chip` crate contains microcontroller-specific implementations of the
interfaces defined in `kernel/src/hil`.

Chips have a lot of features and Tock supports a large number of interfaces to
express them. Build up the implementation of a new chip incrementally. Get reset
and initialization code working. Set it up to run on the chip's default clock
and add a GPIO interface. That's a good point to put together a minimal board
that uses the chip and validate with an end-to-end userland application that
uses GPIOs.

Once you have something small like GPIOs working, it's a great time to open a
pull request to Tock. This lets others know about your efforts with this chip
and can hopefully attract additional support. It also is a chance to get some
feedback from the Tock core team before you have written too much code.

Moving forward, chips tend to break down into reasonable units of work.
Implement something like `kernel::hil::UART` for your chip, then submit a pull
request. Pick a new peripheral and repeat!

Historically, Tock chips defined peripherals as `static mut` global variables,
which made them easy to access but encouraged use of unsafe code and prevented
boards from instantiating only the set of peripherals they needed. Now,
peripherals are instantiated at runtime in `main.rs`, which resolves these
issues. To prevent each board from having to instantiate peripherals
individually, chips should provide a `ChipNameDefaultPeripherals` struct that
defines and creates all peripherals available for the chip in Tock. This will be
used by upstream boards using the chip, without forcing the overhead and code
size of all peripherals on more minimal out-of-tree boards.

#### Tips and Tools

- Using System View Description (SVD) files for specific microcontrollers can
  help with setting up the register mappings for individual peripherals. See the
  `tools/svd2regs.py` tool (`./svd2regs.py -h`) for help with automatically
  generating the register mappings.

### `board` Crate

The `board` crate, in `boards/src`, is specific to a physical hardware platform.
The board file essentially configures the kernel to support the specific
hardware setup. This includes instantiating drivers for sensors, mapping
communication buses to those sensors, configuring GPIO pins, etc.

Tock is leveraging "components" for setting up board crates. Components are
contained structs that include all of the setup code for a particular driver,
and only require boards to pass in the specific options that are unique to the
particular platform. For example:

```rust
let ambient_light = AmbientLightComponent::new(board_kernel, mux_i2c, mux_alarm)
    .finalize(components::isl29035_component_helper!(sam4l::ast::Ast));
```

instantiates the component for an ambient light sensor. Board initiation should
be largely done using components, but not all components have been created yet,
so board files are generally a mix of components and verbose driver
instantiation. The best bet is to start from an existing board's `main.rs` file
and adapt it. Initially, you will likely want to delete most of the capsules and
add them slowly as you get things working.

> Warning: Many components are singletons, that is they may not be instantiated
> multiple times. Components should only be instantiated in the reset handler to
> avoid any multiple instantiations.

#### Component Creation

Creating a component for a capsule has two main benefits: 1) all subtleties and
any complexities with setting up the capsule can be contained in the component,
reducing the chance for error when using the capsule, and 2) the details of
instantiating a capsule are abstracted from the high-level setup of a board.
Therefore, Tock encourages boards to use components for their main startup
process.

Basic components generally have a structure like the following simplified
example for a `Console` component:

```rust
use core::mem::MaybeUninit;
use kernel::static_buf;

/// Helper macro that calls `static_buf!()`. This helps allow components to be
/// instantiated multiple times.
#[macro_export]
macro_rules! console_component_static {
    () => {{
        let console = static_buf!(capsules::console::Console<'static>);
        console
    }};
}

/// Main struct that represents the component. This should contain all
/// configuration and resources needed to instantiate this capsule.
pub struct ConsoleComponent {
    uart: &'static capsules::virtual_uart::UartDevice<'static>,
}

impl ConsoleComponent {
    /// The constructor for the component where the resources and configuration
    /// are provided.
    pub fn new(
        uart: &'static capsules::virtual_uart::UartDevice,
    ) -> ConsoleComponent {
        ConsoleComponent {
            uart,
        }
    }
}

impl Component for ConsoleComponent {
    /// The statically defined (using `static_buf!()`) structures where the
    /// instantiated capsules will actually be stored.
    type StaticInput = &'static mut MaybeUninit<capsules::console::Console<'static>>;
    /// What will be returned to the user of the component.
    type Output = &'static capsules::console::Console<'static>;

    /// Initializes and configures the capsule.
    unsafe fn finalize(self, s: Self::StaticInput) -> Self::Output {
        /// Call `.write()` on the static buffer to set its contents with the
        /// constructor from the capsule.
        let console = s.write(console::Console::new(self.uart));

        /// Set any needed clients or other configuration steps.
        hil::uart::Transmit::set_transmit_client(self.uart, console);
        hil::uart::Receive::set_receive_client(self.uart, console);

        /// Return the static reference to the newly created capsule object.
        console
    }
}
```

Using a basic component like this console example looks like:

```rust
// in main.rs:

let console = ConsoleComponent::new(uart_device)
    .finalize(components::console_component_static!());
```

When creating components, keep the following steps in mind:

- All static buffers needed for the component **MUST** be created using
  `static_buf!()` inside of a macro, and nowhere else. This is necessary to help
  allow components to be used multiple times (for example if a board has two
  temperature sensors). Because the same `static_buf!()` call cannot be executed
  multiple times, `static_buf!()` cannot be placed in a function, and must be
  called directly from main.rs. To preserve the ergonomics of components, we
  wrap the call to `static_buf!()` in a macro, and call the macro from main.rs
  instead of `static_buf!()` directly.

  The naming convention of the macro that wraps `static_buf!()` should be
  `[capsule name]_component_static!()` to indicate this is where the static
  buffers are created. The macro should _only_ create static buffers.

- All configuration and resources not related to static buffers should be passed
  to the `new()` constructor of the component object.

Finally, some capsules and resources are templated over chip-specific resources.
This slightly complicates defining the static buffers for certain capsules. To
ensure that components can be re-used across different boards and
microcontrollers, components use the same macro strategy for other static
buffers.

```rust
use core::mem::MaybeUninit;
use kernel::static_buf;

#[macro_export]
macro_rules! alarm_mux_component_static {
    ($A: ty) => {{
        let alarm = static_buf!(capsules::virtual_alarm::MuxAlarm<'static, $A>);
        alarm
    }};
}

pub struct AlarmMuxComponent<A: 'static + time::Alarm<'static>> {
    alarm: &'static A,
}

impl<A: 'static + time::Alarm<'static>> AlarmMuxComponent<A> {
    pub fn new(alarm: &'static A) -> AlarmMuxComponent<A> {
        AlarmMuxComponent { alarm }
    }
}

impl<A: 'static + time::Alarm<'static>> Component for AlarmMuxComponent<A> {
    type StaticInput = &'static mut MaybeUninit<capsules::virtual_alarm::MuxAlarm<'static, A>>;
    type Output = &'static MuxAlarm<'static, A>;

    unsafe fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let mux_alarm = s.write(MuxAlarm::new(self.alarm));
        self.alarm.set_alarm_client(mux_alarm);
        mux_alarm
    }
}
```

Here, the `alarm_mux_component_static!()` macro needs the type of the underlying
alarm hardware. The usage looks like:

```rust
let mux_alarm = components::alarm::AlarmMuxComponent::new(&peripherals.ast)
    .finalize(components::alarm_mux_component_static!(sam4l::ast::Ast));
```


#### Board Support

In addition to kernel code, boards also require some support files. These
specify metadata such as the board name, how to load code onto the board, and
anything special that userland applications may need for this board.

##### `panic!`s (aka `io.rs`)

Each board must author a custom routine to handle `panic!`s. Most `panic!`
machinery is handled by the Tock kernel, but the board author must provide some
minimalist access to hardware interfaces, specifically LEDs and/or UART.

As a first step, it is simplest to just get LED-based `panic!` working. Have
your `panic!` handler set up a prominent LED and then call
[kernel::debug::panic_blink_forever](https://docs.tockos.org/kernel/debug/fn.panic_blink_forever.html).

If UART is available, the kernel is capable of printing a lot of very helpful
additional debugging information. However, as we are in a `panic!` situation,
it's important to strip this down to a minimalist implementation. In particular,
the supplied UART must be synchronous (note that this in contrast to the rest of
the kernel UART interfaces, which are all asynchronous). Usually implementing a
very simple `Writer` that simply writes one byte at a time directly to the UART
is easiest/best. It is not important that `panic!` UART writer be efficient. You
can then replace the call to
[kernel::debug::panic_blink_forever](https://docs.tockos.org/kernel/debug/fn.panic_blink_forever.html)
with a call to
[kernel::debug::panic](https://docs.tockos.org/kernel/debug/fn.panic.html).

For largely historical reasons, panic implementations for all boards live in a
file named `io.rs` adjacent to the board's `main.rs` file.

##### Board Cargo.toml, build.rs

Every board crate must author a top-level manifest, `Cargo.toml`. In general,
you can probably simply copy this from another board, modifying the board name
and author(s) as appropriate. Note that Tock also includes a build script,
`build.rs`, that you should also copy. The build script simply adds a dependency
on the kernel layout.

##### Board Makefile

There is a Makefile in the root of every board crate, at a minimum, the board
Makefile must include:

```make
# Makefile for building the tock kernel for the Hail platform

TARGET=thumbv7em-none-eabi      # Target triple
PLATFORM=hail                   # Board name here

include ../Makefile.common      # ../ assumes board lives in $(TOCK)/boards/<board>
```

Tock provides `boards/Makefile.common` that drives most of the build system. In
general, you should not need to dig into this Makefile -- if something doesn't
seem to be working, hop on slack and ask.

###### Getting the built kernel onto a board

In addition to building the kernel, the board Makefile should include rules for
getting code onto the board. This will naturally be fairly board-specific, but
Tock does have two targets normally supplied:

  - _program_: For "plug-'n-plug" loading. Usually these are boards with a
    bootloader or some other support IC. The expectation is that during normal
    operation, a user could simply plug in a board and type `make program` to
    load code.
  - _flash_: For "more direct" loading. Usually this means that a JTAG or some
    equivalent interface is being used. Often it implies that external hardware
    is required, though some of the development kit boards have an integrated
    JTAG on-board, so external hardware is not a hard and fast rule.
  - _install_: This should be an alias to either `program` or `flash`, whichever
    is the preferred approach for this board.

If you don't support _program_ or _flash_, you should define an empty rule
that explains how to program the board:

```make
.PHONY: program
        echo "To program, run SPEICAL_COMMAND"
        exit 1
```

##### Board README

Every board must have a `README.md` file included in the top level of the crate.
This file must:

- Provide links to information about the platform and how to purchase/acquire
  the platform. If there are different versions of the platform the version used
  in testing should be clearly specified.
- Include an overview on how to program the hardware, including any additional
  dependencies that are required.

#### Loading Apps

Ideally, [Tockloader](https://github.com/tock/tockloader) will support loading
apps on to your board (perhaps with some flags set to specific values). If that
is not the case, please create an issue on the Tockloader repo so we can update
the tool to support loading code onto your board.

#### Common Pitfalls

- Make sure you are careful when setting up the board `main.rs` file. In
  particular, it is important to ensure that all of the required `set_client`
  functions for capsules are called so that callbacks are not lost. Forgetting
  these often results in the platform looking like it doesn't do anything.


Adding a Platform to Tock Repository
------------------------------------

After creating a new platform, we would be thrilled to have it included in
mainline Tock. However, Tock has a few guidelines for the minimum requirements
of a board that is merged into the main Tock repository:

1. The hardware must be widely available. Generally that means the hardware
   platform can be purchased online.
2. The port of Tock to the platform must include at least:
    - `Console` support so that `debug!()` and `printf()` work.
    - Timer support.
    - GPIO support with interrupt functionality.
3. The contributor must be willing to maintain the platform, at least initially,
   and help test the platform for future releases.

With these requirements met we should be able to merge the platform into Tock
relatively quickly. In the pull request to add the platform, you should add this
checklist:

```md
### New Platform Checklist

- [ ] Hardware is widely available.
- [ ] I can support the platform, which includes release testing for the platform, at least initially.
- Basic features are implemented:
  - [ ] `Console`, including `debug!()` and userspace `printf()`.
  - [ ] Timers.
  - [ ] GPIO with interrupts.
```

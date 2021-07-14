//! Provides userspace applications with a driver interface to asynchronous GPIO
//! pins.
//!
//! Async GPIO pins are pins that exist on something like a GPIO extender or a
//! radio that has controllable GPIOs.
//!
//! Usage
//! -----
//!
//! ```rust
//! # use kernel::static_init;
//!
//! // Generate a list of ports to group into one userspace driver.
//! let async_gpio_ports = static_init!(
//!     [&'static capsules::mcp230xx::MCP230xx; 1],
//!     [mcp23008]);
//!
//! let gpio_async = static_init!(
//!     capsules::gpio_async::GPIOAsync<'static, capsules::mcp230xx::MCP230xx<'static>>,
//!     capsules::gpio_async::GPIOAsync::new(async_gpio_ports));
//!
//! // Setup the clients correctly.
//! for port in async_gpio_ports.iter() {
//!     port.set_client(gpio_async);
//! }
//! ```

use kernel::grant::Grant;
use kernel::hil;
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::utilities::cells::OptionalCell;
use kernel::{ErrorCode, ProcessId};

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::GpioAsync as usize;

pub struct GPIOAsync<'a, Port: hil::gpio_async::Port> {
    ports: &'a [&'a Port],
    grants: Grant<App, 2>,
    /// **Transient** ownership of the partially virtualized peripheral.
    ///
    /// Current GPIO HIL semantics notify *all* processes of interrupts
    /// to any pin with interrupts configured (and it's left to higher
    /// layers to filter activity based on which pin generated their
    /// interrupt). For Async GPIO, this is awkward to virtualize, as
    /// there is no owning process of a pin for activity interrupts, but
    /// there is for configuration result interrupts. Also, the underlying
    /// hardware likely can't handle multiple concurrent configurations
    /// from multiple apps. Hence, this variable, which tracks a configuration
    /// while it is in flight and notifies the correct process that their
    /// configuration has succeeded. In the rare case where two apps attempt
    /// concurrent configuration requests, the later app will receive `EBUSY`.
    /// A retry loop should be sufficient for most apps to handle this rare
    /// case.
    configuring_process: OptionalCell<ProcessId>,
}

#[derive(Default)]
pub struct App {}

impl<'a, Port: hil::gpio_async::Port> GPIOAsync<'a, Port> {
    pub fn new(ports: &'a [&'a Port], grants: Grant<App, 2>) -> GPIOAsync<'a, Port> {
        GPIOAsync {
            ports,
            grants,
            configuring_process: OptionalCell::empty(),
        }
    }

    fn configure_input_pin(&self, port: usize, pin: usize, config: usize) -> Result<(), ErrorCode> {
        let ports = self.ports.as_ref();
        let mode = match config {
            0 => hil::gpio::FloatingState::PullNone,
            1 => hil::gpio::FloatingState::PullUp,
            2 => hil::gpio::FloatingState::PullDown,
            _ => return Err(ErrorCode::INVAL),
        };
        ports[port].make_input(pin, mode)
    }

    fn configure_interrupt(&self, port: usize, pin: usize, config: usize) -> Result<(), ErrorCode> {
        let ports = self.ports.as_ref();
        let mode = match config {
            0 => hil::gpio::InterruptEdge::EitherEdge,
            1 => hil::gpio::InterruptEdge::RisingEdge,
            2 => hil::gpio::InterruptEdge::FallingEdge,
            _ => return Err(ErrorCode::INVAL),
        };
        ports[port].enable_interrupt(pin, mode)
    }
}

impl<Port: hil::gpio_async::Port> hil::gpio_async::Client for GPIOAsync<'_, Port> {
    fn fired(&self, pin: usize, identifier: usize) {
        // schedule callback with the pin number and value for all apps
        self.grants.each(|_, _app, upcalls| {
            upcalls.schedule_upcall(1, identifier, pin, 0).ok();
        });
    }

    fn done(&self, value: usize) {
        // alert currently configuring app
        self.configuring_process.map(|pid| {
            let _ = self.grants.enter(*pid, |_app, upcalls| {
                upcalls.schedule_upcall(0, 0, value, 0).ok();
            });
        });
        // then clear currently configuring app
        self.configuring_process.clear();
    }
}

impl<Port: hil::gpio_async::Port> SyscallDriver for GPIOAsync<'_, Port> {
    // Setup callbacks for gpio_async events.
    //
    // ### `subscribe_num`
    //
    // - `0`: Setup a callback for when **split-phase operations complete**.
    //   This callback gets called from the gpio_async `done()` event and
    //   signals the end of operations like asserting a GPIO pin or configuring
    //   an interrupt pin. The callback will be called with two valid
    //   arguments. The first is the callback type, which is currently 0 for
    //   all `done()` events. The second is a value, which is only useful for
    //   operations which should return something, like a GPIO read.
    // - `1`: Setup a callback for when a **GPIO interrupt** occurs. This
    //   callback will be called with two arguments, the first being the port
    //   number of the interrupting pin, and the second being the pin number.

    /// Configure and read GPIO pins.
    ///
    /// `pin` is the index of the pin.
    ///
    /// `data` is a 32 bit value packed with the lowest 16 bits as the port
    /// number, and the remaining upper bits as a command-specific value.
    ///
    /// ### `command_num`
    ///
    /// - `0`: Driver check and get number of GPIO ports supported.
    /// - `1`: Set a pin as an output.
    /// - `2`: Set a pin high by setting it to 1.
    /// - `3`: Clear a pin by setting it to 0.
    /// - `4`: Toggle a pin.
    /// - `5`: Set a pin as an input and configure its pull-up or pull-down
    ///   state. The command-specific field should be set to 0 for a pull-up, 1
    ///   for a pull-down, or 2 for neither.
    /// - `6`: Read a GPIO pin state, and have its value returned in the done()
    ///   callback.
    /// - `7`: Enable an interrupt on a GPIO pin. The command-specific data
    ///   should be 0 for an either-edge interrupt, 1 for a rising edge
    ///   interrupt, and 2 for a falling edge interrupt.
    /// - `8`: Disable an interrupt on a pin.
    /// - `9`: Disable a GPIO pin.
    fn command(
        &self,
        command_number: usize,
        pin: usize,
        data: usize,
        process_id: ProcessId,
    ) -> CommandReturn {
        let port = data & 0xFFFF;
        let other = (data >> 16) & 0xFFFF;
        let ports = self.ports.as_ref();

        // Special case command 0; everything else results in a process-owned,
        // split-phase call.
        if command_number == 0 {
            // How many ports
            return CommandReturn::success_u32(ports.len() as u32);
        }

        // On any command other than 0, we check for ports length.
        if port >= ports.len() {
            return CommandReturn::failure(ErrorCode::INVAL);
        }

        // On any command other than 0, we check if another command is in flight
        if self.configuring_process.is_some() {
            return CommandReturn::failure(ErrorCode::BUSY);
        };

        let res = match command_number {
            // enable output
            1 => ports[port].make_output(pin),

            // set pin
            2 => ports[port].set(pin),

            // clear pin
            3 => ports[port].clear(pin),

            // toggle pin
            4 => ports[port].toggle(pin),

            // enable and configure input
            5 => self.configure_input_pin(port, pin, other & 0xFF),

            // read input
            6 => ports[port].read(pin),

            // enable interrupt on pin
            7 => self.configure_interrupt(port, pin, other & 0xFF),

            // disable interrupt on pin
            8 => ports[port].disable_interrupt(pin),

            // disable pin
            9 => ports[port].disable(pin),

            // default
            _ => return CommandReturn::failure(ErrorCode::NOSUPPORT),
        };

        // If any async command kicked off, note that the peripheral is busy
        // and which process to return the command result to
        if res.is_ok() {
            self.configuring_process.set(process_id);
        }

        res.into()
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::process::Error> {
        self.grants.enter(processid, |_, _| {})
    }
}

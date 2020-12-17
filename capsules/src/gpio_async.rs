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

use core::cell::RefCell;
use kernel::hil;
use kernel::{AppId, Callback, ErrorCode};
use kernel::{CommandReturn, Driver, ReturnCode};

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::GpioAsync as usize;

pub struct GPIOAsync<'a, Port: hil::gpio_async::Port> {
    ports: &'a [&'a Port],
    callback: RefCell<Callback>,
    interrupt_callback: RefCell<Callback>,
}

impl<'a, Port: hil::gpio_async::Port> GPIOAsync<'a, Port> {
    pub fn new(ports: &'a [&'a Port]) -> GPIOAsync<'a, Port> {
        GPIOAsync {
            ports,
            callback: RefCell::new(Callback::default()),
            interrupt_callback: RefCell::new(Callback::default()),
        }
    }

    fn configure_input_pin(&self, port: usize, pin: usize, config: usize) -> ReturnCode {
        let ports = self.ports.as_ref();
        let mode = match config {
            0 => hil::gpio::FloatingState::PullNone,
            1 => hil::gpio::FloatingState::PullUp,
            2 => hil::gpio::FloatingState::PullDown,
            _ => return ReturnCode::EINVAL,
        };
        ports[port].make_input(pin, mode)
    }

    fn configure_interrupt(&self, port: usize, pin: usize, config: usize) -> ReturnCode {
        let ports = self.ports.as_ref();
        let mode = match config {
            0 => hil::gpio::InterruptEdge::EitherEdge,
            1 => hil::gpio::InterruptEdge::RisingEdge,
            2 => hil::gpio::InterruptEdge::FallingEdge,
            _ => return ReturnCode::EINVAL,
        };
        ports[port].enable_interrupt(pin, mode)
    }
}

impl<Port: hil::gpio_async::Port> hil::gpio_async::Client for GPIOAsync<'_, Port> {
    fn fired(&self, pin: usize, identifier: usize) {
        self.interrupt_callback
            .borrow_mut()
            .schedule(identifier, pin, 0);
    }

    fn done(&self, value: usize) {
        self.callback.borrow_mut().schedule(0, value, 0);
    }
}

impl<Port: hil::gpio_async::Port> Driver for GPIOAsync<'_, Port> {
    /// Setup callbacks for gpio_async events.
    ///
    /// ### `subscribe_num`
    ///
    /// - `0`: Setup a callback for when **split-phase operations complete**.
    ///   This callback gets called from the gpio_async `done()` event and
    ///   signals the end of operations like asserting a GPIO pin or configuring
    ///   an interrupt pin. The callback will be called with two valid
    ///   arguments. The first is the callback type, which is currently 0 for
    ///   all `done()` events. The second is a value, which is only useful for
    ///   operations which should return something, like a GPIO read.
    /// - `1`: Setup a callback for when a **GPIO interrupt** occurs. This
    ///   callback will be called with two arguments, the first being the port
    ///   number of the interrupting pin, and the second being the pin number.
    fn subscribe(
        &self,
        subscribe_num: usize,
        callback: Callback,
        _app_id: AppId,
    ) -> Result<Callback, (Callback, ErrorCode)> {
        match subscribe_num {
            // Set callback for `done()` events
            0 => Ok(self.callback.replace(callback)),

            // Set callback for pin interrupts
            1 => Ok(self.interrupt_callback.replace(callback)),

            // default
            _ => Err((callback, ErrorCode::NOSUPPORT)),
        }
    }

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
        _appid: AppId,
    ) -> CommandReturn {
        let port = data & 0xFFFF;
        let other = (data >> 16) & 0xFFFF;
        let ports = self.ports.as_ref();

        // On any command other than 0, we check for ports length.
        if command_number != 0 && port >= ports.len() {
            return CommandReturn::failure(ErrorCode::INVAL);
        }

        match command_number {
            // How many ports
            0 => CommandReturn::success_u32(ports.len() as u32),

            // enable output
            1 => ports[port].make_output(pin).into(),

            // set pin
            2 => ports[port].set(pin).into(),

            // clear pin
            3 => ports[port].clear(pin).into(),

            // toggle pin
            4 => ports[port].toggle(pin).into(),

            // enable and configure input
            5 => self.configure_input_pin(port, pin, other & 0xFF).into(),

            // read input
            6 => ports[port].read(pin).into(),

            // enable interrupt on pin
            7 => self.configure_interrupt(port, pin, other & 0xFF).into(),

            // disable interrupt on pin
            8 => ports[port].disable_interrupt(pin).into(),

            // disable pin
            9 => ports[port].disable(pin).into(),

            // default
            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }
}

//! Provides userspace applications with access to GPIO pins.
//!
//! GPIOs are presented through a driver interface with synchronous commands
//! and a callback for interrupts.
//!
//! This capsule takes an array of pins to expose as generic GPIOs.
//! Note that this capsule is used for general purpose GPIOs. Pins that are
//! attached to LEDs or buttons are generally wired directly to those capsules,
//! not through this capsule as an intermediary.
//!
//! Usage
//! -----
//!
//! ```rust
//! let gpio_pins = static_init!(
//!     [&'static sam4l::gpio::GPIOPin; 4],
//!     [&sam4l::gpio::PB[14],
//!      &sam4l::gpio::PB[15],
//!      &sam4l::gpio::PB[11],
//!      &sam4l::gpio::PB[12]]);
//! let gpio = static_init!(
//!     capsules::gpio::GPIO<'static, sam4l::gpio::GPIOPin>,
//!     capsules::gpio::GPIO::new(gpio_pins));
//! for pin in gpio_pins.iter() {
//!     pin.set_client(gpio);
//! }
//! ```
//!
//! Syscall Interface
//! -----------------
//!
//! - Stability: 2 - Stable
//!
//! ### Commands
//!
//! All GPIO operations are synchronous.
//!
//! Commands control and query GPIO information, namely how many GPIOs are
//! present, the GPIO direction and state, and whether they should interrupt.
//!
//! ### Subscribes
//!
//! The GPIO interface provides only one callback, which is used for pins that
//! have had interrupts enabled.

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::GPIO as usize;

use kernel::hil::gpio::{Client, InputMode, InterruptMode, Pin, PinCtl};
use kernel::{AppId, Callback, Driver, Grant, ReturnCode};

pub struct GPIO<'a, G: Pin> {
    pins: &'a [&'a G],
    apps: Grant<Option<Callback>>,
}

impl<G: Pin + PinCtl> GPIO<'a, G> {
    pub fn new(pins: &'a [&'a G], grant: Grant<Option<Callback>>) -> GPIO<'a, G> {
        GPIO {
            pins: pins,
            apps: grant,
        }
    }

    fn configure_input_pin(&self, pin_num: usize, config: usize) -> ReturnCode {
        let pin = self.pins[pin_num];
        pin.make_input();
        match config {
            0 => {
                pin.set_input_mode(InputMode::PullNone);
                ReturnCode::SUCCESS
            }
            1 => {
                pin.set_input_mode(InputMode::PullUp);
                ReturnCode::SUCCESS
            }
            2 => {
                pin.set_input_mode(InputMode::PullDown);
                ReturnCode::SUCCESS
            }
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    fn configure_interrupt(&self, pin_num: usize, config: usize) -> ReturnCode {
        let pins = self.pins.as_ref();
        match config {
            0 => {
                pins[pin_num].enable_interrupt(pin_num, InterruptMode::EitherEdge);
                ReturnCode::SUCCESS
            }

            1 => {
                pins[pin_num].enable_interrupt(pin_num, InterruptMode::RisingEdge);
                ReturnCode::SUCCESS
            }

            2 => {
                pins[pin_num].enable_interrupt(pin_num, InterruptMode::FallingEdge);
                ReturnCode::SUCCESS
            }

            _ => ReturnCode::ENOSUPPORT,
        }
    }
}

impl<G: Pin> Client for GPIO<'a, G> {
    fn fired(&self, pin_num: usize) {
        // read the value of the pin
        let pins = self.pins.as_ref();
        let pin_state = pins[pin_num].read();

        // schedule callback with the pin number and value
        self.apps.each(|callback| {
            callback.map(|mut cb| cb.schedule(pin_num, pin_state as usize, 0));
        });
    }
}

impl<G: Pin + PinCtl> Driver for GPIO<'a, G> {
    /// Subscribe to GPIO pin events.
    ///
    /// ### `subscribe_num`
    ///
    /// - `0`: Subscribe to interrupts from all pins with interrupts enabled.
    ///        The callback signature is `fn(pin_num: usize, pin_state: bool)`
    fn subscribe(
        &self,
        subscribe_num: usize,
        callback: Option<Callback>,
        app_id: AppId,
    ) -> ReturnCode {
        match subscribe_num {
            // subscribe to all pin interrupts (no affect or reliance on
            // individual pins being configured as interrupts)
            0 => self
                .apps
                .enter(app_id, |app, _| {
                    **app = callback;
                    ReturnCode::SUCCESS
                })
                .unwrap_or_else(|err| err.into()),

            // default
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    /// Query and control pin values and states.
    ///
    /// Each byte of the `data` argument is treated as its own field.
    /// For all commands, the lowest order halfword is the pin number (`pin`).
    /// A few commands use higher order bytes for purposes documented below.
    /// If the higher order bytes are not used, they must be set to `0`.
    ///
    /// Other data bytes:
    ///
    ///   - `pin_config`: An internal resistor setting.
    ///                   Set to `0` for a pull-up resistor.
    ///                   Set to `1` for a pull-down resistor.
    ///                   Set to `2` for none.
    ///   - `irq_config`: Interrupt configuration setting.
    ///                   Set to `0` to interrupt on either edge.
    ///                   Set to `1` for rising edge.
    ///                   Set to `2` for falling edge.
    ///
    /// ### `command_num`
    ///
    /// - `0`: Number of pins.
    /// - `1`: Enable output on `pin`.
    /// - `2`: Set `pin`.
    /// - `3`: Clear `pin`.
    /// - `4`: Toggle `pin`.
    /// - `5`: Enable input on `pin` with `pin_config` in 0x00XX00000
    /// - `6`: Read `pin` value.
    /// - `7`: Configure interrupt on `pin` with `irq_config` in 0x00XX00000
    /// - `8`: Disable interrupt on `pin`.
    /// - `9`: Disable `pin`.
    fn command(&self, command_num: usize, data1: usize, data2: usize, _: AppId) -> ReturnCode {
        let pins = self.pins.as_ref();
        let pin = data1;
        match command_num {
            // number of pins
            0 => ReturnCode::SuccessWithValue {
                value: pins.len() as usize,
            },

            // enable output
            1 => {
                if pin >= pins.len() {
                    ReturnCode::EINVAL /* impossible pin */
                } else {
                    pins[pin].make_output();
                    ReturnCode::SUCCESS
                }
            }

            // set pin
            2 => {
                if pin >= pins.len() {
                    ReturnCode::EINVAL /* impossible pin */
                } else {
                    pins[pin].set();
                    ReturnCode::SUCCESS
                }
            }

            // clear pin
            3 => {
                if pin >= pins.len() {
                    ReturnCode::EINVAL /* impossible pin */
                } else {
                    pins[pin].clear();
                    ReturnCode::SUCCESS
                }
            }

            // toggle pin
            4 => {
                if pin >= pins.len() {
                    ReturnCode::EINVAL /* impossible pin */
                } else {
                    pins[pin].toggle();
                    ReturnCode::SUCCESS
                }
            }

            // enable and configure input
            5 => {
                let pin_config = data2;
                if pin >= pins.len() {
                    ReturnCode::EINVAL /* impossible pin */
                } else {
                    self.configure_input_pin(pin, pin_config)
                }
            }

            // read input
            6 => {
                if pin >= pins.len() {
                    ReturnCode::EINVAL /* impossible pin */
                } else {
                    let pin_state = pins[pin].read();
                    ReturnCode::SuccessWithValue {
                        value: pin_state as usize,
                    }
                }
            }

            // configure interrupts on pin
            // (no affect or reliance on registered callback)
            7 => {
                let irq_config = data2;
                if pin >= pins.len() {
                    ReturnCode::EINVAL /* impossible pin */
                } else {
                    self.configure_interrupt(pin, irq_config)
                }
            }

            // disable interrupts on pin, also disables pin
            // (no affect or reliance on registered callback)
            8 => {
                if pin >= pins.len() {
                    ReturnCode::EINVAL /* impossible pin */
                } else {
                    pins[pin].disable_interrupt();
                    pins[pin].disable();
                    ReturnCode::SUCCESS
                }
            }

            // disable pin
            9 => {
                if pin >= pins.len() {
                    ReturnCode::EINVAL /* impossible pin */
                } else {
                    pins[pin].disable();
                    ReturnCode::SUCCESS
                }
            }

            // default
            _ => ReturnCode::ENOSUPPORT,
        }
    }
}

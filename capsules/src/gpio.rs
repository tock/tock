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
//! # use kernel::static_init;
//!
//! let gpio_pins = static_init!(
//!     [Option<&'static sam4l::gpio::GPIOPin>; 4],
//!     [Option<&sam4l::gpio::PB[14]>,
//!      Option<&sam4l::gpio::PB[15]>,
//!      Option<&sam4l::gpio::PB[11]>,
//!      Option<&sam4l::gpio::PB[12]>]);
//! let gpio = static_init!(
//!     capsules::gpio::GPIO<'static, sam4l::gpio::GPIOPin>,
//!     capsules::gpio::GPIO::new(gpio_pins));
//! for maybe_pin in gpio_pins.iter() {
//!     if let Some(pin) = maybe_pin {
//!         pin.set_client(gpio);
//!     }
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
pub const DRIVER_NUM: usize = driver::NUM::Gpio as usize;

use kernel::hil::gpio;
use kernel::hil::gpio::{Configure, Input, InterruptWithValue, Output};
use kernel::{AppId, Callback, Grant, LegacyDriver, ReturnCode};

pub struct GPIO<'a, IP: gpio::InterruptPin<'a>> {
    pins: &'a [Option<&'a gpio::InterruptValueWrapper<'a, IP>>],
    apps: Grant<Option<Callback>>,
}

impl<'a, IP: gpio::InterruptPin<'a>> GPIO<'a, IP> {
    pub fn new(
        pins: &'a [Option<&'a gpio::InterruptValueWrapper<'a, IP>>],
        grant: Grant<Option<Callback>>,
    ) -> Self {
        for (i, maybe_pin) in pins.iter().enumerate() {
            if let Some(pin) = maybe_pin {
                pin.set_value(i as u32);
            }
        }
        Self {
            pins: pins,
            apps: grant,
        }
    }

    fn configure_input_pin(&self, pin_num: u32, config: usize) -> ReturnCode {
        let maybe_pin = self.pins[pin_num as usize];
        if let Some(pin) = maybe_pin {
            pin.make_input();
            match config {
                0 => {
                    pin.set_floating_state(gpio::FloatingState::PullNone);
                    ReturnCode::SUCCESS
                }
                1 => {
                    pin.set_floating_state(gpio::FloatingState::PullUp);
                    ReturnCode::SUCCESS
                }
                2 => {
                    pin.set_floating_state(gpio::FloatingState::PullDown);
                    ReturnCode::SUCCESS
                }
                _ => ReturnCode::ENOSUPPORT,
            }
        } else {
            ReturnCode::ENODEVICE
        }
    }

    fn configure_interrupt(&self, pin_num: u32, config: usize) -> ReturnCode {
        let pins = self.pins.as_ref();
        let index = pin_num as usize;
        if let Some(pin) = pins[index] {
            match config {
                0 => {
                    pin.enable_interrupts(gpio::InterruptEdge::EitherEdge);
                    ReturnCode::SUCCESS
                }

                1 => {
                    pin.enable_interrupts(gpio::InterruptEdge::RisingEdge);
                    ReturnCode::SUCCESS
                }

                2 => {
                    pin.enable_interrupts(gpio::InterruptEdge::FallingEdge);
                    ReturnCode::SUCCESS
                }

                _ => ReturnCode::ENOSUPPORT,
            }
        } else {
            ReturnCode::ENODEVICE
        }
    }
}

impl<'a, IP: gpio::InterruptPin<'a>> gpio::ClientWithValue for GPIO<'a, IP> {
    fn fired(&self, pin_num: u32) {
        // read the value of the pin
        let pins = self.pins.as_ref();
        if let Some(pin) = pins[pin_num as usize] {
            let pin_state = pin.read();

            // schedule callback with the pin number and value
            self.apps.each(|callback| {
                callback.map(|mut cb| cb.schedule(pin_num as usize, pin_state as usize, 0));
            });
        }
    }
}

impl<'a, IP: gpio::InterruptPin<'a>> LegacyDriver for GPIO<'a, IP> {
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
        let pin_index = data1;
        match command_num {
            // number of pins
            0 => ReturnCode::SuccessWithValue {
                value: pins.len() as usize,
            },

            // enable output
            1 => {
                if pin_index >= pins.len() {
                    ReturnCode::EINVAL /* impossible pin */
                } else {
                    if let Some(pin) = pins[pin_index] {
                        pin.make_output();
                        ReturnCode::SUCCESS
                    } else {
                        ReturnCode::ENODEVICE
                    }
                }
            }

            // set pin
            2 => {
                if pin_index >= pins.len() {
                    ReturnCode::EINVAL /* impossible pin */
                } else {
                    if let Some(pin) = pins[pin_index] {
                        pin.set();
                        ReturnCode::SUCCESS
                    } else {
                        ReturnCode::ENODEVICE
                    }
                }
            }

            // clear pin
            3 => {
                if pin_index >= pins.len() {
                    ReturnCode::EINVAL /* impossible pin */
                } else {
                    if let Some(pin) = pins[pin_index] {
                        pin.clear();
                        ReturnCode::SUCCESS
                    } else {
                        ReturnCode::ENODEVICE
                    }
                }
            }

            // toggle pin
            4 => {
                if pin_index >= pins.len() {
                    ReturnCode::EINVAL /* impossible pin */
                } else {
                    if let Some(pin) = pins[pin_index] {
                        pin.toggle();
                        ReturnCode::SUCCESS
                    } else {
                        ReturnCode::ENODEVICE
                    }
                }
            }

            // enable and configure input
            5 => {
                let pin_config = data2;
                if pin_index >= pins.len() {
                    ReturnCode::EINVAL /* impossible pin */
                } else {
                    self.configure_input_pin(pin_index as u32, pin_config)
                }
            }

            // read input
            6 => {
                if pin_index >= pins.len() {
                    ReturnCode::EINVAL /* impossible pin */
                } else {
                    if let Some(pin) = pins[pin_index] {
                        let pin_state = pin.read();
                        ReturnCode::SuccessWithValue {
                            value: pin_state as usize,
                        }
                    } else {
                        ReturnCode::ENODEVICE
                    }
                }
            }

            // configure interrupts on pin
            // (no affect or reliance on registered callback)
            7 => {
                let irq_config = data2;
                if pin_index >= pins.len() {
                    ReturnCode::EINVAL /* impossible pin */
                } else {
                    self.configure_interrupt(pin_index as u32, irq_config)
                }
            }

            // disable interrupts on pin, also disables pin
            // (no affect or reliance on registered callback)
            8 => {
                if pin_index >= pins.len() {
                    ReturnCode::EINVAL /* impossible pin */
                } else {
                    if let Some(pin) = pins[pin_index] {
                        pin.disable_interrupts();
                        pin.deactivate_to_low_power();
                        ReturnCode::SUCCESS
                    } else {
                        ReturnCode::ENODEVICE
                    }
                }
            }

            // disable pin
            9 => {
                if pin_index >= pins.len() {
                    ReturnCode::EINVAL /* impossible pin */
                } else {
                    if let Some(pin) = pins[pin_index] {
                        pin.deactivate_to_low_power();
                        ReturnCode::SUCCESS
                    } else {
                        ReturnCode::ENODEVICE
                    }
                }
            }

            // default
            _ => ReturnCode::ENOSUPPORT,
        }
    }
}

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

use kernel::grant::Grant;
use kernel::hil::gpio;
use kernel::hil::gpio::{Configure, Input, InterruptWithValue, Output};
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::{ErrorCode, ProcessId};

/// ### `subscribe_num`
///
/// - `0`: Subscribe to interrupts from all pins with interrupts enabled.
///        The callback signature is `fn(pin_num: usize, pin_state: bool)`
const UPCALL_NUM: usize = 0;

pub struct GPIO<'a, IP: gpio::InterruptPin<'a>> {
    pins: &'a [Option<&'a gpio::InterruptValueWrapper<'a, IP>>],
    apps: Grant<(), 1>,
}

impl<'a, IP: gpio::InterruptPin<'a>> GPIO<'a, IP> {
    pub fn new(
        pins: &'a [Option<&'a gpio::InterruptValueWrapper<'a, IP>>],
        grant: Grant<(), 1>,
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

    fn configure_input_pin(&self, pin_num: u32, config: usize) -> CommandReturn {
        let maybe_pin = self.pins[pin_num as usize];
        if let Some(pin) = maybe_pin {
            pin.make_input();
            match config {
                0 => {
                    pin.set_floating_state(gpio::FloatingState::PullNone);
                    CommandReturn::success()
                }
                1 => {
                    pin.set_floating_state(gpio::FloatingState::PullUp);
                    CommandReturn::success()
                }
                2 => {
                    pin.set_floating_state(gpio::FloatingState::PullDown);
                    CommandReturn::success()
                }
                _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
            }
        } else {
            CommandReturn::failure(ErrorCode::NODEVICE)
        }
    }

    fn configure_interrupt(&self, pin_num: u32, config: usize) -> CommandReturn {
        let pins = self.pins.as_ref();
        let index = pin_num as usize;
        if let Some(pin) = pins[index] {
            match config {
                0 => {
                    let _ = pin.enable_interrupts(gpio::InterruptEdge::EitherEdge);
                    CommandReturn::success()
                }

                1 => {
                    let _ = pin.enable_interrupts(gpio::InterruptEdge::RisingEdge);
                    CommandReturn::success()
                }

                2 => {
                    let _ = pin.enable_interrupts(gpio::InterruptEdge::FallingEdge);
                    CommandReturn::success()
                }

                _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
            }
        } else {
            CommandReturn::failure(ErrorCode::NODEVICE)
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
            self.apps.each(|_, _, upcalls| {
                upcalls
                    .schedule_upcall(UPCALL_NUM, pin_num as usize, pin_state as usize, 0)
                    .ok();
            });
        }
    }
}

impl<'a, IP: gpio::InterruptPin<'a>> SyscallDriver for GPIO<'a, IP> {
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
    fn command(
        &self,
        command_num: usize,
        data1: usize,
        data2: usize,
        _: ProcessId,
    ) -> CommandReturn {
        let pins = self.pins.as_ref();
        let pin_index = data1;
        match command_num {
            // number of pins
            0 => CommandReturn::success_u32(pins.len() as u32),

            // enable output
            1 => {
                if pin_index >= pins.len() {
                    /* impossible pin */
                    CommandReturn::failure(ErrorCode::INVAL)
                } else {
                    if let Some(pin) = pins[pin_index] {
                        pin.make_output();
                        CommandReturn::success()
                    } else {
                        CommandReturn::failure(ErrorCode::NODEVICE)
                    }
                }
            }

            // set pin
            2 => {
                if pin_index >= pins.len() {
                    /* impossible pin */
                    CommandReturn::failure(ErrorCode::INVAL)
                } else {
                    if let Some(pin) = pins[pin_index] {
                        pin.set();
                        CommandReturn::success()
                    } else {
                        CommandReturn::failure(ErrorCode::NODEVICE)
                    }
                }
            }

            // clear pin
            3 => {
                if pin_index >= pins.len() {
                    /* impossible pin */
                    CommandReturn::failure(ErrorCode::INVAL)
                } else {
                    if let Some(pin) = pins[pin_index] {
                        pin.clear();
                        CommandReturn::success()
                    } else {
                        CommandReturn::failure(ErrorCode::NODEVICE)
                    }
                }
            }

            // toggle pin
            4 => {
                if pin_index >= pins.len() {
                    /* impossible pin */
                    CommandReturn::failure(ErrorCode::INVAL)
                } else {
                    if let Some(pin) = pins[pin_index] {
                        pin.toggle();
                        CommandReturn::success()
                    } else {
                        CommandReturn::failure(ErrorCode::NODEVICE)
                    }
                }
            }

            // enable and configure input
            5 => {
                let pin_config = data2;
                if pin_index >= pins.len() {
                    /* impossible pin */
                    CommandReturn::failure(ErrorCode::INVAL)
                } else {
                    self.configure_input_pin(pin_index as u32, pin_config)
                }
            }

            // read input
            6 => {
                if pin_index >= pins.len() {
                    /* impossible pin */
                    CommandReturn::failure(ErrorCode::INVAL)
                } else {
                    if let Some(pin) = pins[pin_index] {
                        let pin_state = pin.read();
                        CommandReturn::success_u32(pin_state as u32)
                    } else {
                        CommandReturn::failure(ErrorCode::NODEVICE)
                    }
                }
            }

            // configure interrupts on pin
            // (no affect or reliance on registered callback)
            7 => {
                let irq_config = data2;
                if pin_index >= pins.len() {
                    /* impossible pin */
                    CommandReturn::failure(ErrorCode::INVAL)
                } else {
                    self.configure_interrupt(pin_index as u32, irq_config)
                }
            }

            // disable interrupts on pin, also disables pin
            // (no affect or reliance on registered callback)
            8 => {
                if pin_index >= pins.len() {
                    /* impossible pin */
                    CommandReturn::failure(ErrorCode::INVAL)
                } else {
                    if let Some(pin) = pins[pin_index] {
                        pin.disable_interrupts();
                        pin.deactivate_to_low_power();
                        CommandReturn::success()
                    } else {
                        CommandReturn::failure(ErrorCode::NODEVICE)
                    }
                }
            }

            // disable pin
            9 => {
                if pin_index >= pins.len() {
                    /* impossible pin */
                    CommandReturn::failure(ErrorCode::INVAL)
                } else {
                    if let Some(pin) = pins[pin_index] {
                        pin.deactivate_to_low_power();
                        CommandReturn::success()
                    } else {
                        CommandReturn::failure(ErrorCode::NODEVICE)
                    }
                }
            }

            // default
            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::process::Error> {
        self.apps.enter(processid, |_, _| {})
    }
}

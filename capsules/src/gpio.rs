//! Provides userspace applications with access to GPIO pins.
//!
//! GPIOs are presented through a driver interface with synchronous commands
//! and a callback for interrupts.
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

/// Syscall driver number.
pub const DRIVER_NUM: usize = 0x00000004;

use core::cell::Cell;
use kernel::{AppId, Callback, Driver, ReturnCode};
use kernel::hil::gpio::{Pin, PinCtl, InputMode, InterruptMode, Client};

pub struct GPIO<'a, G: Pin + 'a> {
    pins: &'a [&'a G],
    callback: Cell<Option<Callback>>,
}

impl<'a, G: Pin + PinCtl> GPIO<'a, G> {
    pub fn new(pins: &'a [&'a G]) -> GPIO<'a, G> {
        GPIO {
            pins: pins,
            callback: Cell::new(None),
        }
    }

    fn configure_input_pin(&self, pin_num: usize, config: usize) -> ReturnCode {
        let pin = self.pins[pin_num];
        pin.make_input();
        match config {
            0 => {
                pin.set_input_mode(InputMode::PullUp);
                ReturnCode::SUCCESS
            }

            1 => {
                pin.set_input_mode(InputMode::PullDown);
                ReturnCode::SUCCESS
            }

            2 => {
                pin.set_input_mode(InputMode::PullNone);
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

impl<'a, G: Pin> Client for GPIO<'a, G> {
    fn fired(&self, pin_num: usize) {
        // read the value of the pin
        let pins = self.pins.as_ref();
        let pin_state = pins[pin_num].read();

        // schedule callback with the pin number and value
        if self.callback.get().is_some() {
            self.callback.get().unwrap().schedule(pin_num, pin_state as usize, 0);
        }
    }
}

impl<'a, G: Pin + PinCtl> Driver for GPIO<'a, G> {
    fn subscribe(&self, subscribe_num: usize, callback: Callback) -> ReturnCode {
        match subscribe_num {
            // subscribe to all pin interrupts
            // (no affect or reliance on individual pins being configured as interrupts)
            0 => {
                self.callback.set(Some(callback));
                ReturnCode::SUCCESS
            }

            // default
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    fn command(&self, command_num: usize, data: usize, _: AppId) -> ReturnCode {
        let pins = self.pins.as_ref();
        match command_num {
            // number of pins
            0 => ReturnCode::SuccessWithValue { value: pins.len() as usize },

            // enable output
            1 => {
                if data >= pins.len() {
                    ReturnCode::EINVAL /* impossible pin */
                } else {
                    pins[data].make_output();
                    ReturnCode::SUCCESS
                }
            }

            // set pin
            2 => {
                if data >= pins.len() {
                    ReturnCode::EINVAL /* impossible pin */
                } else {
                    pins[data].set();
                    ReturnCode::SUCCESS
                }
            }

            // clear pin
            3 => {
                if data >= pins.len() {
                    ReturnCode::EINVAL /* impossible pin */
                } else {
                    pins[data].clear();
                    ReturnCode::SUCCESS
                }
            }

            // toggle pin
            4 => {
                if data >= pins.len() {
                    ReturnCode::EINVAL /* impossible pin */
                } else {
                    pins[data].toggle();
                    ReturnCode::SUCCESS
                }
            }

            // enable and configure input
            5 => {
                // XXX: this is clunky
                // data == ((pin_config << 8) | pin)
                // this allows two values to be passed into a command interface
                let pin_num = data & 0xFF;
                let pin_config = (data >> 8) & 0xFF;
                if pin_num >= pins.len() {
                    ReturnCode::EINVAL /* impossible pin */
                } else {
                    let err_code = self.configure_input_pin(pin_num, pin_config);
                    err_code
                }
            }

            // read input
            6 => {
                if data >= pins.len() {
                    ReturnCode::EINVAL /* impossible pin */
                } else {
                    let pin_state = pins[data].read();
                    ReturnCode::SuccessWithValue { value: pin_state as usize }
                }
            }

            // enable and configure interrupts on pin, also sets pin as input
            // (no affect or reliance on registered callback)
            7 => {
                // TODO(brghena): this is clunky
                // data == ((irq_config << 16) | (pin_config << 8) | pin)
                // this allows three values to be passed into a command interface
                let pin_num = data & 0xFF;
                let pin_config = (data >> 8) & 0xFF;
                let irq_config = (data >> 16) & 0xFF;
                if pin_num >= pins.len() {
                    ReturnCode::EINVAL /* impossible pin */
                } else {
                    let mut err_code = self.configure_input_pin(pin_num, pin_config);
                    if err_code == ReturnCode::SUCCESS {
                        err_code = self.configure_interrupt(pin_num, irq_config);
                    }
                    err_code
                }
            }

            // disable interrupts on pin, also disables pin
            // (no affect or reliance on registered callback)
            8 => {
                if data >= pins.len() {
                    ReturnCode::EINVAL /* impossible pin */
                } else {
                    pins[data].disable_interrupt();
                    pins[data].disable();
                    ReturnCode::SUCCESS
                }
            }

            // disable pin
            9 => {
                if data >= pins.len() {
                    ReturnCode::EINVAL /* impossible pin */
                } else {
                    pins[data].disable();
                    ReturnCode::SUCCESS
                }
            }

            // default
            _ => ReturnCode::ENOSUPPORT,
        }
    }
}

//! Provides userspace control of buttons on a board.
//!
//! This allows for much more cross platform controlling of buttons without
//! having to know which of the GPIO pins exposed across the syscall interface
//! are buttons.
//!
//! Usage
//! -----
//!
//! ```rust
//! let button_pins = static_init!(
//!     [&'static sam4l::gpio::GPIOPin; 1],
//!     [&sam4l::gpio::PA[16]]);
//! let button = static_init!(
//!     capsules::button::Button<'static, sam4l::gpio::GPIOPin>,
//!     capsules::button::Button::new(button_pins, kernel::Grant::create()));
//! for btn in button_pins.iter() {
//!     btn.set_client(button);
//! }
//! ```
//!
//! Syscall Interface
//! -----------------
//!
//! - Stability: 2 - Stable
//!
//! ### Command
//!
//! Enable or disable button interrupts and read the current button state.
//!
//! #### `command_num`
//!
//! - `0`: Driver check and get number of buttons on the board.
//! - `1`: Enable interrupts for a given button. This will enable both press
//!   and depress events.
//! - `2`: Disable interrupts for a button. No affect or reliance on
//!   registered callback.
//! - `3`: Read the current state of the button.
//!
//! ### Subscribe
//!
//! Setup a callback for button presses.
//!
//! #### `subscribe_num`
//!
//! - `0`: Set callback for pin interrupts. Note setting this callback has
//!   no reliance on individual pins being configured as interrupts. The
//!   interrupt will be called with two parameters: the index of the button
//!   that triggered the interrupt and the pressed (1) or not pressed (0) state
//!   of the button.

use core::cell::Cell;
use kernel::hil;
use kernel::hil::gpio::{Client, InterruptMode};
use kernel::{AppId, Callback, Driver, Grant, ReturnCode};

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::BUTTON as usize;

/// This capsule keeps track for each app of which buttons it has a registered
/// interrupt for. `SubscribeMap` is a bit array where bits are set to one if
/// that app has an interrupt registered for that button.
pub type SubscribeMap = u32;

/// Whether the GPIOs for the buttons on this platform are low when the button
/// is pressed or high.
#[derive(Clone, Copy)]
pub enum GpioMode {
    LowWhenPressed,
    HighWhenPressed,
}

/// Values that are passed to userspace to identify if the button is pressed
/// or not.
#[derive(Clone, Copy)]
pub enum ButtonState {
    NotPressed = 0,
    Pressed = 1,
}

/// Manages the list of GPIO pins that are connected to buttons and which apps
/// are listening for interrupts from which buttons.
pub struct Button<'a, G: hil::gpio::Pin> {
    pins: &'a [(&'a G, GpioMode)],
    apps: Grant<(Option<Callback>, SubscribeMap)>,
}

impl<G: hil::gpio::Pin + hil::gpio::PinCtl> Button<'a, G> {
    pub fn new(
        pins: &'a [(&'a G, GpioMode)],
        grant: Grant<(Option<Callback>, SubscribeMap)>,
    ) -> Button<'a, G> {
        for &(pin, _) in pins.iter() {
            pin.make_input();
        }

        Button {
            pins: pins,
            apps: grant,
        }
    }

    fn get_button_state(&self, pin_num: usize) -> ButtonState {
        let pin_value = self.pins[pin_num].0.read();
        match self.pins[pin_num].1 {
            GpioMode::LowWhenPressed => match pin_value {
                false => ButtonState::Pressed,
                true => ButtonState::NotPressed,
            },
            GpioMode::HighWhenPressed => match pin_value {
                false => ButtonState::NotPressed,
                true => ButtonState::Pressed,
            },
        }
    }
}

impl<G: hil::gpio::Pin + hil::gpio::PinCtl> Driver for Button<'a, G> {
    /// Set callbacks.
    ///
    /// ### `subscribe_num`
    ///
    /// - `0`: Set callback for pin interrupts. Note setting this callback has
    ///   no reliance on individual pins being configured as interrupts. The
    ///   interrupt will be called with two parameters: the index of the button
    ///   that triggered the interrupt and the pressed/not pressed state of the
    ///   button.
    fn subscribe(
        &self,
        subscribe_num: usize,
        callback: Option<Callback>,
        app_id: AppId,
    ) -> ReturnCode {
        match subscribe_num {
            0 => self
                .apps
                .enter(app_id, |cntr, _| {
                    cntr.0 = callback;
                    ReturnCode::SUCCESS
                })
                .unwrap_or_else(|err| err.into()),

            // default
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    /// Configure interrupts and read state for buttons.
    ///
    /// `data` is the index of the button in the button array as passed to
    /// `Button::new()`.
    ///
    /// All commands greater than zero return `EINVAL` if an invalid button
    /// number is passed in.
    ///
    /// ### `command_num`
    ///
    /// - `0`: Driver check and get number of buttons on the board.
    /// - `1`: Enable interrupts for a given button. This will enable both press
    ///   and depress events.
    /// - `2`: Disable interrupts for a button. No affect or reliance on
    ///   registered callback.
    /// - `3`: Read the current state of the button.
    fn command(&self, command_num: usize, data: usize, _: usize, appid: AppId) -> ReturnCode {
        let pins = self.pins;
        match command_num {
            // return button count
            0 => ReturnCode::SuccessWithValue {
                value: pins.len() as usize,
            },

            // enable interrupts for a button
            1 => {
                if data < pins.len() {
                    self.apps
                        .enter(appid, |cntr, _| {
                            cntr.1 |= 1 << data;
                            pins[data]
                                .0
                                .enable_interrupt(data, InterruptMode::EitherEdge);
                            ReturnCode::SUCCESS
                        })
                        .unwrap_or_else(|err| err.into())
                } else {
                    ReturnCode::EINVAL /* impossible button */
                }
            }

            // disable interrupts for a button
            2 => {
                if data >= pins.len() {
                    ReturnCode::EINVAL /* impossible button */
                } else {
                    let res = self
                        .apps
                        .enter(appid, |cntr, _| {
                            cntr.1 &= !(1 << data);
                            ReturnCode::SUCCESS
                        })
                        .unwrap_or_else(|err| err.into());

                    // are any processes waiting for this button?
                    let interrupt_count = Cell::new(0);
                    self.apps.each(|cntr| {
                        cntr.0.map(|_| {
                            if cntr.1 & (1 << data) != 0 {
                                interrupt_count.set(interrupt_count.get() + 1);
                            }
                        });
                    });

                    // if not, disable the interrupt
                    if interrupt_count.get() == 0 {
                        self.pins[data].0.disable_interrupt();
                    }

                    res
                }
            }

            // read input
            3 => {
                if data >= pins.len() {
                    ReturnCode::EINVAL /* impossible button */
                } else {
                    let button_state = self.get_button_state(data);
                    ReturnCode::SuccessWithValue {
                        value: button_state as usize,
                    }
                }
            }

            // default
            _ => ReturnCode::ENOSUPPORT,
        }
    }
}

impl<G: hil::gpio::Pin + hil::gpio::PinCtl> Client for Button<'a, G> {
    fn fired(&self, pin_num: usize) {
        // Read the value of the pin and get the button state.
        let button_state = self.get_button_state(pin_num);
        let interrupt_count = Cell::new(0);

        // schedule callback with the pin number and value
        self.apps.each(|cntr| {
            cntr.0.map(|mut callback| {
                if cntr.1 & (1 << pin_num) != 0 {
                    interrupt_count.set(interrupt_count.get() + 1);
                    callback.schedule(pin_num, button_state as usize, 0);
                }
            });
        });

        // It's possible we got an interrupt for a process that has since died
        // (and didn't unregister the interrupt). Lazily disable interrupts for
        // this button if so.
        if interrupt_count.get() == 0 {
            self.pins[pin_num].0.disable_interrupt();
        }
    }
}

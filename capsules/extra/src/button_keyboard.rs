// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! Mimic buttons using keyboard presses.
//!
//! This implements the same `Driver` interface as the normal button capsule,
//! but instead of using GPIO pins as the underlying source for the buttons
//! it uses keyboard key presses.

use kernel::grant::{AllowRoCount, AllowRwCount, Grant, UpcallCount};
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::{ErrorCode, ProcessId};

/// Syscall driver number.
use capsules_core::driver;
pub const DRIVER_NUM: usize = driver::NUM::Button as usize;

/// Keeps track for each app of which buttons it has a registered interrupt
/// for.
///
/// `subscribe_map` is a bit array where bits are set to one if that app has an
/// interrupt registered for that button.
#[derive(Default)]
pub struct App {
    subscribe_map: u32,
}

/// Manages the list of GPIO pins that are connected to buttons and which apps
/// are listening for interrupts from which buttons.
pub struct ButtonKeyboard<'a> {
    /// The key codes we are looking for to map to buttons. These are in order,
    /// e.g., the second key code in the array maps to button with index 1.
    key_codes: &'a [u16],
    apps: Grant<App, UpcallCount<1>, AllowRoCount<0>, AllowRwCount<0>>,
}

impl<'a> ButtonKeyboard<'a> {
    pub fn new(
        key_codes: &'a [u16],
        grant: Grant<App, UpcallCount<1>, AllowRoCount<0>, AllowRwCount<0>>,
    ) -> Self {
        if key_codes.len() >= 32 {
            panic!("ButtonKeyboard capsule only supports up to 32 buttons.");
        }
        Self {
            key_codes,
            apps: grant,
        }
    }
}

/// ### `subscribe_num`
///
/// - `0`: Set callback for pin interrupts.
const UPCALL_NUM: usize = 0;

impl SyscallDriver for ButtonKeyboard<'_> {
    fn command(
        &self,
        command_num: usize,
        data: usize,
        _: usize,
        processid: ProcessId,
    ) -> CommandReturn {
        match command_num {
            // return button count
            // TODO(Tock 3.0): TRD104 specifies that Command 0 should return Success, not SuccessU32,
            // but this driver is unchanged since it has been stabilized. It will be brought into
            // compliance as part of the next major release of Tock. See #3375.
            0 => CommandReturn::success_u32(self.key_codes.len() as u32),

            // enable interrupts for a button
            1 => {
                if data >= self.key_codes.len() {
                    CommandReturn::failure(ErrorCode::INVAL)
                } else {
                    self.apps
                        .enter(processid, |app, _| {
                            app.subscribe_map |= 1 << data;
                            CommandReturn::success()
                        })
                        .unwrap_or_else(|err| CommandReturn::failure(err.into()))
                }
            }

            // disable interrupts for a button
            2 => {
                if data >= self.key_codes.len() {
                    CommandReturn::failure(ErrorCode::INVAL)
                } else {
                    self.apps
                        .enter(processid, |app, _| {
                            app.subscribe_map &= !(1 << data);
                            CommandReturn::success()
                        })
                        .unwrap_or_else(|err| CommandReturn::failure(err.into()))
                }
            }

            // read input
            3 => {
                if data >= self.key_codes.len() {
                    CommandReturn::failure(ErrorCode::INVAL)
                } else {
                    // Always return not pressed
                    CommandReturn::success_u32(0)
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

impl kernel::hil::keyboard::KeyboardClient for ButtonKeyboard<'_> {
    fn keys_pressed(&self, keys: &[(u16, bool)], result: Result<(), ErrorCode>) {
        if result.is_ok() {
            // Iterate all key presses we received.
            for (key, is_pressed) in keys.iter() {
                // Iterate through all of the keys we are looking for.
                for (active_key_index, active_key) in self.key_codes.iter().enumerate() {
                    // If there is a match then we may want to handle this key.
                    if key == active_key {
                        kernel::debug!(
                            "[ButtonKeyboard] Notify button {} (key {})",
                            active_key_index,
                            key
                        );

                        // Schedule callback for apps waiting on that key.
                        self.apps.each(|_, app, upcalls| {
                            if app.subscribe_map & (1 << active_key_index) != 0 {
                                let button_state = usize::from(*is_pressed);
                                let _ = upcalls.schedule_upcall(
                                    UPCALL_NUM,
                                    (active_key_index, button_state, 0),
                                );
                            }
                        });
                    }
                }
            }
        }
    }
}

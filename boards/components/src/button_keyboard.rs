// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! Component for buttons using keyboard presses.

use core::mem::MaybeUninit;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil;

#[macro_export]
macro_rules! keyboard_button_component_static {
    () => {{
        kernel::static_buf!(capsules_extra::button_keyboard::ButtonKeyboard<'static>)
    };};
}

pub type KeyboardButtonComponentType = capsules_extra::button_keyboard::ButtonKeyboard<'static>;

pub struct KeyboardButtonComponent<K: 'static + hil::keyboard::Keyboard<'static>> {
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
    keyboard: &'static K,
    key_codes: &'static [u16],
}

impl<K: 'static + hil::keyboard::Keyboard<'static>> KeyboardButtonComponent<K> {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        driver_num: usize,
        keyboard: &'static K,
        key_codes: &'static [u16],
    ) -> Self {
        Self {
            board_kernel,
            driver_num,
            keyboard,
            key_codes,
        }
    }
}

impl<K: 'static + hil::keyboard::Keyboard<'static>> Component for KeyboardButtonComponent<K> {
    type StaticInput =
        &'static mut MaybeUninit<capsules_extra::button_keyboard::ButtonKeyboard<'static>>;
    type Output = &'static capsules_extra::button_keyboard::ButtonKeyboard<'static>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let button_keyboard = s.write(capsules_extra::button_keyboard::ButtonKeyboard::new(
            self.key_codes,
            self.board_kernel.create_grant(self.driver_num, &grant_cap),
        ));
        self.keyboard.set_client(button_keyboard);

        button_keyboard
    }
}

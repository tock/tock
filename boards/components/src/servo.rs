// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! Components for collections of servomotors.
//!
//! Usage
//! -----
//! ```rust
//! let servo = components::servo::ServosComponent::new().finalize(components::servo_component_static!(
//!     servo1, servo2,
//! ));
//! ```
use capsules_extra::servo::Servo as ServoDriver;
use core::mem::MaybeUninit;
use kernel::component::Component;

#[macro_export]
macro_rules! servo_component_static {
    ($($S:expr),+ $(,)?) => {{
        use kernel::count_expressions;
        use kernel::static_init;
        const SERVO_COUNT: usize = count_expressions!($($S),+);
        let arr = static_init!(
            [&'static dyn Servo; SERVO_COUNT],
            [
                $(

                        $S

                ),+
            ]
        );

        let servo = kernel::static_buf!( capsules_extra::servo::Servo<'static, SERVO_COUNT>);
        (servo, arr)
    };};
}

pub type ServosComponentType<const SERVO_COUNT: usize> = ServoDriver<'static, SERVO_COUNT>;

pub struct ServosComponent<const SERVO_COUNT: usize> {}

impl<const SERVO_COUNT: usize> ServosComponent<SERVO_COUNT> {
    pub fn new() -> Self {
        Self {}
    }
}

impl<const SERVO_COUNT: usize> Component for ServosComponent<SERVO_COUNT> {
    type StaticInput = (
        &'static mut MaybeUninit<ServoDriver<'static, SERVO_COUNT>>,
        &'static mut [&'static dyn kernel::hil::servo::Servo<'static>; SERVO_COUNT],
    );
    type Output = &'static ServoDriver<'static, SERVO_COUNT>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        static_buffer.0.write(ServoDriver::new(static_buffer.1))
    }
}

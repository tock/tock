// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Component for a PWM-based buzzer driver.
//!
//! This creates a `Buzzer` syscall driver using a PWM pin and a virtual alarm.
//! It internally allocates the `VirtualMuxAlarm`, `PwmBuzzer`, and `Buzzer`
//! objects.
//!
//! Usage
//! -----
//! ```rust
//! let buzzer = components::buzzer::BuzzerComponent::new(
//!     board_kernel,
//!     capsules_extra::buzzer_driver::DRIVER_NUM,
//!     mux_alarm,
//!     virtual_pwm_buzzer,
//! )
//! .finalize(components::buzzer_component_static!(AlarmHw, PwmHw));
//! ```

use capsules_core::virtualizers::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use capsules_core::virtualizers::virtual_pwm::PwmPinUser;
use capsules_extra::buzzer_driver::Buzzer;
use capsules_extra::buzzer_pwm::PwmBuzzer;
use core::mem::MaybeUninit;
use kernel::capabilities::MemoryAllocationCapability;
use kernel::component::Component;
use kernel::hil;

#[macro_export]
macro_rules! buzzer_component_static {
    ($A:ty, $P:ty $(,)?) => {{
        use capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm;
        use capsules_core::virtualizers::virtual_pwm::PwmPinUser;
        use capsules_extra::buzzer_driver::Buzzer;
        use capsules_extra::buzzer_pwm::PwmBuzzer;
        let alarm = kernel::static_buf!(VirtualMuxAlarm<'static, $A>);
        let pwm_buzzer = kernel::static_buf!(
            PwmBuzzer<'static, VirtualMuxAlarm<'static, $A>, PwmPinUser<'static, $P>>
        );
        let buzzer = kernel::static_buf!(
            Buzzer<
                'static,
                PwmBuzzer<'static, VirtualMuxAlarm<'static, $A>, PwmPinUser<'static, $P>>,
            >
        );
        (alarm, pwm_buzzer, buzzer)
    };};
}

pub type BuzzerComponentType<A, P> = capsules_extra::buzzer_driver::Buzzer<
    'static,
    capsules_extra::buzzer_pwm::PwmBuzzer<
        'static,
        capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm<'static, A>,
        capsules_core::virtualizers::virtual_pwm::PwmPinUser<'static, P>,
    >,
>;

pub struct BuzzerComponent<
    A: 'static + hil::time::Alarm<'static>,
    P: 'static + hil::pwm::Pwm,
    CAP: MemoryAllocationCapability + 'static,
> {
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
    mux_alarm: &'static MuxAlarm<'static, A>,
    pwm_pin: &'static PwmPinUser<'static, P>,
    mem_cap: CAP,
}

impl<
        A: 'static + hil::time::Alarm<'static>,
        P: 'static + hil::pwm::Pwm,
        CAP: MemoryAllocationCapability + 'static,
    > BuzzerComponent<A, P, CAP>
{
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        driver_num: usize,
        mux_alarm: &'static MuxAlarm<'static, A>,
        pwm_pin: &'static PwmPinUser<'static, P>,
        mem_cap: CAP,
    ) -> Self {
        Self {
            board_kernel,
            driver_num,
            mux_alarm,
            pwm_pin,
            mem_cap,
        }
    }
}

impl<
        A: 'static + hil::time::Alarm<'static>,
        P: 'static + hil::pwm::Pwm,
        CAP: MemoryAllocationCapability + 'static,
    > Component for BuzzerComponent<A, P, CAP>
{
    type StaticInput = (
        &'static mut MaybeUninit<VirtualMuxAlarm<'static, A>>,
        &'static mut MaybeUninit<
            PwmBuzzer<'static, VirtualMuxAlarm<'static, A>, PwmPinUser<'static, P>>,
        >,
        &'static mut MaybeUninit<
            Buzzer<
                'static,
                PwmBuzzer<'static, VirtualMuxAlarm<'static, A>, PwmPinUser<'static, P>>,
            >,
        >,
    );
    type Output = &'static Buzzer<
        'static,
        PwmBuzzer<'static, VirtualMuxAlarm<'static, A>, PwmPinUser<'static, P>>,
    >;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let virtual_alarm_buzzer = static_buffer.0.write(VirtualMuxAlarm::new(self.mux_alarm));
        virtual_alarm_buzzer.setup();

        let pwm_buzzer = static_buffer.1.write(PwmBuzzer::new(
            self.pwm_pin,
            virtual_alarm_buzzer,
            capsules_extra::buzzer_pwm::DEFAULT_MAX_BUZZ_TIME_MS,
        ));

        let buzzer = static_buffer.2.write(Buzzer::new(
            pwm_buzzer,
            capsules_extra::buzzer_driver::DEFAULT_MAX_BUZZ_TIME_MS,
            self.board_kernel
                .create_grant(self.driver_num, &self.mem_cap),
        ));

        hil::buzzer::Buzzer::set_client(pwm_buzzer, buzzer);
        hil::time::Alarm::set_alarm_client(virtual_alarm_buzzer, pwm_buzzer);

        buzzer
    }
}

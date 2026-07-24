// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Component for any Sound Pressure sensor.
//!
//! Usage
//! -----
//! ```rust
//! let sound_pressure = SoundPressureComponent::new(board_kernel, adc_microphone)
//!     .finalize(sound_pressure_component_static!());
//! ```

use capsules_extra::sound_pressure::SoundPressureSensor;
use core::mem::MaybeUninit;
use kernel::capabilities::MemoryAllocationCapability;
use kernel::component::Component;
use kernel::hil;

#[macro_export]
macro_rules! sound_pressure_component_static {
    () => {{
        kernel::static_buf!(capsules_extra::sound_pressure::SoundPressureSensor<'static>)
    };};
}

pub type SoundPressureComponentType = capsules_extra::sound_pressure::SoundPressureSensor<'static>;

pub struct SoundPressureComponent<
    S: 'static + hil::sensors::SoundPressure<'static>,
    CAP: MemoryAllocationCapability + 'static,
> {
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
    sound_sensor: &'static S,
    mem_cap: CAP,
}

impl<S: 'static + hil::sensors::SoundPressure<'static>, CAP: MemoryAllocationCapability + 'static>
    SoundPressureComponent<S, CAP>
{
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        driver_num: usize,
        sound_sensor: &'static S,
        mem_cap: CAP,
    ) -> SoundPressureComponent<S, CAP> {
        SoundPressureComponent {
            board_kernel,
            driver_num,
            sound_sensor,
            mem_cap,
        }
    }
}

impl<S: 'static + hil::sensors::SoundPressure<'static>, CAP: MemoryAllocationCapability + 'static>
    Component for SoundPressureComponent<S, CAP>
{
    type StaticInput = &'static mut MaybeUninit<SoundPressureSensor<'static>>;
    type Output = &'static SoundPressureSensor<'static>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let sound_pressure = s.write(SoundPressureSensor::new(
            self.sound_sensor,
            self.board_kernel
                .create_grant(self.driver_num, &self.mem_cap),
        ));

        hil::sensors::SoundPressure::set_client(self.sound_sensor, sound_pressure);
        sound_pressure
    }
}

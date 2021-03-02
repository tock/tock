//! Component for any Sound Pressure sensor.
//!
//! Usage
//! -----
//! ```rust
//! let sound_pressure = SoundPressureComponent::new(board_kernel, adc_microphone).finalize(());
//! ```

use capsules::sound_pressure::SoundPressureSensor;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil;
use kernel::static_init;

pub struct SoundPressureComponent<S: 'static + hil::sensors::SoundPressure<'static>> {
    board_kernel: &'static kernel::Kernel,
    driver_num: u32,
    sound_sensor: &'static S,
}

impl<S: 'static + hil::sensors::SoundPressure<'static>> SoundPressureComponent<S> {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        driver_num: u32,
        sound_sensor: &'static S,
    ) -> SoundPressureComponent<S> {
        SoundPressureComponent {
            board_kernel,
            driver_num,
            sound_sensor,
        }
    }
}

impl<S: 'static + hil::sensors::SoundPressure<'static>> Component for SoundPressureComponent<S> {
    type StaticInput = ();
    type Output = &'static SoundPressureSensor<'static>;

    unsafe fn finalize(self, _s: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let sound_pressure = static_init!(
            capsules::sound_pressure::SoundPressureSensor<'static>,
            capsules::sound_pressure::SoundPressureSensor::new(
                self.sound_sensor,
                self.board_kernel.create_grant(self.driver_num, &grant_cap)
            )
        );

        hil::sensors::SoundPressure::set_client(self.sound_sensor, sound_pressure);
        sound_pressure
    }
}

//! Service capsule for a buzzer that uses a PWM pin.
//!
//! ## Instantiation
//!
//! Instantiate the capsule for use as a service capsule, using a virtual pwm buzzer
//! and a virtual alarm. For example:
//!
//! ```rust
//! # use kernel::static_init;
//!
//! let mux_pwm = static_init!(
//!     capsules::virtual_pwm::MuxPwm<'static, nrf52833::pwm::Pwm>,
//!     capsules::virtual_pwm::MuxPwm::new(&base_peripherals.pwm0)
//! );
//! let virtual_pwm_buzzer = static_init!(
//!     capsules::virtual_pwm::PwmPinUser<'static, nrf52833::pwm::Pwm>,
//!     capsules::virtual_pwm::PwmPinUser::new(
//!         mux_pwm,
//!         nrf52833::pinmux::Pinmux::new(SPEAKER_PIN as u32)
//!     )
//! );
//! virtual_pwm_buzzer.add_to_mux();
//!
//! let virtual_alarm_buzzer = static_init!(
//!     capsules::virtual_alarm::VirtualMuxAlarm<'static, nrf52833::rtc::Rtc>,
//!     capsules::virtual_alarm::VirtualMuxAlarm::new(mux_alarm)
//! );
//! virtual_alarm_buzzer.setup();
//!
//! let pwm_buzzer = static_init!(
//!     capsules::buzzer_pwm::PwmBuzzer<
//!         'static,
//!         capsules::virtual_alarm::VirtualMuxAlarm<'static, nrf52833::rtc::Rtc>,
//!         capsules::virtual_pwm::PwmPinUser<'static, nrf52833::pwm::Pwm>,
//!     >,
//!     capsules::buzzer_pwm::PwmBuzzer::new(
//!         virtual_pwm_buzzer,
//!         virtual_alarm_buzzer,
//!         capsules::buzzer_pwm::DEFAULT_MAX_BUZZ_TIME_MS,
//!     )
//! );
//!
//! pwm_buzzer.set_client(buzzer);
//!
//! virtual_alarm_buzzer.set_alarm_client(pwm_buzzer);
//!
//! ```

use core::cmp;

use kernel::hil;
use kernel::hil::buzzer::BuzzerClient;
use kernel::hil::time::Frequency;
use kernel::utilities::cells::OptionalCell;
use kernel::ErrorCode;

/// Standard max buzz time.
pub const DEFAULT_MAX_BUZZ_TIME_MS: usize = 5000;

pub struct PwmBuzzer<'a, A: hil::time::Alarm<'a>, P: hil::pwm::PwmPin> {
    /// The underlying PWM generator to make the buzzer buzz.
    pwm_pin: &'a P,
    /// Alarm to stop the PWM after some time.
    alarm: &'a A,
    /// Max buzz time.
    max_duration_ms: usize,
    /// The client currently using the service capsule.
    client: OptionalCell<&'a dyn BuzzerClient>,
}

impl<'a, A: hil::time::Alarm<'a>, P: hil::pwm::PwmPin> PwmBuzzer<'a, A, P> {
    pub fn new(pwm_pin: &'a P, alarm: &'a A, max_duration_ms: usize) -> PwmBuzzer<'a, A, P> {
        PwmBuzzer {
            pwm_pin: pwm_pin,
            alarm: alarm,
            client: OptionalCell::empty(),
            max_duration_ms: max_duration_ms,
        }
    }
}

impl<'a, A: hil::time::Alarm<'a>, P: hil::pwm::PwmPin> hil::buzzer::Buzzer<'a>
    for PwmBuzzer<'a, A, P>
{
    fn set_client(&self, client: &'a dyn BuzzerClient) {
        self.client.replace(client);
    }

    fn buzz(&self, frequency_hz: usize, duration_ms: usize) -> Result<(), ErrorCode> {
        let duration_ms_cmp = cmp::min(duration_ms, self.max_duration_ms);
        self.pwm_pin
            .start(frequency_hz, self.pwm_pin.get_maximum_duty_cycle() / 2)?;

        // Set an alarm for the given duration.
        let interval = (duration_ms_cmp as u32) * <A::Frequency>::frequency() / 1000;
        self.alarm
            .set_alarm(self.alarm.now(), A::Ticks::from(interval));
        Ok(())
    }

    fn stop(&self) -> Result<(), ErrorCode> {
        // Disarm the current alarm and instantly fire another.
        self.alarm.disarm()?;
        // This method was used to reduce the size of the code.
        self.alarm.set_alarm(self.alarm.now(), A::Ticks::from(0));
        Ok(())
    }
}

impl<'a, A: hil::time::Alarm<'a>, P: hil::pwm::PwmPin> hil::time::AlarmClient
    for PwmBuzzer<'a, A, P>
{
    fn alarm(&self) {
        // Stop the pin output and signal that the buzzer has finished
        // playing.
        self.client
            .map(|buzz_client| buzz_client.buzzer_done(self.pwm_pin.stop()));
    }
}

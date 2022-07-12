//! Service capsule for a buzzer that uses a PWM pin.

use core::cmp;

use kernel::hil;
use kernel::hil::buzzer::BuzzerClient;
use kernel::hil::buzzer::BuzzerCommand;
use kernel::hil::time::Frequency;
use kernel::utilities::cells::OptionalCell;
use kernel::ErrorCode;

// Standard max buzz time.
pub const DEFAULT_MAX_BUZZ_TIME_MS: usize = 5000;

pub struct PwmBuzzer<'a, A: hil::time::Alarm<'a>, P: hil::pwm::PwmPin> {
    // The underlying PWM generator to make the buzzer buzz.
    pwm_pin: &'a P,
    /// Alarm to stop the PWM after some time.
    alarm: &'a A,
    // Max buzz time.
    max_duration_ms: usize,
    // The client currently using the service capsule.
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
    // Set the current client.
    fn set_client(&self, client: &'a dyn BuzzerClient) {
        self.client.replace(client);
    }

    // Play the sound.
    fn buzz(&self, command: BuzzerCommand) -> Result<(), ErrorCode> {
        match command {
            BuzzerCommand::Buzz {
                frequency_hz,
                duration_ms,
            } => {
                let duration_ms_cmp = cmp::min(duration_ms, self.max_duration_ms);
                let ret = self
                    .pwm_pin
                    .start(frequency_hz, self.pwm_pin.get_maximum_duty_cycle() / 2);

                // If starting the pin output failed, return the error.
                if ret != Ok(()) {
                    return ret;
                }

                // Set an alarm for the given duration.
                let interval = (duration_ms_cmp as u32) * <A::Frequency>::frequency() / 1000;
                self.alarm
                    .set_alarm(self.alarm.now(), A::Ticks::from(interval));
                Ok(())
            }
        }
    }

    fn stop(&self) -> Result<(), ErrorCode> {
        // Clear the current alarm and instantly fire another.
        let ret = self.alarm.disarm();
        self.alarm.set_alarm(self.alarm.now(), A::Ticks::from(0));
        ret
    }
}

impl<'a, A: hil::time::Alarm<'a>, P: hil::pwm::PwmPin> hil::time::AlarmClient
    for PwmBuzzer<'a, A, P>
{
    fn alarm(&self) {
        // Stop the pin output and signal that the buzzer has finished
        // playing.
        let ret = self.pwm_pin.stop();
        self.client.map(|buzz_client| buzz_client.buzzer_done(ret));
    }
}

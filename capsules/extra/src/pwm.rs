use kernel::grant::{AllowRoCount, AllowRwCount, Grant, UpcallCount};
use kernel::hil;
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::utilities::cells::OptionalCell;
use kernel::{ErrorCode, ProcessId};

/// Syscall driver number.
use core_capsules::driver;
pub const DRIVER_NUM: usize = driver::NUM::Pwm as usize;

// An empty app, for potential uses in future updates of the driver
#[derive(Default)]
pub struct App;

pub struct Pwm<'a, const NUM_PINS: usize> {
    /// The usable pwm pins.
    pwm_pins: &'a [&'a dyn hil::pwm::PwmPin; NUM_PINS],
    /// Per-app state.
    apps: Grant<App, UpcallCount<1>, AllowRoCount<0>, AllowRwCount<0>>,
    /// An array of apps associated to their reserved pins.
    active_process: [OptionalCell<ProcessId>; NUM_PINS],
}

impl<'a, const NUM_PINS: usize> Pwm<'a, NUM_PINS> {
    pub fn new(
        pwm_pins: &'a [&'a dyn hil::pwm::PwmPin; NUM_PINS],
        grant: Grant<App, UpcallCount<1>, AllowRoCount<0>, AllowRwCount<0>>,
    ) -> Pwm<'a, NUM_PINS> {
        assert!(NUM_PINS <= (u16::MAX as usize));
        const EMPTY: OptionalCell<ProcessId> = OptionalCell::empty();
        Pwm {
            pwm_pins: pwm_pins,
            apps: grant,
            active_process: [EMPTY; NUM_PINS],
        }
    }

    pub fn claim_pin(&self, processid: ProcessId, pin: usize) -> bool {
        // Attempt to get the app that is using the pin.
        self.active_process[pin].map_or(true, |id| {
            // If the app is empty, that means that there is no app currently using this pin,
            // therefore the pin could be usable by the new app
            if id == &processid {
                // The same app is trying to access the pin it has access to, valid
                true
            } else {
                // An app is trying to access another app's pin, invalid
                false
            }
        })
    }

    pub fn release_pin(&self, pin: usize) {
        // Release the claimed pin so that it can now be used by another process.
        self.active_process[pin].clear();
    }
}

/// Provide an interface for userland.
impl<'a, const NUM_PINS: usize> SyscallDriver for Pwm<'a, NUM_PINS> {
    /// Command interface.
    ///
    /// ### `command_num`
    ///
    /// - `0`: Return number of PWM pins if this driver is included on the platform.
    /// - `1`: Start the PWM pin output. First 16 bits of `data1` are used for the duty cycle, as a
    ///     percentage with 2 decimals, and the last 16 bits of `data1` are used for the PWM channel
    ///     to be controlled. `data2` is used for the frequency in hertz. For the duty cycle, 100% is
    ///     the max duty cycle for this pin.
    /// - `2`: Stop the PWM output.
    /// - `3`: Return the maximum possible frequency for this pin.
    fn command(
        &self,
        command_num: usize,
        data1: usize,
        data2: usize,
        processid: ProcessId,
    ) -> CommandReturn {
        match command_num {
            // Return number of usable PWM pins.
            0 => CommandReturn::success_u32(NUM_PINS as u32),

            // Start the pwm output.

            // data1 stores the duty cycle and the pin number in the format
            // +------------------+------------------+
            // | duty cycle (u16) |   pwm pin (u16)  |
            // +------------------+------------------+
            // This format was chosen because there are only 2 parameters in the command function that can be used for storing values,
            // but in this case, 3 values are needed (pin, frequency, duty cycle), so data1 stores two of these values that can be
            // represented using only 16 bits.
            1 => {
                let pin = data1 & ((1 << 16) - 1);
                let duty_cycle = data1 >> 16;
                let frequency_hz = data2;

                if pin >= NUM_PINS {
                    // App asked to use a pin that doesn't exist.
                    CommandReturn::failure(ErrorCode::INVAL)
                } else {
                    if !self.claim_pin(processid, pin) {
                        // App cannot claim pin.
                        CommandReturn::failure(ErrorCode::RESERVE)
                    } else {
                        // App can claim pin, start pwm pin at given frequency and duty_cycle.
                        self.active_process[pin].set(processid);
                        // Duty cycle is represented as a 4 digit number, so we divide by 10000 to get the percentage of the max duty cycle.
                        // e.g.: a duty cycle of 60.5% is represented as 6050, so the actual value of the duty cycle is
                        // 6050 * max_duty_cycle / 10000 = 0.605 * max_duty_cycle
                        self.pwm_pins[pin]
                            .start(
                                frequency_hz,
                                duty_cycle * self.pwm_pins[pin].get_maximum_duty_cycle() / 10000,
                            )
                            .into()
                    }
                }
            }

            // Stop the PWM output.
            2 => {
                let pin = data1;
                if pin >= NUM_PINS {
                    // App asked to use a pin that doesn't exist.
                    CommandReturn::failure(ErrorCode::INVAL)
                } else {
                    if !self.claim_pin(processid, pin) {
                        // App cannot claim pin.
                        CommandReturn::failure(ErrorCode::RESERVE)
                    } else if self.active_process[pin].is_none() {
                        // If there is no active app, the pwm pin isn't in use.
                        CommandReturn::failure(ErrorCode::OFF)
                    } else {
                        // Release the pin and stop pwm output.
                        self.release_pin(pin);
                        self.pwm_pins[pin].stop().into()
                    }
                }
            }

            // Get max frequency of pin.
            3 => {
                let pin = data1;
                if pin >= NUM_PINS {
                    CommandReturn::failure(ErrorCode::INVAL)
                } else {
                    CommandReturn::success_u32(self.pwm_pins[pin].get_maximum_frequency_hz() as u32)
                }
            }

            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::process::Error> {
        self.apps.enter(processid, |_, _| {})
    }
}

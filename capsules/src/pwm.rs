//! ```
//! Provides userspace applications with the ability to generate PWM Signals
//! ```

use kernel::hil;
use kernel::{AppId, AppSlice, Callback, Driver, ReturnCode, Shared};

/// Syscall driver number.
pub const DRIVER_NUM: usize = 0x00DE0005;

pub struct Pwm<'a, S: hil::pwm::Signal> {
    base_freq: usize,
    signals: &'a [S],
}

impl<'a, S: hil::pwm::Signal> Pwm<'a, S> {
    pub fn new(base_freq: usize, signals: &'a [S]) -> Pwm<'a, S> {
        Pwm { base_freq, signals }
    }
}

use enum_primitive::cast::FromPrimitive;

enum_from_primitive!{
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum CMD {
    PING = 0,
    CONFIGURE = 1,
    SET_FREQ_HZ = 2,
    SET_DUTY = 3,
    NUM_SIGNALS = 4,
}
}

impl<'a, S: hil::pwm::Signal> Driver for Pwm<'a, S> {
    fn allow(
        &self,
        _appid: AppId,
        _allow_num: usize,
        _slice: Option<AppSlice<Shared, u8>>,
    ) -> ReturnCode {
        // there is currently no concept of granting memory to PWM
        ReturnCode::ENOSUPPORT
    }

    fn subscribe(
        &self,
        _subscribe_num: usize,
        _callback: Option<Callback>,
        _app_id: AppId,
    ) -> ReturnCode {
        // there is currently no concept of subscribing to PWM
        ReturnCode::ENOSUPPORT
    }

    fn command(&self, cmd_num: usize, arg1: usize, arg2: usize, _appid: AppId) -> ReturnCode {
        if let Some(cmd) = CMD::from_usize(cmd_num) {
            match cmd {
                CMD::PING => ReturnCode::SUCCESS,
                CMD::CONFIGURE => {
                    let signal_index = arg1;
                    if signal_index > self.signals.len() {
                        return ReturnCode::EINVAL;
                    }

                    let period: u16 = (arg2 >> 16) as u16;
                    let on_period: u16 = arg2 as u16;
                    self.signals[signal_index].configure(period, on_period);
                    ReturnCode::SUCCESS
                }
                CMD::SET_FREQ_HZ => {
                    let signal_index = arg1;
                    if signal_index > self.signals.len() {
                        return ReturnCode::EINVAL;
                    }

                    let freq_hz = arg2;

                    // parameter given is too large
                    if freq_hz > self.base_freq {
                        return ReturnCode::ESIZE;
                    }

                    // parameter given is too small
                    let period: usize = self.base_freq / freq_hz;
                    if period & 0xFFFF0000 != 0 {
                        return ReturnCode::EINVAL;
                    }

                    let period: u16 = period as u16;
                    let on_period = period >> 1;
                    self.signals[signal_index].configure(period, on_period);
                    ReturnCode::SUCCESS
                }
                CMD::SET_DUTY => {
                    let signal_index = arg1;
                    if signal_index > self.signals.len() {
                        return ReturnCode::EINVAL;
                    }

                    let duty_cycle: f32 = arg2 as f32;

                    // parameter given is too large
                    if duty_cycle > 100.0 {
                        return ReturnCode::ESIZE;
                    }

                    // we are going to do play games to get accuracy from the 24 bit expansion
                    // exp will accumulate the exponent of the floating point representaiton
                    let mut exp = duty_cycle as u32;
                    for _n in 0..3 {
                        // by multipling the float by 100, we bring convert some bits of the fractional float
                        // to the exponent bits and by casting to u32, we extact the exponent
                        let extract = ((duty_cycle - exp as f32) * 100.0) as u32;
                        // update the exponent
                        exp = exp * 100 + extract;
                    }

                    // since the initial float was max 100.0, we can multiple by 10 again
                    // this allows us to represent the division later with a more accurate int
                    exp *= 10;
                    // convert the percentage into a fraction of the 0xFFFF period
                    // 100*100**3 *10 / float(0xFFFF) ~= 15259
                    let on_period = (exp / 15259) as u16;
                    self.signals[signal_index].configure(0xFFFF, on_period);

                    ReturnCode::SUCCESS
                }
                CMD::NUM_SIGNALS => ReturnCode::SuccessWithValue {
                    value: self.signals.len(),
                },
            }
        } else {
            ReturnCode::ENOSUPPORT
        }
    }
}

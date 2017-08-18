//! Capsule that reads a ninedof sensor and outputs its results via
//! debug! messages.
//!
//! Usage
//! -----
//!
//! You need a device that provides the `hil::ninedof::NineDof` trait
//! as well as a virtualized alarm.
//!
//! ``rust
//! let rc_virtual_alarm = static_init!(
//!     VirtualMuxAlarm<'static, sam4l::ast::Ast>,
//!     VirtualMuxAlarm::new(mux_alarm));
//! let rustconf = static_init!(
//!     capsules::rustconf::RustConf<'static>,
//!     capsules::rustconf::RustConf::new(fxos8700, rc_virtual_alarm);
//! ```

use core::cell::Cell;
use kernel::ReturnCode;
use kernel::hil;
use kernel::hil::ninedof;
use kernel::hil::time;
use kernel::process::Error;

pub struct RustConf<'a, A: time::Alarm + 'a> {
    driver: &'a hil::ninedof::NineDof,
    alarm: &'a A,
    interval: Cell<u32>,
}

impl<'a, A: time::Alarm + 'a> RustConf<'a, A> {
    pub fn new(driver: &'a hil::ninedof::NineDof,
               alarm: &'a A) -> RustConf<'a, A> {
        RustConf {
            driver: driver,
            alarm: alarm,
            interval: Cell::new(1000),
        }
    }

    pub fn start(&self, interval: u32) -> ReturnCode {
        debug!("Starting RustConf capsule with interval {} at time {}", interval, self.alarm.now());
        self.interval.set(interval);
        let when = self.alarm.now().wrapping_add(interval);
        self.alarm.set_alarm(when);
        debug!("Set to fire at {}", when);
        ReturnCode::SUCCESS
    }
}

impl<'a, A: time::Alarm + 'a> time::Client for RustConf<'a, A> {
    fn fired(&self) {
        debug!("RustConf fired!");
        let when = self.alarm.now().wrapping_add(self.interval.get());
        self.alarm.set_alarm(when);
        self.driver.read_accelerometer();
    }
}


impl<'a, A: time::Alarm + 'a> hil::ninedof::NineDofClient for RustConf<'a, A> {
    fn callback(&self, arg1: usize, arg2: usize, arg3: usize) {
        debug!("Accelerometer reading: {} {} {}", arg1, arg2, arg3);
    }
}

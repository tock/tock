//! SysTick Simulation

use core::cell::Cell;
use core::ptr::read_volatile;
use nix::libc;
use nix::sys::signal::{self, SaFlags, SigAction, SigHandler, SigSet, Signal};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::nvic::{set_interrupt, SYSTICK};

use crate::syscall::{switch_context_kernel, PosixContext, CURRENT_PROCESS, RUNNING_KERNEL};

extern "C" {
    // used for debug
    // fn alarm(seconds: libc::c_uint) -> libc::c_int;
    fn ualarm(useconds: libc::c_uint, interval: libc::c_uint) -> libc::c_int;
}

extern "C" fn handle_sigalrm(
    _signal: libc::c_int,
    _siginfo: *mut libc::siginfo_t,
    unused: *mut libc::c_void,
) {
    unsafe {
        if !read_volatile(&RUNNING_KERNEL) {
            let mut context = &mut *(unused as *mut usize as *mut PosixContext);
            switch_context_kernel(&mut CURRENT_PROCESS, &mut context);
        }
    }
    unsafe {
        set_interrupt(SYSTICK);
    }
}

pub struct SysTick {
    us: Cell<u32>,
    expires: Cell<u128>,
}

impl SysTick {
    /// Initialize the `SysTick` with default values
    ///
    /// Use this constructor if the core implementation has a pre-calibration
    /// value in hardware.
    pub unsafe fn new() -> SysTick {
        let handler = SigHandler::SigAction(handle_sigalrm);
        let sigaction = SigAction::new(handler, SaFlags::empty(), SigSet::all());
        signal::sigaction(Signal::SIGALRM, &sigaction).unwrap();

        SysTick {
            us: Cell::new(0),
            expires: Cell::new(0),
        }
    }

    fn get_system_micros() -> u128 {
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        since_the_epoch.as_micros()
    }
}

impl kernel::SchedulerTimer for SysTick {
    fn start(&self, us: u32) {
        self.us.set(us);
        self.expires.set(0);
    }

    // fn has_expired(&self) -> bool {
    //     self.expires.get() != 0 && self.expires.get() < Self::get_system_micros()
    // }

    fn reset(&self) {
        self.us.set(0);
        self.expires.set(0);
    }

    fn arm(&self) {
        self.expires
            .set(Self::get_system_micros() + self.us.get() as u128);
        unsafe {
            // used for debug
            // alarm(5);
            ualarm(self.us.get(), 0);
        }
    }

    fn disarm(&self) {
        unsafe {
            ualarm(0, 0);
        }
    }

    fn get_remaining_us(&self) -> Option<u32> {
        let micros = Self::get_system_micros();
        if self.expires.get() == 0 {
            Some(self.us.get())
        } else if self.expires.get() > micros {
            Some((self.expires.get() - micros) as u32)
        } else {
            None
        }
    }
}

//! Scheduler Timer for enforcing Process Timeslices
//!
//! Interface for use by the Kernel to configure timers which can preempt userspace
//! processes.

use crate::hil::time::{self, Frequency};

/// Interface for the system tick timer.
///
/// A system tick timer provides a countdown timer to enforce process scheduling
/// quantums.  Implementations should have consistent timing while the CPU is
/// active, but need not operate during sleep.
///
/// On most chips, this will be implemented by the core (e.g. the ARM core systick), but
/// some chips lack this optional peripheral, in which case it might be
/// implemented by another timer or alarm controller, or require virtualization
/// on top of a single hardware timer.
pub trait SchedulerTimer {
    /// Sets the timer as close as possible to the given interval in
    /// microseconds, and starts counting down. Interrupts are not enabled.
    ///
    /// Callers can assume at least a 24-bit wide clock. Specific timing is
    /// dependent on the driving clock. For ARM boards with a dedicated
    /// SysTick peripheral, increments of 10ms are most
    /// accurate thanks to additional hardware support for this value, but
    /// values up to 400ms are valid.
    fn start_timer(&self, us: u32);

    /// Returns if there is at least `us` microseconds left
    fn at_least_us_remaining(&self, us: u32) -> bool;

    /// Returns true if the timer has expired since the last time this or set_timer()
    /// was called. If called a second time without an intermittent call to set_timer(),
    /// the return value is unspecified (implementations can return whatever they like)
    fn expired(&self) -> bool;

    /// Resets the timer
    ///
    /// Resets the timer to 0 and disables it
    fn reset(&self);

    /// Disarm the underlying timer. This does not stop the timer,
    /// but may disable the underlying interrupt if one has been set,
    /// preventing overhead from the timer firing after an executing application
    /// has returned to the kernel.
    fn disarm(&self);

    /// Arm the underlying timer. This does not start the timer,
    /// it just guarantees that an interrupt will be generated
    /// if an already started timer expires, which is useful for
    /// preempting userspace applications.
    fn arm(&self);

    /// Return the number of microseconds remaining before the alarm expires.
    fn get_remaining_us(&self) -> u32;
}

/// A dummy `SchedulerTimer` implementation in which the timer never expires.
///
/// Using this implementation is functional, but will mean the scheduler cannot
/// interrupt non-yielding processes.
impl SchedulerTimer for () {
    fn reset(&self) {}

    fn start_timer(&self, _: u32) {}

    fn disarm(&self) {}

    fn arm(&self) {}

    fn expired(&self) -> bool {
        false
    }

    fn at_least_us_remaining(&self, _: u32) -> bool {
        true
    }

    fn get_remaining_us(&self) -> u32 {
        10000 // chose arbitrary large value
    }
}

/// Implementation of SchedulerTimer trait on top of an arbitrary Alarm.
/// This mostly handles conversions from wall time, the required inputs
/// to the trait, to ticks, which are used to track time for alarms.
pub struct VirtualSchedulerTimer<A: 'static + time::Alarm<'static>> {
    alarm: &'static A,
}

impl<A: 'static + time::Alarm<'static>> VirtualSchedulerTimer<A> {
    pub fn new(alarm: &'static A) -> Self {
        Self { alarm }
    }
}

impl<A: 'static + time::Alarm<'static>> SchedulerTimer for VirtualSchedulerTimer<A> {
    fn reset(&self) {
        self.alarm.disable();
    }

    fn start_timer(&self, us: u32) {
        let tics = {
            // We need to convert from microseconds to native tics, which could overflow in 32-bit
            // arithmetic. So we convert to 64-bit. 64-bit division is an expensive subroutine, but
            // if `us` is a power of 10 the compiler will simplify it with the 1_000_000 divisor
            // instead.
            let us = us as u64;
            let hertz = A::Frequency::frequency() as u64;

            (hertz * us / 1_000_000) as u32
        };
        let fire_at = self.alarm.now().wrapping_add(tics);
        self.alarm.set_alarm(fire_at);
        self.alarm.disable(); //interrupts are off, but value is saved by MuxAlarm
    }

    fn arm(&self) {
        self.alarm.enable();
    }

    fn disarm(&self) {
        self.alarm.disable();
    }

    fn expired(&self) -> bool {
        // If next alarm is more than one second away from now, alarm must have expired.
        // Use this formulation to protect against errors when systick wraps around.
        // 1 second was chosen because it is significantly greater than the 400ms max value allowed
        // by set_timer, and requires no computational overhead (e.g. using 500ms would require
        // dividing the returned ticks by 2)
        !(self.alarm.get_alarm().wrapping_sub(self.alarm.now()) < A::Frequency::frequency())
    }

    fn at_least_us_remaining(&self, us: u32) -> bool {
        let tics = {
            // We need to convert from microseconds to native tics, which could overflow in 32-bit
            // arithmetic. So we convert to 64-bit. 64-bit division is an expensive subroutine, but
            // if `us` is a power of 10 the compiler will simplify it with the 1_000_000 divisor
            // instead.
            let us = us as u64;
            let hertz = A::Frequency::frequency() as u64;

            (hertz * us / 1_000_000) as u32
        };
        self.alarm.now() + tics < self.alarm.get_alarm()
    }

    fn get_remaining_us(&self) -> u32 {
        // Should this return 0 if the result is very large to handle it being called
        // after the alarm fires?
        self.alarm.get_alarm().wrapping_sub(self.alarm.now())
    }
}

impl<A: 'static + time::Alarm<'static>> time::AlarmClient for VirtualSchedulerTimer<A> {
    fn fired(&self) {
        // No need to handle the interrupt! The entire purpose
        // of the interrupt is to cause a transition to userspace, which
        // already happens for any mtimer interrupt, and the overflow check is sufficient
        // to determine that it was an mtimer interrupt.
        // However, because of how the MuxAlarm code is written, if the passed alarm
        // is a VirtualMuxAlarm, we must register as a client
        // of the MuxAlarm in order to guarantee that requested interrupts are not dropped.
    }
}

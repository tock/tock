//! Scheduler Timer for enforcing Process Timeslices
//!
//! Interface for use by the Kernel to configure timers which can preempt
//! userspace processes.

use crate::hil::time::{self, Frequency};

/// Interface for the system scheduler timer.
///
/// A system scheduler timer provides a countdown timer to enforce process
/// scheduling time quanta. Implementations should have consistent timing
/// while the CPU is active, but need not operate during sleep.
///
/// The primary requirement an implementation of this interface must satisfy is
/// it must be capable of generating an interrupt when the timer expires. This
/// interrupt will interrupt the executing process, returning control to the
/// kernel, and allowing the scheduler to make decisions about what to run next.
///
/// On most chips, this interface will be implemented by a core peripheral (e.g.
/// the ARM core systick peripheral). However, some chips lack this optional
/// peripheral, in which case it might be implemented by another timer or alarm
/// peripheral, or require virtualization on top of a shared hardware timer.
///
/// The `SchedulerTimer` interface is carefully designed to be rather general to
/// support the various implementations required on different hardware
/// platforms. The general operation is the kernel will start a timer, in effect
/// starting the time quantum assigned to a process. While the process is
/// running, it will arm the timer, indicating to the implementation to ensure
/// that an interrupt will occur when the time quantum is exhausted. When the
/// process has stopped running the timer will be disarmed, indicating that an
/// interrupt is no longer required. Note, many scheduler implementations also
/// charge time spent running the kernel against the process time quantum.
/// When the kernel needs to know if a process has exhausted its time quantum
/// it will call `expired()`.
///
/// Implementations must take care when using interrupts. Since the
/// `SchedulerTimer` is used in the core kernel loop and scheduler, top half
/// interrupt handlers may not have executed before `SchedulerTimer` functions
/// are called. In particular, implementations on top of virtualized timers may
/// receive the interrupt fired callback "late" (i.e. after the kernel calls
/// `expired()`). Implementations should ensure that they can reliably check for
/// timeslice expirations.
pub trait SchedulerTimer {
    /// Start a timer for a process timeslice.
    ///
    /// This must set a timer for an interval as close as possible to the given
    /// interval in microseconds. Interrupts do need to be enabled.
    ///
    /// Callers can assume at least a 24-bit wide clock. Specific timing is
    /// dependent on the driving clock. For ARM boards with a dedicated SysTick
    /// peripheral, increments of 10ms are most accurate thanks to additional
    /// hardware support for this value. ARM SysTick supports intervals up to
    /// 400ms.
    fn start_timer(&self, us: u32);

    /// Reset the scheduler timer.
    ///
    /// This must reset the timer, and can safely disable it and put it in a low
    /// power state. Calling any function other than `start_timer()` immediately
    /// after `reset()` is invalid.
    ///
    /// Implementations should disable the timer and put it in a lower power
    /// state, but this will depend on the hardware and whether virualization
    /// layers are used.
    fn reset(&self);

    /// Arm the SchedulerTimer timer and ensure an interrupt will be generated.
    ///
    /// The timer must already be started by calling `start_timer()`. This
    /// function guarantees that an interrupt will be generated when the already
    /// started timer expires. This interrupt will preempt the running userspace
    /// process.
    fn arm(&self);

    /// Disarm the SchedulerTimer timer indicating an interrupt is no longer
    /// required.
    ///
    /// This does not stop the timer, but indicates to the SchedulerTimer that
    /// an interrupt is no longer required (i.e. the process is no longer
    /// executing). By not requiring an interrupt this may allow certain
    /// implementations to be more efficient by removing the overhead of
    /// handling the interrupt. The implementation may disable the underlying
    /// interrupt if one has been set, depending on the requirements of the
    /// implementation.
    fn disarm(&self);

    /// Check if there are at least `us` microseconds remaining in the process's
    /// timeslice.
    fn at_least_us_remaining(&self, us: u32) -> bool;

    /// Return the number of microseconds remaining in the process's timeslice.
    fn get_remaining_us(&self) -> u32;

    /// Check if the process timeslice has expired.
    ///
    /// Returns `true` if the timer has expired since the last time this
    /// function or `set_timer()` has been called. This function may not be
    /// called after it has returned `true` for a given timeslice until
    /// `set_timer()` is called again (to start a new timeslice). If `expired()`
    /// is called again after returning `true` without an intervening call to
    /// `set_timer()`, the return the return value is unspecified and
    /// implementations may return whatever they like.
    ///
    /// The requirement that this may not be called again after it returns
    /// `true` simplifies implementation on hardware platforms where the
    /// hardware automatically clears the expired flag on a read, as with the
    /// ARM SysTick peripheral.
    fn expired(&self) -> bool;
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
        // No need to handle the interrupt! The entire purpose of the interrupt
        // is to cause a transition to userspace, which already happens for any
        // mtimer interrupt, and the overflow check is sufficient to determine
        // that it was an mtimer interrupt.
        //
        // However, because of how the MuxAlarm code is written, if the passed
        // alarm is a VirtualMuxAlarm, we must register as a client of the
        // MuxAlarm in order to guarantee that requested interrupts are not
        // dropped.
    }
}

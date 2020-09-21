//! Scheduler Timer for enforcing Process Timeslices
//!
//! Interface for use by the Kernel to configure timers which can preempt
//! userspace processes.

use crate::hil::time::{self, Frequency, Ticks};

/// Interface for the system scheduler timer.
///
/// A system scheduler timer provides a countdown timer to enforce process
/// scheduling time quanta. Implementations should have consistent timing while
/// the CPU is active, but need not operate during sleep. Note, many scheduler
/// implementations also charge time spent running the kernel on behalf of the
/// process against the process time quantum.
///
/// The primary requirement an implementation of this interface must satisfy is
/// it must be capable of generating an interrupt when the timer expires. This
/// interrupt will interrupt the executing process, returning control to the
/// kernel, and allowing the scheduler to make decisions about what to run next.
///
/// On most chips, this interface will be implemented by a core peripheral (e.g.
/// the ARM core SysTick peripheral). However, some chips lack this optional
/// peripheral, in which case it might be implemented by another timer or alarm
/// peripheral, or require virtualization on top of a shared hardware timer.
///
/// The `SchedulerTimer` interface is carefully designed to be rather general to
/// support the various implementations required on different hardware
/// platforms. The general operation is the kernel will start a timer, which
/// starts the time quantum assigned to a process. While the process is running,
/// the kernel will arm the timer, telling the implementation it must ensure
/// that an interrupt will occur when the time quantum is exhausted. When the
/// process has stopped running, the kernel will disarm the timer, indicating to
/// the implementation that an interrupt is no longer required. To check if the
/// process has exhausted its time quantum the kernel will explicitly ask the
/// implementation. The kernel itself does not expect to get an interrupt to
/// handle when the time quantum is exhausted. This is because the time quantum
/// may end while the kernel itself is running, and the kernel does not need to
/// effectively preempt itself.
///
/// The `arm()` and `disarm()` functions in this interface serve as an optional
/// optimization opportunity. This pair allows an implementation to only enable
/// the interrupt when it is strictly necessary, i.e. while the process is
/// actually executing. However, a correct implementation can have interrupts
/// enabled anytime the scheduler timer has been started. What the
/// implementation must ensure is that the interrupt is enabled when `arm()` is
/// called.
///
/// Implementations must take care when using interrupts. Since the
/// `SchedulerTimer` is used in the core kernel loop and scheduler, top half
/// interrupt handlers may not have executed before `SchedulerTimer` functions
/// are called. In particular, implementations on top of virtualized timers may
/// receive the interrupt fired callback "late" (i.e. after the kernel calls
/// `has_expired()`). Implementations should ensure that they can reliably check
/// for timeslice expirations.
pub trait SchedulerTimer {
    /// Start a timer for a process timeslice. The `us` argument is the length
    /// of the timeslice in microseconds.
    ///
    /// This must set a timer for an interval as close as possible to the given
    /// interval in microseconds. Interrupts do not need to be enabled. However,
    /// if the implementation cannot separate time keeping from interrupt
    /// generation, the implementation of `start()` should enable interrupts and
    /// leave them enabled anytime the timer is active.
    ///
    /// Callers can assume at least a 24-bit wide clock. Specific timing is
    /// dependent on the driving clock. For ARM boards with a dedicated SysTick
    /// peripheral, increments of 10ms are most accurate thanks to additional
    /// hardware support for this value. ARM SysTick supports intervals up to
    /// 400ms.
    fn start(&self, us: u32);

    /// Reset the SchedulerTimer.
    ///
    /// This must reset the timer, and can safely disable it and put it in a low
    /// power state. Calling any function other than `start()` immediately after
    /// `reset()` is invalid.
    ///
    /// Implementations _should_ disable the timer and put it in a lower power
    /// state. However, not all implementations will be able to guarantee this
    /// (for example depending on the underlying hardware or if the timer is
    /// implemented on top of a virtualized timer).
    fn reset(&self);

    /// Arm the SchedulerTimer timer and ensure an interrupt will be generated.
    ///
    /// The timer must already be started by calling `start()`. This function
    /// guarantees that an interrupt will be generated when the already started
    /// timer expires. This interrupt will preempt the running userspace
    /// process.
    ///
    /// If the interrupt is already enabled when `arm()` is called, this
    /// function should be a no-op implementation.
    fn arm(&self);

    /// Disarm the SchedulerTimer timer indicating an interrupt is no longer
    /// required.
    ///
    /// This does not stop the timer, but indicates to the SchedulerTimer that
    /// an interrupt is no longer required (i.e. the process is no longer
    /// executing). By not requiring an interrupt this may allow certain
    /// implementations to be more efficient by removing the overhead of
    /// handling the interrupt.
    ///
    /// If the implementation cannot disable the interrupt without stopping the
    /// time keeping mechanism, this function should be a no-op implementation.
    fn disarm(&self);

    /// Return the number of microseconds remaining in the process's timeslice
    /// if the timeslice is still active.
    ///
    /// If the timeslice is still active, this returns `Some()` with the number
    /// of microseconds remaining in the timeslice. If the timeslice has
    /// expired, this returns `None`.
    ///
    /// This function may not be called after it has returned `None` (signifying
    /// the timeslice has expired) for a given timeslice until `start()` is
    /// called again (to start a new timeslice). If `get_remaining_us()` is
    /// called again after returning `None` without an intervening call to
    /// `start()`, the return value is unspecified and implementations may
    /// return whatever they like.
    fn get_remaining_us(&self) -> Option<u32>;
}

/// A dummy `SchedulerTimer` implementation in which the timer never expires.
///
/// Using this implementation is functional, but will mean the scheduler cannot
/// interrupt non-yielding processes.
impl SchedulerTimer for () {
    fn reset(&self) {}

    fn start(&self, _: u32) {}

    fn disarm(&self) {}

    fn arm(&self) {}

    fn get_remaining_us(&self) -> Option<u32> {
        Some(10000) // chose arbitrary large value
    }
}

/// Implementation of SchedulerTimer trait on top of a virtual alarm.
///
/// Currently, this implementation depends slightly on the virtual alarm
/// implementation in capsules -- namely it assumes that get_alarm will still
/// return the passed value even after the timer is disarmed. Thus this should
/// only be implemented with a virtual alarm. If a dedicated hardware timer is
/// available, it is more performant to implement the scheduler timer directly
/// for that hardware peripheral without the alarm abstraction in between.
///
/// This mostly handles conversions from wall time, the required inputs to the
/// trait, to ticks, which are used to track time for alarms.
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
        self.alarm.disarm();
    }

    fn start(&self, us: u32) {
        let tics = {
            // We need to convert from microseconds to native tics, which could overflow in 32-bit
            // arithmetic. So we convert to 64-bit. 64-bit division is an expensive subroutine, but
            // if `us` is a power of 10 the compiler will simplify it with the 1_000_000 divisor
            // instead.
            let us = us as u64;
            let hertz = A::Frequency::frequency() as u64;

            (hertz * us / 1_000_000) as u32
        };
        let now = self.alarm.now();
        let fire_at = now.wrapping_add(A::Ticks::from(tics));
        self.alarm.set_alarm(now, fire_at);
    }

    fn arm(&self) {
        //self.alarm.arm();
    }

    fn disarm(&self) {
        //self.alarm.disarm();
    }

    fn get_remaining_us(&self) -> Option<u32> {
        // We need to convert from native tics to us, multiplication could overflow in 32-bit
        // arithmetic. So we convert to 64-bit.

        let diff = self.alarm.get_alarm().wrapping_sub(self.alarm.now()).into_u32() as u64;
        
        // If next alarm is more than one second away from now, alarm must have expired.
        // Use this formulation to protect against errors when now has passed alarm.
        // 1 second was chosen because it is significantly greater than the 400ms max value allowed
        // by start(), and requires no computational overhead (e.g. using 500ms would require
        // dividing the returned ticks by 2)
        // However, if the alarm frequency is slow enough relative to the cpu frequency, it is
        // possible this will be evaluated while now() == get_alarm(), so we special case that
        // result where the alarm has fired but the subtraction has not overflowed
        if diff >= A::Frequency::frequency() as u64 || diff == 0 {
            None
        } else {
            let hertz = A::Frequency::frequency() as u64;
            Some(((diff * 1_000_000) / hertz) as u32)
        }
    }
}

impl<A: 'static + time::Alarm<'static>> time::AlarmClient for VirtualSchedulerTimer<A> {
    fn alarm(&self) {
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

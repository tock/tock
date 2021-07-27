//! LiteX timer core
//!
//! Hardware source and documentation available at
//! [`litex/soc/cores/timer.py`](https://github.com/enjoy-digital/litex/blob/master/litex/soc/cores/timer.py).

use core::cell::Cell;
use core::marker::PhantomData;
use kernel::hil::time::{
    Alarm, AlarmClient, Frequency, Ticks, Ticks32, Ticks64, Time, Timer, TimerClient,
};
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::StaticRef;
use kernel::ErrorCode;

use crate::event_manager::LiteXEventManager;
use crate::litex_registers::{
    register_bitfields, LiteXSoCRegisterConfiguration, Read, ReadRegWrapper, Write, WriteRegWrapper,
};

const EVENT_MANAGER_INDEX: usize = 0;

type LiteXTimerEV<'a, R> = LiteXEventManager<
    'a,
    u8,
    <R as LiteXSoCRegisterConfiguration>::ReadOnly8,
    <R as LiteXSoCRegisterConfiguration>::ReadWrite8,
    <R as LiteXSoCRegisterConfiguration>::ReadWrite8,
>;

/// [`LiteXTimer`] register layout
#[repr(C)]
pub struct LiteXTimerRegisters<R: LiteXSoCRegisterConfiguration> {
    /// Load value when Timer is (re-)enabled. In One-Shot mode, the
    /// value written to this register specifies the Timer's duration
    /// in clock cycles.
    load: R::ReadWrite32,
    /// Reload value when Timer reaches `0`. In Periodic mode, the
    /// value written to this register specify the Timer's period in
    /// clock cycles.
    reload: R::ReadWrite32,
    /// Enable flag of the Timer. Set this flag to `1` to enable/start
    /// the Timer. Set to `0` to disable the Timer.
    en: R::ReadWrite8,
    /// Update trigger for the current countdown value. A write to
    /// this register latches the current countdown value to `value`
    /// register.
    update_value: R::ReadWrite8,
    /// Latched countdown value. This value is updated by writing to
    /// `update_value`.
    value: R::ReadWrite32,
    /// LiteX EventManager status register
    ev_status: R::ReadOnly8,
    /// LiteX EventManager pending register
    ev_pending: R::ReadWrite8,
    /// LiteX EventManager pending register
    ev_enable: R::ReadWrite8,
    /// Write a `1` to latch current Uptime cycles to `uptime_cycles`
    /// register.
    ///
    /// # Optional register
    ///
    /// This register is only present if the SoC was configured with
    /// `timer_update = True`. Therefore, it's only indirectly
    /// accessed by the [`LiteXTimerUptime`](LiteXTimerUptime) struct,
    /// which a board will need to construct separately.
    uptime_latch: R::ReadWrite8,
    /// Latched uptime since power-up (in `sys_clk` cycles)
    ///
    /// # Optional register
    ///
    /// This register is only present if the SoC was configured with
    /// `timer_update = True`. Therefore, it's only indirectly
    /// accessed by the [`LiteXTimerUptime`](LiteXTimerUptime) struct,
    /// which a board will need to construct separately.
    uptime: R::ReadOnly64,
}

impl<R: LiteXSoCRegisterConfiguration> LiteXTimerRegisters<R> {
    fn ev<'a>(&'a self) -> LiteXTimerEV<'a, R> {
        LiteXTimerEV::<R>::new(&self.ev_status, &self.ev_pending, &self.ev_enable)
    }
}

register_bitfields![u8,
    en [
        enable OFFSET(0) NUMBITS(1) []
    ],
    update_value [
        latch_value OFFSET(0) NUMBITS(1) []
    ],
    uptime_latch [
        latch_value OFFSET(0) NUMBITS(1) []
    ]
];

/// LiteX hardware timer core uptime extension
///
/// Defined in
/// [`litex/soc/cores/timer.py`](https://github.com/enjoy-digital/litex/blob/master/litex/soc/cores/timer.py).
///
/// This hardware peripheral can optionally feature an uptime
/// register, which counts the current ticks since power on. This
/// wrapper can be used to provide users a safe way to access this
/// register through the [`Time::now`] interface, if it is available
/// in hardware.
pub struct LiteXTimerUptime<'t, R: LiteXSoCRegisterConfiguration, F: Frequency> {
    timer: &'t LiteXTimer<'t, R, F>,
}

impl<'t, R: LiteXSoCRegisterConfiguration, F: Frequency> LiteXTimerUptime<'t, R, F> {
    /// Contruct a new [`LiteXTimerUptime`] wrapper
    ///
    /// The function is marked `unsafe` as it will provide a safe
    /// method to access the `uptime`-register on the underlying
    /// [`LiteXTimer`]. If this register is not present (i.e. the
    /// uptime feature is disabled), this will result in undefined
    /// behavior.
    pub const unsafe fn new(timer: &'t LiteXTimer<'t, R, F>) -> LiteXTimerUptime<'t, R, F> {
        LiteXTimerUptime { timer }
    }
}

impl<R: LiteXSoCRegisterConfiguration, F: Frequency> Time for LiteXTimerUptime<'_, R, F> {
    type Frequency = F;
    type Ticks = Ticks64;

    /// Return the current ticks since sytem power on
    fn now(&self) -> Self::Ticks {
        unsafe { self.timer.uptime() }
    }
}

/// LiteX hardware timer core
///
/// Defined in
/// [`litex/soc/cores/timer.py`](https://github.com/enjoy-digital/litex/blob/master/litex/soc/cores/timer.py).
///
/// This peripheral supports counting down a certain interval, either
/// as a oneshot timer or in a repeated fashion.
///
/// # Uptime extension
///
/// LiteX timers can _optionally_ be extended to feature an uptime
/// register integrated into the timer peripheral, monotonically
/// counting the clock ticks and wrapping at the maximum value.
///
/// The uptime register may have a different width as the [`Timer`]
/// peripheral itself, hence it must be implemented using a separate
/// type. The type must contain a reference to this [`Timer`]
/// instance, since the register is located on this register bank.
///
/// Since this extension is not always configured, the Timer features
/// an _unsafe_ function to read the uptime. It must only be called by
/// [`LiteXTimerUptime`] struct and only if the uptime has been
/// configured.
pub struct LiteXTimer<'a, R: LiteXSoCRegisterConfiguration, F: Frequency> {
    registers: StaticRef<LiteXTimerRegisters<R>>,
    client: OptionalCell<&'a dyn TimerClient>,
    /// Variable to store whether an interval has been set at least
    /// once (e.g. the timer has been started once)
    interval_set: Cell<bool>,
    _frequency: PhantomData<F>,
}

impl<R: LiteXSoCRegisterConfiguration, F: Frequency> LiteXTimer<'_, R, F> {
    pub fn new(base: StaticRef<LiteXTimerRegisters<R>>) -> Self {
        LiteXTimer {
            registers: base,
            client: OptionalCell::empty(),
            interval_set: Cell::new(false),
            _frequency: PhantomData,
        }
    }

    /// Get the uptime register value.
    ///
    /// This function is marked as unsafe to avoid clients calling it,
    /// if the underlying LiteX hardware timer does not feature the
    /// uptime registers.
    ///
    /// Clients should use the [`LiteXTimerUptime`] wrapper instead,
    /// which exposes this value as part of their
    /// [`Time::now`](Time::now) implementation.
    unsafe fn uptime(&self) -> Ticks64 {
        WriteRegWrapper::wrap(&self.registers.uptime_latch).write(uptime_latch::latch_value::SET);
        self.registers.uptime.get().into()
    }

    pub fn service_interrupt(&self) {
        // Check whether the event is still asserted.
        //
        // It could be that an interrupt was fired and the timer was
        // reset / disabled in the mean time.
        if self.registers.ev().event_asserted(EVENT_MANAGER_INDEX) {
            if self.registers.reload.get() == 0 {
                // Timer is a oneshot

                // If the timer really is a oneshot, the remaining time must be 0
                WriteRegWrapper::wrap(&self.registers.update_value)
                    .write(update_value::latch_value::SET);
                assert!(self.registers.value.get() == 0);

                // Completely disable and make sure it doesn't generate
                // more interrupts until it is started again
                let _ = self.cancel();
            } else {
                // Timer is repeating
                //
                // Simply only acknowledge the current interrupt
                self.registers.ev().clear_event(EVENT_MANAGER_INDEX);
            }

            // In any case, perform a callback to the client
            self.client.map(|client| {
                client.timer();
            });
        }
    }

    fn start_timer(&self, tics: u32, repeat_ticks: Option<u32>) {
        // If the timer is already enabled, cancel it and disable all
        // interrupts
        if self.is_enabled() {
            let _ = self.cancel();
        }

        if let Some(reload) = repeat_ticks {
            // Reload the timer with `reload_ticks` after it has
            // expired (reloading mode)
            self.registers.reload.set(reload);
        } else {
            // Prevent reloading of the timer (oneshot mode)
            self.registers.reload.set(0);
        }

        // Load the countdown in ticks (this register won't get
        // overwritten, hence it is a safe reference to the original
        // tics value which we don't need to save locally)
        self.registers.load.set(tics);

        // Since the timer is disabled, the load register is
        // immediately mirrored to the internal register, which will
        // cause the event source to be off.
        //
        // It is important for the internal value to be non-zero prior
        // to enabling the event, as it could otherwise immediately
        // generate an event again (which might be desired for a timer
        // with tics == 0).
        //
        // Clear any pending event of a previous timer, then enable
        // the event.
        self.registers.ev().clear_event(EVENT_MANAGER_INDEX);
        self.registers.ev().enable_event(EVENT_MANAGER_INDEX);

        // The timer has been started at least once by now
        self.interval_set.set(true);

        // Start the timer
        WriteRegWrapper::wrap(&self.registers.en).write(en::enable::SET);
    }
}

impl<R: LiteXSoCRegisterConfiguration, F: Frequency> Time for LiteXTimer<'_, R, F> {
    type Frequency = F;
    type Ticks = Ticks32;

    fn now(&self) -> Self::Ticks {
        WriteRegWrapper::wrap(&self.registers.update_value).write(update_value::latch_value::SET);
        self.registers.value.get().into()
    }
}

impl<'a, R: LiteXSoCRegisterConfiguration, F: Frequency> Timer<'a> for LiteXTimer<'a, R, F> {
    fn set_timer_client(&self, client: &'a dyn TimerClient) {
        self.client.set(client);
    }

    fn oneshot(&self, interval: Self::Ticks) -> Self::Ticks {
        self.start_timer(interval.into_u32(), None);
        interval
    }

    fn repeating(&self, interval: Self::Ticks) -> Self::Ticks {
        self.start_timer(interval.into_u32(), Some(interval.into_u32()));
        interval
    }

    fn interval(&self) -> Option<Self::Ticks> {
        if self.interval_set.get() {
            // The timer has been started at least once, so we can
            // trust that the `load` register will have the interval
            // of the last requested timer.
            Some(self.registers.load.get().into())
        } else {
            // The timer was never started and so does not have an
            // interval set.
            None
        }
    }

    fn is_repeating(&self) -> bool {
        // The timer is repeating timer if it has been started at
        // least once and the reload register is non-zero
        self.interval_set.get() && self.registers.reload.get() != 0
    }

    fn is_oneshot(&self) -> bool {
        // The timer is a oneshot if it has been started at least once
        // and the reload register is 0
        self.interval_set.get() && self.registers.reload.get() == 0
    }

    fn time_remaining(&self) -> Option<Self::Ticks> {
        if self.is_enabled() {
            WriteRegWrapper::wrap(&self.registers.update_value)
                .write(update_value::latch_value::SET);
            Some(self.registers.value.get().into())
        } else {
            None
        }
    }

    fn is_enabled(&self) -> bool {
        ReadRegWrapper::wrap(&self.registers.en).is_set(en::enable)
    }

    fn cancel(&self) -> Result<(), ErrorCode> {
        // Prevent the event source from generating new interrupts
        self.registers.ev().disable_event(EVENT_MANAGER_INDEX);

        // Stop the timer
        WriteRegWrapper::wrap(&self.registers.en).write(en::enable::CLEAR);

        // Clear any previous event
        self.registers.ev().clear_event(EVENT_MANAGER_INDEX);

        Ok(())
    }
}

/// LiteX alarm implementation, based on [`LiteXTimer`] and
/// [`LiteXTimerUptime`]
///
/// LiteX does not have an [`Alarm`] compatible hardware peripheral,
/// so an [`Alarm`] is emulated using a repeatedly set [`LiteXTimer`],
/// comparing the current time against the [`LiteXTimerUptime`] (which
/// is also exposed as [`Time::now`](Time::now).
pub struct LiteXAlarm<'t, 'c, R: LiteXSoCRegisterConfiguration, F: Frequency> {
    uptime: &'t LiteXTimerUptime<'t, R, F>,
    timer: &'t LiteXTimer<'t, R, F>,
    alarm_client: OptionalCell<&'c dyn AlarmClient>,
    reference_time: Cell<<LiteXTimerUptime<'t, R, F> as Time>::Ticks>,
    alarm_time: OptionalCell<<LiteXTimerUptime<'t, R, F> as Time>::Ticks>,
}

impl<'t, 'c, R: LiteXSoCRegisterConfiguration, F: Frequency> LiteXAlarm<'t, 'c, R, F> {
    pub fn new(
        uptime: &'t LiteXTimerUptime<'t, R, F>,
        timer: &'t LiteXTimer<'t, R, F>,
    ) -> LiteXAlarm<'t, 'c, R, F> {
        LiteXAlarm {
            uptime,
            timer,
            alarm_client: OptionalCell::empty(),
            reference_time: Cell::new((0 as u32).into()),
            alarm_time: OptionalCell::empty(),
        }
    }

    /// Initialize the [`LiteXAlarm`]
    ///
    /// This will register itself as a client for the underlying
    /// [`LiteXTimer`].
    pub fn initialize(&'t self) {
        self.timer.set_timer_client(self);
    }

    fn timer_tick(&self, is_callback: bool) {
        // Check whether we've already reached the alarm time,
        // otherwise set the timer to the difference or the max value
        // respectively

        // This function gets called when initially setting the alarm
        // and for every fired LiteXTimer oneshot operation. Since we
        // can't call a client-callback within the same call stack of
        // the initial `set_alarm` call, we need to wait on the timer
        // at least once (even if the alarm time has already passed).

        let reference = self.reference_time.get();
        let alarm_time = self.alarm_time.expect("alarm not set");

        if !self.now().within_range(reference, alarm_time) {
            // It's time, ring the alarm

            // Make sure we're in an callback, otherwise set the timer
            // to a very small value to trigger a Timer interrupt
            // immediately
            if is_callback {
                // Reset the alarm to 0
                self.alarm_time.clear();

                // Call the client
                self.alarm_client.map(|c| c.alarm());
            } else {
                // Trigger an interrupt one tick from now, which will
                // call this function again
                self.timer.oneshot((1 as u32).into());
            }
        } else {
            // It's not yet time to call the client, set the timer
            // again

            let remaining = alarm_time.wrapping_sub(self.now());
            if remaining < (u32::MAX as u64).into() {
                // The remaining time fits into a single timer
                // invocation
                self.timer.oneshot(remaining.into_u32().into());
            } else {
                // The remaining time is longer than a single timer
                // invocation can cover, set the timer to the maximum
                // value
                self.timer.oneshot(u32::MAX.into());
            }
        }
    }
}

impl<'t, 'c, R: LiteXSoCRegisterConfiguration, F: Frequency> Time for LiteXAlarm<'t, 'c, R, F> {
    type Frequency = F;
    type Ticks = <LiteXTimerUptime<'t, R, F> as Time>::Ticks;

    fn now(&self) -> Self::Ticks {
        self.uptime.now()
    }
}

impl<'t, 'c, R: LiteXSoCRegisterConfiguration, F: Frequency> Alarm<'c>
    for LiteXAlarm<'t, 'c, R, F>
{
    fn set_alarm_client(&self, client: &'c dyn AlarmClient) {
        self.alarm_client.set(client);
    }

    fn set_alarm(&self, reference: Self::Ticks, dt: Self::Ticks) {
        // Cancel any pending alarm
        if self.is_armed() {
            let _ = self.disarm();
        }

        // Store both the reference and alarm time (required for
        // reliable comparison with wrapping time)
        self.reference_time.set(reference);
        self.alarm_time.set(reference.wrapping_add(dt));

        // Set the underlying timer at least once (`is_callback =
        // false`) to trigger a callback to the client in a different
        // call stack
        self.timer_tick(false);
    }

    fn get_alarm(&self) -> Self::Ticks {
        // Undefined at boot, so 0
        self.alarm_time.unwrap_or((0 as u32).into())
    }

    fn is_armed(&self) -> bool {
        self.alarm_time.is_some()
    }

    fn disarm(&self) -> Result<(), ErrorCode> {
        let _ = self.timer.cancel();
        self.alarm_time.clear();

        Ok(())
    }

    fn minimum_dt(&self) -> Self::Ticks {
        (1 as u32).into()
    }
}

impl<'t, 'c, R: LiteXSoCRegisterConfiguration, F: Frequency> TimerClient
    for LiteXAlarm<'t, 'c, R, F>
{
    fn timer(&self) {
        self.timer_tick(true);
    }
}

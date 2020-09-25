//! LiteX timer core
//!
//! Documentation in `litex/soc/cores/timer.py`.

use core::marker::PhantomData;
use kernel::common::cells::OptionalCell;
use kernel::common::StaticRef;
use kernel::hil::time::{
    Alarm, AlarmClient, Frequency, Ticks, Ticks32, Ticks64, Time, Timer, TimerClient,
};
use kernel::ReturnCode;

use crate::event_manager::LiteXEventManager;
use crate::litex_registers::{
    register_bitfields, LiteXSoCRegisterConfiguration, Read, ReadRegWrapper, Write, WriteRegWrapper,
};

// TODO: Make timer generic over the underlying width

// TODO: Don't enable / disable all events, but just the specific one
//const EVENT_MANAGER_INDEX: usize = 0;

type LiteXTimerEV<'a, R> = LiteXEventManager<
    'a,
    u8,
    <R as LiteXSoCRegisterConfiguration>::ReadOnly8,
    <R as LiteXSoCRegisterConfiguration>::ReadWrite8,
    <R as LiteXSoCRegisterConfiguration>::ReadWrite8,
>;

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
    ///# Optional register
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

pub struct LiteXTimerUptime<'t, R: LiteXSoCRegisterConfiguration, F: Frequency> {
    timer: &'t LiteXTimer<'t, R, F>,
}

impl<'t, R: LiteXSoCRegisterConfiguration, F: Frequency> LiteXTimerUptime<'t, R, F> {
    pub const unsafe fn new(timer: &'t LiteXTimer<'t, R, F>) -> LiteXTimerUptime<'t, R, F> {
        LiteXTimerUptime { timer }
    }
}

impl<R: LiteXSoCRegisterConfiguration, F: Frequency> Time for LiteXTimerUptime<'_, R, F> {
    type Frequency = F;
    type Ticks = Ticks64;

    fn now(&self) -> Self::Ticks {
        unsafe { self.timer.uptime() }
    }
}

/// Hardware timer peripheral found on LiteX SoCs
///
/// Defined in
/// [`litex/soc/cores/timer.py`](https://github.com/enjoy-digital/litex/blob/master/litex/soc/cores/timer.py).
///
/// This peripheral supports counting down a certain interval, either
/// as a oneshot timer or in a repeated fashion.
///
/// # Uptime peripheral
///
/// LiteX timers can _optionally_ be extended to feature an uptime
/// register integrated into the timer peripheral, monotonically
/// counting the clock ticks and wrapping at the maximum value.
///
/// The uptime register may have a different width as the `Timer`
/// peripheral itself, hence it must be implemented using a separate
/// type. The type must contain a reference to this `Timer` instance,
/// since the register is located on this register bank.
///
/// Since this extension is not always configured, the Timer features
/// an _unsafe_ function to read the uptime. It must only be called by
/// `LiteXTimerUptime` struct and only if the uptime has been
/// configured.
pub struct LiteXTimer<'a, R: LiteXSoCRegisterConfiguration, F: Frequency> {
    registers: StaticRef<LiteXTimerRegisters<R>>,
    client: OptionalCell<&'a dyn TimerClient>,
    _frequency: PhantomData<F>,
}

impl<R: LiteXSoCRegisterConfiguration, F: Frequency> LiteXTimer<'_, R, F> {
    pub const fn new(base: StaticRef<LiteXTimerRegisters<R>>) -> Self {
        LiteXTimer {
            registers: base,
            client: OptionalCell::empty(),
            _frequency: PhantomData,
        }
    }

    unsafe fn uptime(&self) -> Ticks64 {
        WriteRegWrapper::wrap(&self.registers.uptime_latch).write(uptime_latch::latch_value::SET);
        self.registers.uptime.get().into()
    }

    pub fn service_interrupt(&self) {
        if self.registers.reload.get() == 0 {
            // Timer is a oneshot

            // If the timer really is a oneshot, the remaining time must be 0
            WriteRegWrapper::wrap(&self.registers.update_value)
                .write(update_value::latch_value::SET);
            assert!(self.registers.value.get() == 0);

            // Completely disable and make sure it doesn't generate
            // more interrupts until it is started again
            self.cancel();
        } else {
            // Timer is repeating only acknowledge the current
            // interrupt
            self.registers.ev().clear_all();
        }

        self.client.map(|client| {
            client.timer();
        });
    }

    fn start_timer(&self, tics: u32, repeat_ticks: Option<u32>) {
        if self.is_enabled() {
            self.cancel();
        }

        // Stop the timer if it is currently running to prevent
        // interrupts from happening while modifying the timer value
        // non-atomically.
        WriteRegWrapper::wrap(&self.registers.en).write(en::enable::CLEAR);

        if let Some(reload) = repeat_ticks {
            // Reload the timer with `reload_ticks` after it has
            // expired
            self.registers.reload.set(reload);
        } else {
            // Prevent reloading of the timer
            self.registers.reload.set(0);
        }

        // Load the countdown in ticks (won't get overwritten, hence a
        // safe reference to the original tics value)
        self.registers.load.set(tics);

        // Since the timer is disabled, the load register is
        // immediately mirrored to the internal register, which will
        // cause the event source to be off.
        //
        // It is important for the internal value to be non-zero prior
        // to enabling the event, as it could otherwise immediately
        // generate an event again.
        //
        // Clear any pending event of a previous timer, then enable
        // the event.
        self.registers.ev().clear_all();
        self.registers.ev().enable_all();

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
        // TODO: Check whether it was set at least once
        Some(self.registers.load.get().into())
    }

    fn is_repeating(&self) -> bool {
        self.registers.reload.get() != 0
    }

    fn is_oneshot(&self) -> bool {
        !self.is_repeating()
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
        // If the enable register is set, the timer will either fire
        // in every case:
        //
        // - either the countdown is not yet done
        //
        // - or the countdown is done, which would set event in the
        //   event manager, which will be cleared in the respective
        //   interrupt handler
        ReadRegWrapper::wrap(&self.registers.en).is_set(en::enable)
    }

    fn cancel(&self) -> ReturnCode {
        // Prevent the event source from generating new interrupts
        self.registers.ev().disable_all();

        // Stop the timer
        WriteRegWrapper::wrap(&self.registers.en).write(en::enable::CLEAR);

        // Clear any previous event
        self.registers.ev().clear_all();

        ReturnCode::SUCCESS
    }
}

// TODO: This is written under the assumption that the u64-uptime
// clock does not wrap around

// TODO: The timer must be only used once

pub struct LiteXAlarm<'t, 'c, R: LiteXSoCRegisterConfiguration, F: Frequency> {
    uptime: &'t LiteXTimerUptime<'t, R, F>,
    timer: &'t LiteXTimer<'t, R, F>,
    alarm_client: OptionalCell<&'c dyn AlarmClient>,
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
            alarm_time: OptionalCell::empty(),
        }
    }

    pub fn initialize(&'t self) {
        self.timer.set_timer_client(self);
    }

    pub fn timer_tick(&self, is_upcall: bool) {
        // Check if we've already reached the alarm time, otherwise
        // set the timer to the difference or the max value
        // respectively

        if self.now() >= self.alarm_time.expect("alarm not set") {
            // It's time, ring the alarm

            // Make sure we're in an upcall, otherwise set the timer
            // to a very small value to trigger a Timer interrupt
            // immediately
            if is_upcall {
                // Reset the alarm to 0
                self.alarm_time.clear();

                // Call the client
                self.alarm_client.map(|c| c.alarm());
            } else {
                // Trigger an interrupt almost immediately, which will
                // call this function again
                self.timer.oneshot((1 as u32).into());
            }
        } else {
            // It's not yet time to call the client, set the timer
            // again

            let remaining = self
                .alarm_time
                .expect("alarm not set")
                .wrapping_sub(self.now());
            if remaining < (u32::MAX as u64).into() {
                self.timer.oneshot(remaining.into_u32().into());
            } else {
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
            self.disarm();
        }

        // We assume that the 64-bit counter will never wrap
        //
        // TODO: Write alarm that works with a wrapping time
        assert!(self.uptime.now() >= reference);
        assert!(reference.wrapping_add(dt) > reference);

        self.alarm_time.set(reference.wrapping_add(dt));

        self.timer_tick(false);
    }

    fn get_alarm(&self) -> Self::Ticks {
        // Undefined at boot, so 0
        self.alarm_time.unwrap_or((0 as u32).into())
    }

    fn is_armed(&self) -> bool {
        self.alarm_time.is_some()
    }

    fn disarm(&self) -> ReturnCode {
        self.timer.cancel();
        self.alarm_time.clear();

        ReturnCode::SUCCESS
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

use cortexm0p;
use cortexm0p::support::atomic;
use kernel::hil;
use kernel::hil::time::{Alarm, Ticks, Ticks32, Time};
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{
    register_bitfields, register_structs, ReadOnly, ReadWrite, WriteOnly,
};
use kernel::utilities::StaticRef;
use kernel::ErrorCode;

use crate::interrupts::TIMER_IRQ_0;

register_structs! {
    /// Controls time and alarms\n
    /// time is a 64 bit value indicating the time in usec since power-on\n
    /// timeh is the top 32 bits of time & timel is the bottom 32 bits\n
    /// to change time write to timelw before timehw\n
    /// to read time read from timelr before timehr\n
    /// An alarm is set by setting alarm_enable and writing to the corresponding
    /// When an alarm is pending, the corresponding alarm_running signal will be
    /// An alarm can be cancelled before it has finished by clearing the alarm_e
    /// When an alarm fires, the corresponding alarm_irq is set and alarm_runnin
    /// To clear the interrupt write a 1 to the corresponding alarm_irq
    TimerRegisters {
        /// Write to bits 63:32 of time\n
        /// always write timelw before timehw
        (0x000 => timehw: WriteOnly<u32, TIMEHW::Register>),
        /// Write to bits 31:0 of time\n
        /// writes do not get copied to time until timehw is written
        (0x004 => timelw: WriteOnly<u32, TIMELW::Register>),
        /// Read from bits 63:32 of time\n
        /// always read timelr before timehr
        (0x008 => timehr: ReadOnly<u32, TIMEHR::Register>),
        /// Read from bits 31:0 of time
        (0x00C => timelr: ReadOnly<u32, TIMELR::Register>),
        /// Arm alarm 0, and configure the time it will fire.\n
        /// Once armed, the alarm fires when TIMER_ALARM0 == TIMELR.\n
        /// The alarm will disarm itself once it fires, and can\n
        /// be disarmed early using the ARMED status register.
        (0x010 => alarm0: ReadWrite<u32, ALARM0::Register>),
        /// Arm alarm 1, and configure the time it will fire.\n
        /// Once armed, the alarm fires when TIMER_ALARM1 == TIMELR.\n
        /// The alarm will disarm itself once it fires, and can\n
        /// be disarmed early using the ARMED status register.
        (0x014 => alarm1: ReadWrite<u32, ALARM1::Register>),
        /// Arm alarm 2, and configure the time it will fire.\n
        /// Once armed, the alarm fires when TIMER_ALARM2 == TIMELR.\n
        /// The alarm will disarm itself once it fires, and can\n
        /// be disarmed early using the ARMED status register.
        (0x018 => alarm2: ReadWrite<u32, ALARM2::Register>),
        /// Arm alarm 3, and configure the time it will fire.\n
        /// Once armed, the alarm fires when TIMER_ALARM3 == TIMELR.\n
        /// The alarm will disarm itself once it fires, and can\n
        /// be disarmed early using the ARMED status register.
        (0x01C => alarm3: ReadWrite<u32, ALARM3::Register>),
        /// Indicates the armed/disarmed status of each alarm.\n
        /// A write to the corresponding ALARMx register arms the alarm.\n
        /// Alarms automatically disarm upon firing, but writing ones here\n
        /// will disarm immediately without waiting to fire.
        (0x020 => armed: ReadWrite<u32>),
        /// Raw read from bits 63:32 of time (no side effects)
        (0x024 => timerawh: ReadOnly<u32, TIMERAWH::Register>),
        /// Raw read from bits 31:0 of time (no side effects)
        (0x028 => timerawl: ReadOnly<u32, TIMERAWL::Register>),
        /// Set bits high to enable pause when the corresponding debug ports are active
        (0x02C => dbgpause: ReadWrite<u32, DBGPAUSE::Register>),
        /// Set high to pause the timer
        (0x030 => pause: ReadWrite<u32>),
        /// Raw Interrupts
        (0x034 => intr: ReadWrite<u32, INTR::Register>),
        /// Interrupt Enable
        (0x038 => inte: ReadWrite<u32, INTE::Register>),
        /// Interrupt Force
        (0x03C => intf: ReadWrite<u32, INTF::Register>),
        /// Interrupt status after masking & forcing
        (0x040 => ints: ReadWrite<u32, INTS::Register>),
        (0x044 => @END),
    }
}
register_bitfields![u32,
TIMEHW [
    VALUE OFFSET (0) NUMBITS (32) []
],
TIMELW [
    VALUE OFFSET (0) NUMBITS (32) []
],
TIMEHR [
    VALUE OFFSET (0) NUMBITS (32) []
],
TIMELR [
    VALUE OFFSET (0) NUMBITS (32) []
],
ALARM0 [
    VALUE OFFSET (0) NUMBITS (32) []
],
ALARM1 [
    VALUE OFFSET (0) NUMBITS (32) []
],
ALARM2 [
    VALUE OFFSET (0) NUMBITS (32) []
],
ALARM3 [
    VALUE OFFSET (0) NUMBITS (32) []
],
ARMED [
    ARMED OFFSET(0) NUMBITS(4) []
],
TIMERAWH [
    VALUE OFFSET (0) NUMBITS (32) []
],
TIMERAWL [
    VALUE OFFSET (0) NUMBITS (32) []
],
DBGPAUSE [
    /// Pause when processor 1 is in debug mode
    DBG1 OFFSET(2) NUMBITS(1) [],
    /// Pause when processor 0 is in debug mode
    DBG0 OFFSET(1) NUMBITS(1) []
],
PAUSE [

    PAUSE OFFSET(0) NUMBITS(1) []
],
INTR [

    ALARM_3 OFFSET(3) NUMBITS(1) [],

    ALARM_2 OFFSET(2) NUMBITS(1) [],

    ALARM_1 OFFSET(1) NUMBITS(1) [],

    ALARM_0 OFFSET(0) NUMBITS(1) []
],
INTE [

    ALARM_3 OFFSET(3) NUMBITS(1) [],

    ALARM_2 OFFSET(2) NUMBITS(1) [],

    ALARM_1 OFFSET(1) NUMBITS(1) [],

    ALARM_0 OFFSET(0) NUMBITS(1) []
],
INTF [

    ALARM_3 OFFSET(3) NUMBITS(1) [],

    ALARM_2 OFFSET(2) NUMBITS(1) [],

    ALARM_1 OFFSET(1) NUMBITS(1) [],

    ALARM_0 OFFSET(0) NUMBITS(1) []
],
INTS [

    ALARM_3 OFFSET(3) NUMBITS(1) [],

    ALARM_2 OFFSET(2) NUMBITS(1) [],

    ALARM_1 OFFSET(1) NUMBITS(1) [],

    ALARM_0 OFFSET(0) NUMBITS(1) []
]
];
const TIMER_BASE: StaticRef<TimerRegisters> =
    unsafe { StaticRef::new(0x40054000 as *const TimerRegisters) };

pub struct RPTimer<'a> {
    registers: StaticRef<TimerRegisters>,
    client: OptionalCell<&'a dyn hil::time::AlarmClient>,
}

impl<'a> RPTimer<'a> {
    pub const fn new() -> RPTimer<'a> {
        RPTimer {
            registers: TIMER_BASE,
            client: OptionalCell::empty(),
        }
    }

    fn enable_interrupt(&self) {
        self.registers.inte.modify(INTE::ALARM_0::SET);
    }

    fn disable_interrupt(&self) {
        self.registers.inte.modify(INTE::ALARM_0::CLEAR);
    }

    fn enable_timer_interrupt(&self) {
        // Even though setting the INTE::ALARM_0 bit should be enough to enable
        // the interrupt firing, it seems that RP2040 requires manual NVIC
        // enabling of the interrupt.
        //
        // Failing to do so results in the interrupt being set as pending but
        // not fired. This means that the interrupt will be handled whenever the
        // next kernel tasks are processed.
        unsafe {
            atomic(|| {
                let n = cortexm0p::nvic::Nvic::new(TIMER_IRQ_0);
                n.enable();
            })
        }
    }

    fn disable_timer_interrupt(&self) {
        // Even though clearing the INTE::ALARM_0 bit should be enough to disable
        // the interrupt firing, it seems that RP2040 requires manual NVIC
        // disabling of the interrupt.
        unsafe {
            cortexm0p::nvic::Nvic::new(TIMER_IRQ_0).disable();
        }
    }

    pub fn handle_interrupt(&self) {
        self.registers.intr.modify(INTR::ALARM_0::SET);
        self.client.map(|client| client.alarm());
    }
}

impl Time for RPTimer<'_> {
    type Frequency = hil::time::Freq1MHz;
    type Ticks = Ticks32;

    fn now(&self) -> Self::Ticks {
        Self::Ticks::from(self.registers.timerawl.get())
    }
}

impl<'a> Alarm<'a> for RPTimer<'a> {
    fn set_alarm_client(&self, client: &'a dyn hil::time::AlarmClient) {
        self.client.set(client);
    }

    fn set_alarm(&self, reference: Self::Ticks, dt: Self::Ticks) {
        let mut expire = reference.wrapping_add(dt);
        let now = self.now();
        if !now.within_range(reference, expire) {
            expire = now;
        }

        if expire.wrapping_sub(now) < self.minimum_dt() {
            expire = now.wrapping_add(self.minimum_dt());
        }

        self.registers.alarm0.set(expire.into_u32());
        self.enable_timer_interrupt();
        self.enable_interrupt();
    }

    fn get_alarm(&self) -> Self::Ticks {
        Self::Ticks::from(self.registers.alarm0.get())
    }

    fn disarm(&self) -> Result<(), ErrorCode> {
        self.registers.armed.set(1);
        unsafe {
            atomic(|| {
                // Clear pending interrupts
                cortexm0p::nvic::Nvic::new(TIMER_IRQ_0).clear_pending();
            });
        }
        self.disable_interrupt();
        self.disable_timer_interrupt();
        Ok(())
    }

    fn is_armed(&self) -> bool {
        let armed = self.registers.armed.get() & 0b0001;
        if armed == 1 {
            return true;
        }
        false
    }

    fn minimum_dt(&self) -> Self::Ticks {
        Self::Ticks::from(50)
    }
}

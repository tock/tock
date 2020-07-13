//! Timer driver.

use kernel::common::cells::OptionalCell;
use kernel::common::registers::{register_bitfields, register_structs, ReadWrite, WriteOnly};
use kernel::common::StaticRef;
use kernel::hil::time;

use crate::chip::CHIP_FREQ;

const PRESCALE: u16 = ((CHIP_FREQ / 10_000) - 1) as u16; // 10Khz

/// 10KHz `Frequency`
#[derive(Debug)]
pub struct Freq10KHz;
impl time::Frequency for Freq10KHz {
    fn frequency() -> u32 {
        10_000
    }
}

register_structs! {
    pub TimerRegisters {
        (0x000 => ctrl: ReadWrite<u32, ctrl::Register>),

        (0x004 => _reserved),

        (0x100 => config: ReadWrite<u32, config::Register>),

        (0x104 => value_low: ReadWrite<u32>),
        (0x108 => value_high: ReadWrite<u32>),

        (0x10c => compare_low: ReadWrite<u32>),
        (0x110 => compare_high: ReadWrite<u32>),

        (0x114 => intr_enable: ReadWrite<u32, intr::Register>),
        (0x118 => intr_state: ReadWrite<u32, intr::Register>),
        (0x11c => intr_test: WriteOnly<u32, intr::Register>),
        (0x120 => @END),
    }
}

register_bitfields![u32,
    ctrl [
        enable OFFSET(0) NUMBITS(1) []
    ],
    config [
        prescale OFFSET(0) NUMBITS(12) [],
        step OFFSET(16) NUMBITS(8) []
    ],
    intr [
        timer0 OFFSET(0) NUMBITS(1) []
    ]
];

pub struct RvTimer<'a> {
    registers: StaticRef<TimerRegisters>,
    client: OptionalCell<&'a dyn time::AlarmClient>,
}

impl<'a> RvTimer<'a> {
    const fn new(base: StaticRef<TimerRegisters>) -> RvTimer<'a> {
        RvTimer {
            registers: base,
            client: OptionalCell::empty(),
        }
    }

    pub fn setup(&self) {
        let regs = self.registers;
        // Set proper prescaler and the like
        regs.config
            .write(config::prescale.val(PRESCALE as u32) + config::step.val(1u32));
        regs.compare_high.set(0);
        regs.intr_enable.write(intr::timer0::CLEAR);
        regs.ctrl.write(ctrl::enable::SET);
    }

    pub fn service_interrupt(&self) {
        let regs = self.registers;

        regs.intr_enable.write(intr::timer0::CLEAR);
        regs.intr_state.write(intr::timer0::SET);
        self.client.map(|client| {
            client.fired();
        });
    }
}

impl time::Time for RvTimer<'_> {
    type Frequency = Freq10KHz;

    fn now(&self) -> u32 {
        self.registers.value_low.get()
    }
    fn max_tics(&self) -> u32 {
        core::u32::MAX
    }
}

impl<'a> time::Alarm<'a> for RvTimer<'a> {
    fn set_client(&self, client: &'a dyn time::AlarmClient) {
        self.client.set(client);
    }

    fn set_alarm(&self, tics: u32) {
        let regs = self.registers;

        // Make sure that any overlow into the high bits of the timer (which we are ignoring for
        // now) do not have an effect on the alarm.
        regs.value_high.set(0);

        regs.compare_low.set(tics);
        regs.intr_enable.write(intr::timer0::SET);
    }

    fn get_alarm(&self) -> u32 {
        self.registers.compare_low.get()
    }

    fn disable(&self) {
        self.registers.intr_enable.write(intr::timer0::CLEAR);
    }

    fn is_enabled(&self) -> bool {
        self.registers.intr_enable.is_set(intr::timer0)
    }
}

const TIMER_BASE: StaticRef<TimerRegisters> =
    unsafe { StaticRef::new(0x4008_0000 as *const TimerRegisters) };

pub static mut TIMER: RvTimer = RvTimer::new(TIMER_BASE);

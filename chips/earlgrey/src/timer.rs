//! Timer driver.

use crate::chip_config::CONFIG;
use kernel::hil::time::{self, Ticks64};
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::registers::interfaces::{Readable, Writeable};
use kernel::utilities::registers::{register_bitfields, register_structs, ReadWrite, WriteOnly};
use kernel::utilities::StaticRef;
use kernel::ErrorCode;
use rv32i::machine_timer::MachineTimer;

const PRESCALE: u16 = ((CONFIG.cpu_freq / 10_000) - 1) as u16; // 10Khz

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
    alarm_client: OptionalCell<&'a dyn time::AlarmClient>,
    overflow_client: OptionalCell<&'a dyn time::OverflowClient>,
    mtimer: MachineTimer<'a>,
}

impl<'a> RvTimer<'a> {
    pub fn new() -> Self {
        Self {
            registers: TIMER_BASE,
            alarm_client: OptionalCell::empty(),
            overflow_client: OptionalCell::empty(),
            mtimer: MachineTimer::new(
                &TIMER_BASE.compare_low,
                &TIMER_BASE.compare_high,
                &TIMER_BASE.value_low,
                &TIMER_BASE.value_high,
            ),
        }
    }

    pub fn setup(&self) {
        let regs = self.registers;
        // Set proper prescaler and the like
        regs.config
            .write(config::prescale.val(PRESCALE as u32) + config::step.val(1u32));
        regs.compare_high.set(0);
        regs.value_low.set(0xFFFF_0000);
        regs.intr_enable.write(intr::timer0::CLEAR);
        regs.ctrl.write(ctrl::enable::SET);
    }

    pub fn service_interrupt(&self) {
        let regs = self.registers;
        regs.intr_enable.write(intr::timer0::CLEAR);
        regs.intr_state.write(intr::timer0::SET);
        self.alarm_client.map(|client| {
            client.alarm();
        });
    }
}

impl time::Time for RvTimer<'_> {
    type Frequency = Freq10KHz;
    type Ticks = Ticks64;

    fn now(&self) -> Ticks64 {
        self.mtimer.now()
    }
}

impl<'a> time::Counter<'a> for RvTimer<'a> {
    fn set_overflow_client(&'a self, client: &'a dyn time::OverflowClient) {
        self.overflow_client.set(client);
    }

    fn start(&self) -> Result<(), ErrorCode> {
        Ok(())
    }

    fn stop(&self) -> Result<(), ErrorCode> {
        // RISCV counter can't be stopped...
        Err(ErrorCode::BUSY)
    }

    fn reset(&self) -> Result<(), ErrorCode> {
        // RISCV counter can't be reset
        Err(ErrorCode::FAIL)
    }

    fn is_running(&self) -> bool {
        true
    }
}

impl<'a> time::Alarm<'a> for RvTimer<'a> {
    fn set_alarm_client(&self, client: &'a dyn time::AlarmClient) {
        self.alarm_client.set(client);
    }

    fn set_alarm(&self, reference: Self::Ticks, dt: Self::Ticks) {
        self.registers.intr_enable.write(intr::timer0::SET);

        self.mtimer.set_alarm(reference, dt)
    }

    fn get_alarm(&self) -> Self::Ticks {
        self.mtimer.get_alarm()
    }

    fn disarm(&self) -> Result<(), ErrorCode> {
        self.registers.intr_enable.write(intr::timer0::CLEAR);

        self.mtimer.disarm()
    }

    fn is_armed(&self) -> bool {
        self.registers.intr_enable.is_set(intr::timer0)
    }

    fn minimum_dt(&self) -> Self::Ticks {
        self.mtimer.minimum_dt()
    }
}

const TIMER_BASE: StaticRef<TimerRegisters> =
    unsafe { StaticRef::new(0x4010_0000 as *const TimerRegisters) };

//! Timer driver.

use kernel::common::cells::OptionalCell;
use kernel::common::registers::{register_bitfields, register_structs, ReadWrite, WriteOnly};
use kernel::common::StaticRef;
use kernel::hil::time;
use kernel::hil::time::{Ticks, Ticks64, Time};
use kernel::ReturnCode;

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
    alarm_client: OptionalCell<&'a dyn time::AlarmClient>,
    overflow_client: OptionalCell<&'a dyn time::OverflowClient>,
}

impl<'a> RvTimer<'a> {
    const fn new(base: StaticRef<TimerRegisters>) -> RvTimer<'a> {
        RvTimer {
            registers: base,
            alarm_client: OptionalCell::empty(),
            overflow_client: OptionalCell::empty(),
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
        // RISC-V has a 64-bit counter but you can only read 32 bits
        // at once, which creates a race condition if the lower register
        // wraps between the reads. So the recommended approach is to read
        // low, read high, read low, and if the second low is lower, re-read
        // high. -pal 8/6/20
        let first_low: u32 = self.registers.value_low.get();
        let mut high: u32 = self.registers.value_high.get();
        let second_low: u32 = self.registers.value_low.get();
        if second_low < first_low {
            // Wraparound
            high = self.registers.value_high.get();
        }
        Ticks64::from(((high as u64) << 32) | second_low as u64)
    }
}

impl<'a> time::Counter<'a> for RvTimer<'a> {
    fn set_overflow_client(&'a self, client: &'a dyn time::OverflowClient) {
        self.overflow_client.set(client);
    }

    fn start(&self) -> ReturnCode {
        ReturnCode::SUCCESS
    }

    fn stop(&self) -> ReturnCode {
        // RISCV counter can't be stopped...
        ReturnCode::EBUSY
    }

    fn reset(&self) -> ReturnCode {
        // RISCV counter can't be reset
        ReturnCode::FAIL
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
        // This does not handle the 64-bit wraparound case.
        // Because mtimer fires if the counter is >= the compare,
        // handling wraparound requires setting compare to the
        // maximum value, issuing a callback on the overflow client
        // if there is one, spinning until it wraps around to 0, then
        // setting the compare to the correct value.
        let regs = self.registers;
        let now = self.now();
        let mut expire = reference.wrapping_add(dt);

        if !now.within_range(reference, expire) {
            expire = now;
        }

        let val = expire.into_u64();
        let high = (val >> 32) as u32;
        let low = (val & 0xffffffff) as u32;

        // Recommended approach for setting the two compare registers
        // (RISC-V Privileged Architectures 3.1.15) -pal 8/6/20
        regs.compare_low.set(0xffffffff);
        regs.compare_high.set(high);
        regs.compare_low.set(low);
	//debug!("TIMER: set to {}", expire.into_u64());
        self.registers.intr_enable.write(intr::timer0::SET);
    }

    fn get_alarm(&self) -> Self::Ticks {
        let mut val: u64 = (self.registers.compare_high.get() as u64) << 32;
        val |= self.registers.compare_low.get() as u64;
        Ticks64::from(val)
    }

    fn disarm(&self) -> ReturnCode {
        // You clear the RISCV mtime interrupt by writing to the compare
        // registers. Since the only way to do so is to set a new alarm,
        // and this is also the only way to re-enable the interrupt, disabling
        // the interrupt is sufficient. Calling set_alarm will clear the
        // pending interrupt before re-enabling. -pal 8/6/20
        self.registers.intr_enable.write(intr::timer0::CLEAR);
        ReturnCode::SUCCESS
    }

    fn is_armed(&self) -> bool {
        self.registers.intr_enable.is_set(intr::timer0)
    }

    fn minimum_dt(&self) -> Self::Ticks {
        Self::Ticks::from(1 as u64)
    }
}

const TIMER_BASE: StaticRef<TimerRegisters> =
    unsafe { StaticRef::new(0x4008_0000 as *const TimerRegisters) };

pub static mut TIMER: RvTimer = RvTimer::new(TIMER_BASE);

//! Create a timer using the Machine Timer registers.

use kernel::hil::time::{self, Alarm, ConvertTicks, Freq32KHz, Frequency, Ticks, Ticks64, Time};
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::registers::interfaces::Writeable;
use kernel::utilities::registers::{register_structs, ReadWrite};
use kernel::utilities::StaticRef;
use kernel::ErrorCode;
use rv32i::machine_timer::MachineTimer;

register_structs! {
    pub ClintRegisters {
        (0x0000 => msip: ReadWrite<u32>),
        (0x0004 => _reserved),
        (0x4000 => compare_low: ReadWrite<u32>),
        (0x4004 => compare_high: ReadWrite<u32>),
        (0x4008 => _reserved2),
        (0xBFF8 => value_low: ReadWrite<u32>),
        (0xBFFC => value_high: ReadWrite<u32>),
        (0xC000 => @END),
    }
}

pub struct Clint<'a> {
    registers: StaticRef<ClintRegisters>,
    client: OptionalCell<&'a dyn time::AlarmClient>,
    mtimer: MachineTimer<'a>,
}

impl<'a> Clint<'a> {
    pub fn new(base: &'a StaticRef<ClintRegisters>) -> Self {
        Self {
            registers: *base,
            client: OptionalCell::empty(),
            mtimer: MachineTimer::new(
                &base.compare_low,
                &base.compare_high,
                &base.value_low,
                &base.value_high,
            ),
        }
    }

    pub fn handle_interrupt(&self) {
        self.disable_machine_timer();

        self.client.map(|client| {
            client.alarm();
        });
    }

    pub fn disable_machine_timer(&self) {
        self.registers.compare_high.set(0xFFFF_FFFF);
        self.registers.compare_low.set(0xFFFF_FFFF);
    }
}

impl Time for Clint<'_> {
    type Frequency = Freq32KHz;
    type Ticks = Ticks64;

    fn now(&self) -> Ticks64 {
        self.mtimer.now()
    }
}

impl<'a> time::Alarm<'a> for Clint<'a> {
    fn set_alarm_client(&self, client: &'a dyn time::AlarmClient) {
        self.client.set(client);
    }

    fn set_alarm(&self, reference: Self::Ticks, dt: Self::Ticks) {
        self.mtimer.set_alarm(reference, dt)
    }

    fn get_alarm(&self) -> Self::Ticks {
        self.mtimer.get_alarm()
    }

    fn disarm(&self) -> Result<(), ErrorCode> {
        self.mtimer.disarm()
    }

    fn is_armed(&self) -> bool {
        self.mtimer.is_armed()
    }

    fn minimum_dt(&self) -> Self::Ticks {
        self.mtimer.minimum_dt()
    }
}

/// SchedulerTimer Implementation for RISC-V mtimer. Notably, this implementation should only be
/// used by a chip if that chip has multiple hardware timer peripherals such that a different
/// hardware timer can be used to provide alarms to capsules and userspace. This
/// implementation will not work alongside other uses of the machine timer.
impl kernel::platform::scheduler_timer::SchedulerTimer for Clint<'_> {
    fn start(&self, us: u32) {
        let now = self.now();
        let tics = self.ticks_from_us(us);
        self.set_alarm(now, tics);
    }

    fn get_remaining_us(&self) -> Option<u32> {
        // We need to convert from native tics to us, multiplication could overflow in 32-bit
        // arithmetic. So we convert to 64-bit.
        let diff = self.get_alarm().wrapping_sub(self.now()).into_u64();

        // If next alarm is more than one second away from now, alarm must have expired.
        // Use this formulation to protect against errors when the alarm has passed.
        // 1 second was chosen because it is significantly greater than the 400ms max value allowed
        // by start(), and requires no computational overhead (e.g. using 500ms would require
        // dividing the returned ticks by 2)
        // However, if the alarm frequency is slow enough relative to the cpu frequency, it is
        // possible this will be evaluated while now() == get_alarm(), so we special case that
        // result where the alarm has fired but the subtraction has not overflowed
        if diff >= <Self as Time>::Frequency::frequency() as u64 || diff == 0 {
            None
        } else {
            let hertz = <Self as Time>::Frequency::frequency() as u64;
            Some(((diff * 1_000_000) / hertz) as u32)
        }
    }

    fn reset(&self) {
        self.disable_machine_timer();
    }

    fn arm(&self) {
        // Arm and disarm are optional, but controlling the mtimer interrupt
        // should be re-enabled if Tock moves to a design that allows direct control of
        // interrupt enables
        //csr::CSR.mie.modify(csr::mie::mie::mtimer::SET);
    }

    fn disarm(&self) {
        //csr::CSR.mie.modify(csr::mie::mie::mtimer::CLEAR);
    }
}

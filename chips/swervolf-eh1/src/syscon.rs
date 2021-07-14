//! System Controller driver.

use kernel::hil::time::{self, Ticks64};
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::registers::interfaces::Writeable;
use kernel::utilities::registers::{register_structs, ReadWrite};
use kernel::utilities::StaticRef;
use kernel::ErrorCode;
use rv32i::machine_timer::MachineTimer;

/// 100Hz `Frequency`
#[derive(Debug)]
pub struct Freq100Hz;
impl time::Frequency for Freq100Hz {
    fn frequency() -> u32 {
        100
    }
}

register_structs! {
    pub SysConRegisters {
        /// SweRVolf patch version
        (0x000 => version_patch: ReadWrite<u8>),
        /// SweRVolf minor version
        (0x001 => version_minor: ReadWrite<u8>),
        /// SweRVolf major version
        (0x002 => version_major: ReadWrite<u8>),
        /// Bit 7 is set when SweRVolf was built from modified sources
        /// Bit 6:0 revision since last patch version
        (0x003 => version_misc: ReadWrite<u8>),
        /// SHA hash of the build
        (0x004 => version_sha: ReadWrite<u32>),
        /// Outputs a character in simulation. No effect on hardware
        (0x008 => sim_print: ReadWrite<u8>),
        /// Exits a simulation. No effect on hardware
        (0x009 => sim_exit: ReadWrite<u8>),
        /// Bit 0 = RAM initialization complete. Bit 1 = RAM initialization reported errors
        (0x00A => init_status: ReadWrite<u8>),
        /// Software-controlled external interrupts
        (0x00B => sw_irq: ReadWrite<u8>),
        /// Interrupt vector for NMI
        (0x00C => nmi_vec: ReadWrite<u32>),
        /// 64 readable and writable GPIO bits
        (0x010 => gpio: [ReadWrite<u32>; 2]),
        (0x018 => _reserved0),
        /// mtime from RISC-V privilege spec
        (0x020 => mtime_low: ReadWrite<u32>),
        (0x024 => mtime_high: ReadWrite<u32>),
        /// mtimecmp from RISC-V privilege spec
        (0x028 => mtimecmp_low: ReadWrite<u32>),
        (0x02C => mtimecmp_high: ReadWrite<u32>),
        /// IRQ timer counter
        (0x030 => irq_timer_cnt: ReadWrite<u32>),
        /// IRQ timer control
        (0x034 => irq_timer_ctrl: ReadWrite<u8>),
        (0x035 => _reserved1),
        /// Clock frequency of main clock in Hz
        (0x03C => clk_freq_hz: ReadWrite<u32>),
        /// Simple SPI Control register
        (0x040 => spi_spcr: ReadWrite<u64>),
        /// Simple SPI status register
        (0x048 => spi_spsr: ReadWrite<u64>),
        /// Simple SPI data register
        (0x050 => spi_spdr: ReadWrite<u64>),
        /// Simple SPI extended register
        (0x058 => spi_sper: ReadWrite<u64>),
        /// Simple SPI slave select register
        (0x060 => spi_spss: ReadWrite<u64>),
        (0x068 => @END),
    }
}

pub struct SysCon<'a> {
    registers: StaticRef<SysConRegisters>,
    alarm_client: OptionalCell<&'a dyn time::AlarmClient>,
    overflow_client: OptionalCell<&'a dyn time::OverflowClient>,
    mtimer: MachineTimer<'a>,
}

impl<'a> SysCon<'a> {
    pub fn new() -> Self {
        Self {
            registers: SYSCON_BASE,
            alarm_client: OptionalCell::empty(),
            overflow_client: OptionalCell::empty(),
            mtimer: MachineTimer::new(
                &SYSCON_BASE.mtimecmp_low,
                &SYSCON_BASE.mtimecmp_high,
                &SYSCON_BASE.mtime_low,
                &SYSCON_BASE.mtime_high,
            ),
        }
    }

    pub fn handle_interrupt(&self) {
        self.registers.mtimecmp_low.set(0xFFFF_FFFF);
        self.registers.mtimecmp_high.set(0xFFFF_FFFF);
        self.alarm_client.map(|client| {
            client.alarm();
        });
    }
}

impl time::Time for SysCon<'_> {
    type Frequency = Freq100Hz;
    type Ticks = Ticks64;

    fn now(&self) -> Ticks64 {
        self.mtimer.now()
    }
}

impl<'a> time::Counter<'a> for SysCon<'a> {
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

impl<'a> time::Alarm<'a> for SysCon<'a> {
    fn set_alarm_client(&self, client: &'a dyn time::AlarmClient) {
        self.alarm_client.set(client);
    }

    fn set_alarm(&self, reference: Self::Ticks, dt: Self::Ticks) {
        self.mtimer.set_alarm(reference, dt);

        self.registers.irq_timer_ctrl.set(0xFF);
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

const SYSCON_BASE: StaticRef<SysConRegisters> =
    unsafe { StaticRef::new(0x8000_1000 as *const SysConRegisters) };

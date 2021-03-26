//! System Controller driver.

use kernel::common::cells::OptionalCell;
use kernel::common::registers::{register_structs, ReadWrite};
use kernel::common::StaticRef;
use kernel::hil::time;
use kernel::hil::time::{Ticks, Ticks64, Time};
use kernel::ErrorCode;
use kernel::ReturnCode;

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
}

impl<'a> SysCon<'a> {
    pub const fn new() -> SysCon<'a> {
        SysCon {
            registers: SYSCON_BASE,
            alarm_client: OptionalCell::empty(),
            overflow_client: OptionalCell::empty(),
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
        // RISC-V has a 64-bit counter but you can only read 32 bits
        // at once, which creates a race condition if the lower register
        // wraps between the reads. So the recommended approach is to read
        // low, read high, read low, and if the second low is lower, re-read
        // high. -pal 8/6/20
        let first_low: u32 = self.registers.mtime_low.get();
        let mut high: u32 = self.registers.mtime_high.get();
        let second_low: u32 = self.registers.mtime_low.get();
        if second_low < first_low {
            // Wraparound
            high = self.registers.mtime_high.get();
        }
        Ticks64::from(((high as u64) << 32) | second_low as u64)
    }
}

impl<'a> time::Counter<'a> for SysCon<'a> {
    fn set_overflow_client(&'a self, client: &'a dyn time::OverflowClient) {
        self.overflow_client.set(client);
    }

    fn start(&self) -> ReturnCode {
        Ok(())
    }

    fn stop(&self) -> ReturnCode {
        // RISCV counter can't be stopped...
        Err(ErrorCode::BUSY)
    }

    fn reset(&self) -> ReturnCode {
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
        regs.mtimecmp_low.set(0xffffffff);
        regs.mtimecmp_high.set(high);
        regs.mtimecmp_low.set(low);
        self.registers.irq_timer_ctrl.set(0xFF);
    }

    fn get_alarm(&self) -> Self::Ticks {
        let mut val: u64 = (self.registers.mtimecmp_high.get() as u64) << 32;
        val |= self.registers.mtimecmp_low.get() as u64;
        Ticks64::from(val)
    }

    fn disarm(&self) -> ReturnCode {
        // We don't appear to be able to disarm the alarm, so just
        // set it to the future.
        self.registers.mtimecmp_low.set(0xFFFF_FFFF);
        self.registers.mtimecmp_high.set(0xFFFF_FFFF);
        Ok(())
    }

    fn is_armed(&self) -> bool {
        self.registers.mtimecmp_low.get() == 0xFFFF_FFFF
            && self.registers.mtimecmp_high.get() == 0xFFFF_FFFF
    }

    fn minimum_dt(&self) -> Self::Ticks {
        Self::Ticks::from(1 as u64)
    }
}

const SYSCON_BASE: StaticRef<SysConRegisters> =
    unsafe { StaticRef::new(0x8000_1000 as *const SysConRegisters) };

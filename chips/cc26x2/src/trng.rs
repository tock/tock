//! TRNG - Random Number Generator for the cc26x2 family
//!
//! Generates a random number using hardware entropy.
//!

use kernel::common::cells::OptionalCell;
use kernel::common::registers::{ReadOnly, ReadWrite, WriteOnly};
use kernel::common::StaticRef;
use kernel::hil::rng;
use prcm;

#[repr(C)]
struct RngRegisters {
    out0: ReadOnly<u32>,
    out1: ReadOnly<u32>,

    irq_flag_stat: ReadOnly<u32, IrqStatus::Register>,
    _irq_flag_mask: ReadOnly<u32>,
    irq_flag_clr: WriteOnly<u32, IrqFlagClear::Register>,

    ctl: ReadWrite<u32, Control::Register>,
    cfg0: ReadWrite<u32, Config::Register>,

    alarm_ctl: ReadWrite<u32, AlarmControl::Register>,

    _fro_en: ReadOnly<u32>,
    _fro_detune: ReadOnly<u32>,

    _alarm_mask: ReadOnly<u32>,
    _alarm_stop: ReadOnly<u32>,

    _lfsr0: ReadOnly<u32>,
    _lfsr1: ReadOnly<u32>,
    _lfsr2: ReadOnly<u32>,

    _r0: [u8; 0x1FB4],

    sw_reset: ReadWrite<u32, SoftwareReset::Register>,
}

register_bitfields![
    u32,
    IrqStatus [
        READY OFFSET(0) NUMBITS(1) []
    ],
    IrqFlagClear [
        READY OFFSET(0) NUMBITS(1) []
    ],
    Control [
        STARTUP_CYCLES  OFFSET(16) NUMBITS(16)  [],
        TRNG_EN         OFFSET(10) NUMBITS(1)   []
    ],
    Config [
        MAX_REFILL_CYCLES   OFFSET(16) NUMBITS(16) [],
        SMPL_DIV            OFFSET(8)  NUMBITS(4)  [],
        MIN_REFILL_CYCLES   OFFSET(0)  NUMBITS(8)  []
    ],
    AlarmControl [
        // Alarm threshold for repeating pattern detectors
        ALARM_THR   OFFSET(0)   NUMBITS(8) []
    ],
    SoftwareReset [
        RESET OFFSET(0) NUMBITS(1) []
    ]
];

const RNG_BASE: StaticRef<RngRegisters> =
    unsafe { StaticRef::new(0x40028000 as *const RngRegisters) };

pub static mut TRNG: Trng = Trng::new();

pub struct Trng {
    registers: StaticRef<RngRegisters>,
    client: OptionalCell<&'static rng::Client>,
}

impl Trng {
    const fn new() -> Trng {
        Trng {
            registers: RNG_BASE,
            client: OptionalCell::empty(),
        }
    }

    pub fn enable(&self) {
        // Ensure that the power domain TRNG resides in is enabled
        if !prcm::Power::is_enabled(prcm::PowerDomain::Peripherals) {
            prcm::Power::enable_domain(prcm::PowerDomain::Peripherals);

            while !prcm::Power::is_enabled(prcm::PowerDomain::Peripherals) {}
        }

        // Setup the clock
        prcm::Clock::enable_trng();

        let regs = &*self.registers;

        regs.ctl.set(0);

        // Issue a SW reset
        regs.sw_reset.write(SoftwareReset::RESET::SET);
        while regs.sw_reset.is_set(SoftwareReset::RESET) {}

        // Set the startup samples
        regs.ctl.modify(Control::STARTUP_CYCLES.val(1));

        // Configure the minimum and maximum number of samples per generated number
        // and the number of clock cycles per sample.
        //  NOTE: tune these if the generation is not satisfactory
        let max_samples_per_cycle = 0x100;
        let min_samples_per_cycle = 0;
        let cycles_per_sample = 0;
        regs.cfg0.write(
            Config::MAX_REFILL_CYCLES.val(max_samples_per_cycle >> 8) +
                Config::SMPL_DIV.val(cycles_per_sample) +
                Config::MIN_REFILL_CYCLES.val(min_samples_per_cycle >> 6),
        );

        // Reset the alarm control
        regs.alarm_ctl.write(AlarmControl::ALARM_THR.val(0xFF));

        // Enable the TRNG
        regs.ctl.modify(Control::TRNG_EN::SET);
    }

    pub fn read_number_blocking(&self) -> u64 {
        let regs = &*self.registers;

        if !regs.ctl.is_set(Control::TRNG_EN) {
            self.enable();
        }

        // Wait for a number to be ready
        while !regs.irq_flag_stat.is_set(IrqStatus::READY) {}

        // Initiate generation of a new number
        regs.irq_flag_clr.write(IrqFlagClear::READY::SET);

        ((regs.out0.get() as u64) << 32) | (regs.out1.get() as u64)
    }

    pub fn set_client(&self, client: &'static rng::Client) {
        self.client.set(client);
    }
}

impl Iterator for Trng {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        let regs = &*self.registers;
        if regs.ctl.is_set(Control::TRNG_EN) {
            Some((self.read_number_blocking() & 0xFFFFFFFF) as u32)
        } else {
            None
        }
    }
}

impl rng::RNG for Trng {
    fn get(&self) {
        self.enable();
    }
}

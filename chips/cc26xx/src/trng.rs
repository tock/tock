use core::cell::Cell;
use kernel::common::VolatileCell;
use kernel::hil::rng;

use prcm;

#[repr(C)]
struct RngRegisters {
    out0: VolatileCell<u32>,
    out1: VolatileCell<u32>,

    irq_flag_stat: VolatileCell<u32>,
    _irq_flag_mask: VolatileCell<u32>,
    irq_flag_clr: VolatileCell<u32>,

    ctl: VolatileCell<u32>,
    cfg0: VolatileCell<u32>,

    alarm_ctl: VolatileCell<u32>,

    _fro_en: VolatileCell<u32>,
    _fro_detune: VolatileCell<u32>,

    _alarm_mask: VolatileCell<u32>,
    _alarm_stop: VolatileCell<u32>,

    _lfsr0: VolatileCell<u32>,
    _lfsr1: VolatileCell<u32>,
    _lfsr2: VolatileCell<u32>,

    _r0: [u8; 0x1FB4],

    sw_reset: VolatileCell<u32>,
}

const BASE_ADDRESS: *mut RngRegisters = 0x4002_8000 as *mut RngRegisters;

pub static mut TRNG: Trng = Trng::new();

const TRNG_CTL_EN: u32 = (1 << 10);

const TRNG_CFG_MAX_REFILL_CYCLES: u32 = 0xFFFF0000;
const TRNG_CFG_MIN_REFILL_CYCLES: u32 = 0x000000FF;
const TRNG_CFG_SIMPL_DIV: u32 = 0x00000F00;

const TRNG_STATUS_NUMBER_READY: u32 = 0x01;

pub struct Trng {
    regs: *mut RngRegisters,
    client: Cell<Option<&'static rng::Client>>,
}

impl Trng {
    const fn new() -> Trng {
        Trng {
            regs: BASE_ADDRESS,
            client: Cell::new(None),
        }
    }

    pub fn enable(&self) {
        // Ensure that the power domain TRNG resides in is enabled
        if !prcm::Power::is_enabled(prcm::PowerDomain::Peripherals) {
            prcm::Power::enable_domain(prcm::PowerDomain::Peripherals);

            while !prcm::Power::is_enabled(prcm::PowerDomain::Peripherals) { }
        }

        // Setup the clock
        prcm::Clock::enable_trng();

        let regs = unsafe { &*self.regs };

        regs.ctl.set(0);

        // Issue a SW reset
        regs.sw_reset.set(1);
        while regs.sw_reset.get() != 0 { }

        // Set the startup samples
        regs.ctl.set(regs.ctl.get() | (1 << 16));

        // Configure the minimum and maximum number of samples per generated number
        // and the number of clock cycles per sample.
        //  NOTE: tune these if the generation is not satisfactory
        let max_samples_per_cycle = 0x100;
        let min_samples_per_cycle = 0;
        let cycles_per_sample = 0;
        regs.cfg0.set(
            ((max_samples_per_cycle >> 8) << 16) & TRNG_CFG_MAX_REFILL_CYCLES
            | (cycles_per_sample << 8) & TRNG_CFG_SIMPL_DIV
            | (min_samples_per_cycle >> 6) & TRNG_CFG_MIN_REFILL_CYCLES
        );

        // Reset the alarm control
        regs.alarm_ctl.set(0xFF);

        // Enable the TRNG
        regs.ctl.set(regs.ctl.get() | TRNG_CTL_EN);
    }

    pub fn read_number(&self) -> u64 {
        let regs = unsafe { &*self.regs };

        // Wait for a number to be ready
        while (regs.irq_flag_stat.get() & TRNG_STATUS_NUMBER_READY) == 0 { };

        // Initiate generation of a new number
        regs.irq_flag_clr.set(0x1);

        ((regs.out0.get() as u64) << 32) | (regs.out1.get() as u64)
    }

    pub fn set_client(&self, client: &'static rng::Client) {
        self.client.set(Some(client));
    }
}

impl Iterator for Trng {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        let regs = unsafe { &*self.regs };
        if (regs.ctl.get() & TRNG_CTL_EN) != 0 {
            Some((self.read_number() & 0xFFFFFFFF) as u32)
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

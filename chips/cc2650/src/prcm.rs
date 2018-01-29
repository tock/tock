/* PRCM - Power, Clock, and Reset Management */

use kernel::common::VolatileCell;

pub const PRCM_BASE: usize = 0x4008_2000;

#[repr(C)]
struct PRCM {
    _reserved0: [VolatileCell<u8>; 0x28],

    // Write 1 in order to load settings
    pub clk_load_ctl: VolatileCell<u32>,

    _reserved1: [VolatileCell<u8>; 0x1C],

    pub gpio_clk_gate_run: VolatileCell<u32>,
    pub gpio_clk_gate_sleep: VolatileCell<u32>,
    pub gpio_clk_gate_deep_sleep: VolatileCell<u32>,

    _reserved2: [VolatileCell<u8>; 0xD8],

    // Power domain control 0
    pub pd_ctl0: VolatileCell<u32>,
    pub pd_ctl0_rfc: VolatileCell<u32>,
    pub pd_ctl0_serial: VolatileCell<u32>,
    pub pd_ctl0_peripheral: VolatileCell<u32>,

    _reserved3: [VolatileCell<u8>; 0x04],

    // Power domain status 0
    pub pd_stat0: VolatileCell<u32>,
    pub pd_stat0_rfc: VolatileCell<u32>,
    pub pd_stat0_serial: VolatileCell<u32>,
    pub pd_stat0_periph: VolatileCell<u32>,
}

#[allow(non_snake_case)]
fn PRCM() -> &'static PRCM {
    unsafe {
        &*(PRCM_BASE as *const PRCM)
    }
}

/*
    In order to save changes to the PRCM, we need to
    trigger
*/
fn prcm_commit() {
    PRCM().clk_load_ctl.set(1);
}

pub enum PowerDomain {
    // Note: when RFC is to be enabled, you are required to use both
    // power domains (i.e enable RFC on both PowerDomain0 and PowerDomain1)
    RFC,
    Serial,
    Peripherals,
    VIMS,
}

pub struct Power (());

impl Power {
    pub fn enable_domain(domain: PowerDomain) {
        match domain {
            PowerDomain::Peripherals => {
                PRCM().pd_ctl0.set(PRCM().pd_ctl0.get() | 0x4);
            },
            _ => {
                panic!("Tried to turn on a power domain not yet specified!");
            }
        }
    }

    pub fn is_enabled(domain: PowerDomain) -> bool {
        match domain {
            PowerDomain::Peripherals => (PRCM().pd_stat0_periph.get() & 1) >= 1,
            _ => false,
        }
    }
}

pub struct Clock (());

impl Clock {
    pub fn enable_gpio() {
        PRCM().gpio_clk_gate_run.set(1);
        PRCM().gpio_clk_gate_sleep.set(1);
        PRCM().gpio_clk_gate_deep_sleep.set(1);

        prcm_commit();
    }
}

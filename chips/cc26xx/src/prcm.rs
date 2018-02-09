/* PRCM - Power, Clock, and Reset Management */

use kernel::common::VolatileCell;

#[repr(C)]
struct PrcmRegisters {
    _reserved0: [VolatileCell<u8>; 0x28],

    // Write 1 in order to load settings
    pub clk_load_ctl: VolatileCell<u32>,

    _reserved1: [VolatileCell<u8>; 0x10],

    // TRNG, Crypto, and UDMA
    pub sec_dma_clk_run: VolatileCell<u32>,
    pub sec_dma_clk_sleep: VolatileCell<u32>,
    pub sec_dma_clk_deep_sleep: VolatileCell<u32>,

    pub gpio_clk_gate_run: VolatileCell<u32>,
    pub gpio_clk_gate_sleep: VolatileCell<u32>,
    pub gpio_clk_gate_deep_sleep: VolatileCell<u32>,

    _reserved3: [VolatileCell<u8>; 0x18],

    pub uart_clk_gate_run: VolatileCell<u32>,
    pub uart_clk_gate_sleep: VolatileCell<u32>,
    pub uart_clk_gate_deep_sleep: VolatileCell<u32>,

    _reserved4: [VolatileCell<u8>; 0xB4],

    // Power domain control 0
    pub pd_ctl0: VolatileCell<u32>,
    pub pd_ctl0_rfc: VolatileCell<u32>,
    pub pd_ctl0_serial: VolatileCell<u32>,
    pub pd_ctl0_peripheral: VolatileCell<u32>,

    _reserved5: [VolatileCell<u8>; 0x04],

    // Power domain status 0
    pub pd_stat0: VolatileCell<u32>,
    pub pd_stat0_rfc: VolatileCell<u32>,
    pub pd_stat0_serial: VolatileCell<u32>,
    pub pd_stat0_periph: VolatileCell<u32>,
}

const PRCM_BASE: *mut PrcmRegisters = 0x4008_2000 as *mut PrcmRegisters;

/*
    In order to save changes to the PRCM, we need to
    trigger
*/
fn prcm_commit() {
    let regs: &PrcmRegisters = unsafe { &*PRCM_BASE };
    regs.clk_load_ctl.set(1);
    // Wait for the settings to take effect
    while (regs.clk_load_ctl.get() & 0b10) == 0 {}
}

pub enum PowerDomain {
    // Note: when RFC is to be enabled, you are required to use both
    // power domains (i.e enable RFC on both PowerDomain0 and PowerDomain1)
    RFC,
    Serial,
    Peripherals,
    VIMS,
}

pub struct Power(());

impl Power {
    pub fn enable_domain(domain: PowerDomain) {
        let regs: &PrcmRegisters = unsafe { &*PRCM_BASE };

        match domain {
            PowerDomain::Peripherals => {
                regs.pd_ctl0.set(regs.pd_ctl0.get() | 0x4);
            },
            PowerDomain::Serial => {
                regs.pd_ctl0.set(regs.pd_ctl0.get() | 0x2);
            }
            _ => {
                panic!("Tried to turn on a power domain not yet specified!");
            }
        }
    }

    pub fn is_enabled(domain: PowerDomain) -> bool {
        let regs: &PrcmRegisters = unsafe { &*PRCM_BASE };
        match domain {
            PowerDomain::Peripherals => (regs.pd_stat0_periph.get() & 1) >= 1,
            PowerDomain::Serial => (regs.pd_stat0_serial.get() & 1) >= 1,
            _ => false,
        }
    }
}

pub struct Clock(());

impl Clock {
    pub fn enable_gpio() {
        let regs: &PrcmRegisters = unsafe { &*PRCM_BASE };
        regs.gpio_clk_gate_run.set(1);
        regs.gpio_clk_gate_sleep.set(1);
        regs.gpio_clk_gate_deep_sleep.set(1);

        prcm_commit();
    }
}

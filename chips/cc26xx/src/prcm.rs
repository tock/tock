//! Power, Clock, and Reset Management (PRCM)
//!
//! For details see p.411 in the cc2650 technical reference manual.
//!
//! PRCM manages different power domains on the boards, specifically:
//!
//!     * RF Power domain
//!     * Serial Power domain
//!     * Peripheral Power domain
//!
//! It also manages the clocks attached to almost every peripheral, which needs to
//! be enabled before usage.

use kernel::common::regs::{ReadOnly, ReadWrite, WriteOnly};

#[repr(C)]
struct PrcmRegisters {
    _reserved0: [ReadOnly<u8>; 0x28],

    // Write 1 in order to load settings
    pub clk_load_ctl: ReadWrite<u32, ClockLoad::Register>,

    _reserved1: [ReadOnly<u8>; 0x10],

    // TRNG, Crypto, and UDMA
    pub sec_dma_clk_run: ReadWrite<u32, SECDMAClockGate::Register>,
    pub sec_dma_clk_sleep: ReadWrite<u32, SECDMAClockGate::Register>,
    pub sec_dma_clk_deep_sleep: ReadWrite<u32, SECDMAClockGate::Register>,

    pub gpio_clk_gate_run: ReadWrite<u32, ClockGate::Register>,
    pub gpio_clk_gate_sleep: ReadWrite<u32, ClockGate::Register>,
    pub gpio_clk_gate_deep_sleep: ReadWrite<u32, ClockGate::Register>,

    _reserved3: [ReadOnly<u8>; 0x18],

    pub uart_clk_gate_run: ReadWrite<u32, ClockGate::Register>,
    pub uart_clk_gate_sleep: ReadWrite<u32, ClockGate::Register>,
    pub uart_clk_gate_deep_sleep: ReadWrite<u32, ClockGate::Register>,

    _reserved4: [ReadOnly<u8>; 0xB4],

    // Power domain control 0
    pub pd_ctl0: ReadWrite<u32, PowerDomain0::Register>,
    pub pd_ctl0_rfc: WriteOnly<u32, PowerDomainSingle::Register>,
    pub pd_ctl0_serial: WriteOnly<u32, PowerDomainSingle::Register>,
    pub pd_ctl0_peripheral: WriteOnly<u32, PowerDomainSingle::Register>,

    _reserved5: [ReadOnly<u8>; 0x04],

    // Power domain status 0
    pub pd_stat0: ReadOnly<u32, PowerDomainStatus0::Register>,
    pub pd_stat0_rfc: ReadOnly<u32, PowerDomainSingle::Register>,
    pub pd_stat0_serial: ReadOnly<u32, PowerDomainSingle::Register>,
    pub pd_stat0_periph: ReadOnly<u32, PowerDomainSingle::Register>,
}

register_bitfields![
    u32,
    ClockLoad [
        LOAD_DONE   OFFSET(1) NUMBITS(1) [],
        LOAD        OFFSET(0) NUMBITS(1) []
    ],
    SECDMAClockGate [
        DMA_CLK_EN      OFFSET(8) NUMBITS(1) [],
        TRNG_CLK_EN     OFFSET(1) NUMBITS(1) [],
        CRYPTO_CLK_EN   OFFSET(0) NUMBITS(1) []
    ],
    ClockGate [
        CLK_EN  OFFSET(0) NUMBITS(1) []
    ],
    PowerDomain0 [
        PERIPH_ON   OFFSET(2) NUMBITS(1) [],
        SERIAL_ON   OFFSET(1) NUMBITS(1) [],
        RFC_ON      OFFSET(0) NUMBITS(1) []
    ],
    PowerDomainSingle [
        ON  OFFSET(0) NUMBITS(1) []
    ],
    PowerDomainStatus0 [
        PERIPH_ON   OFFSET(2) NUMBITS(1) [],
        SERIAL_ON   OFFSET(1) NUMBITS(1) [],
        RFC_ON      OFFSET(0) NUMBITS(1) []
    ]
];

const PRCM_BASE: *mut PrcmRegisters = 0x4008_2000 as *mut PrcmRegisters;

/*
    In order to save changes to the PRCM, we need to
    trigger
*/
fn prcm_commit() {
    let regs: &PrcmRegisters = unsafe { &*PRCM_BASE };
    regs.clk_load_ctl.write(ClockLoad::LOAD::SET);
    // Wait for the settings to take effect
    while !regs.clk_load_ctl.is_set(ClockLoad::LOAD_DONE) {}
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
                regs.pd_ctl0.modify(PowerDomain0::PERIPH_ON::SET);
            }
            PowerDomain::Serial => {
                regs.pd_ctl0.modify(PowerDomain0::SERIAL_ON::SET);
            }
            _ => {
                panic!("Tried to turn on a power domain not yet specified!");
            }
        }
    }

    pub fn is_enabled(domain: PowerDomain) -> bool {
        let regs: &PrcmRegisters = unsafe { &*PRCM_BASE };
        match domain {
            PowerDomain::Peripherals => regs.pd_stat0_periph.is_set(PowerDomainSingle::ON),
            PowerDomain::Serial => regs.pd_stat0_serial.is_set(PowerDomainSingle::ON),
            _ => false,
        }
    }
}

pub struct Clock(());

impl Clock {
    pub fn enable_gpio() {
        let regs: &PrcmRegisters = unsafe { &*PRCM_BASE };
        regs.gpio_clk_gate_run.write(ClockGate::CLK_EN::SET);
        regs.gpio_clk_gate_sleep.write(ClockGate::CLK_EN::SET);
        regs.gpio_clk_gate_deep_sleep.write(ClockGate::CLK_EN::SET);

        prcm_commit();
    }

    pub fn enable_trng() {
        let regs: &PrcmRegisters = unsafe { &*PRCM_BASE };
        regs.sec_dma_clk_run
            .modify(SECDMAClockGate::TRNG_CLK_EN::SET);
        regs.sec_dma_clk_sleep
            .modify(SECDMAClockGate::TRNG_CLK_EN::SET);
        regs.sec_dma_clk_deep_sleep
            .modify(SECDMAClockGate::TRNG_CLK_EN::SET);

        prcm_commit();
    }

    /// Enables UART clocks for run, sleep and deep sleep mode.
    pub fn enable_uart() {
        let regs: &PrcmRegisters = unsafe { &*PRCM_BASE };
        regs.uart_clk_gate_run.modify(ClockGate::CLK_EN::SET);
        regs.uart_clk_gate_sleep.modify(ClockGate::CLK_EN::SET);
        regs.uart_clk_gate_deep_sleep.modify(ClockGate::CLK_EN::SET);

        prcm_commit();
    }
}

//! Power, Clock, and Reset Management (PRCM)
//!
//! For details see pg 451-566 of the cc13x2/cc26x2 Technical Reference Manual.
//!
//! PRCM manages different power domains on the boards, specifically:
//!
//!     * RF Power domain
//!     * Serial Power domain
//!     * Peripheral Power domain
//!
//! It also manages the clocks attached to almost every peripheral, which needs to
//! be enabled before usage.
//!
use kernel::common::registers::{ReadOnly, ReadWrite, WriteOnly};
use kernel::common::StaticRef;

#[repr(C)]
struct PrcmRegisters {
    // leaving INFRCLKDIVR/INFRCLKDIVRS/INFRCLKDIVD unimplemented for now
    _reserved0: [ReadOnly<u8>; 0x0C],

    // MCU Voltage Domain Control
    pub vd_ctl: ReadWrite<u32, VDControl::Register>,

    _reserved1: [ReadOnly<u8>; 0x18],

    // Write 1 in order to load prcm settings to CLKCTL power domain
    pub clk_load_ctl: ReadWrite<u32, ClockLoad::Register>,

    // RFC Clock Gate
    pub rfc_clk_gate: ReadWrite<u32, ClockGate::Register>,
    // VIMS Clock Gate
    pub vims_clk_gate: ReadWrite<u32, ClockGate::Register>,

    _reserved2: [ReadOnly<u8>; 0x8],

    // TRNG, Crypto, and UDMA
    pub sec_dma_clk_run: ReadWrite<u32, SECDMAClockGate::Register>,
    pub sec_dma_clk_sleep: ReadWrite<u32, SECDMAClockGate::Register>,
    pub sec_dma_clk_deep_sleep: ReadWrite<u32, SECDMAClockGate::Register>,

    // GPIO Clock Gate for run, sleep, and deep sleep modes
    pub gpio_clk_gate_run: ReadWrite<u32, ClockGate::Register>,
    pub gpio_clk_gate_sleep: ReadWrite<u32, ClockGate::Register>,
    pub gpio_clk_gate_deep_sleep: ReadWrite<u32, ClockGate::Register>,

    // GPT Clock Gate for run, sleep, and deep sleep modes
    pub gpt_clk_gate_run: ReadWrite<u32, ClockGate::Register>,
    pub gpt_clk_gate_sleep: ReadWrite<u32, ClockGate::Register>,
    pub gpt_clk_gate_deep_sleep: ReadWrite<u32, ClockGate::Register>,

    // I2C Clock Gate for run, sleep, and deep sleep modes
    pub i2c_clk_gate_run: ReadWrite<u32, ClockGate::Register>,
    pub i2c_clk_gate_sleep: ReadWrite<u32, ClockGate::Register>,
    pub i2c_clk_gate_deep_sleep: ReadWrite<u32, ClockGate::Register>,

    // UART Clock Gate for run, sleep, and deep sleep modes
    pub uart_clk_gate_run: ReadWrite<u32, ClockGate2::Register>,
    pub uart_clk_gate_sleep: ReadWrite<u32, ClockGate2::Register>,
    pub uart_clk_gate_deep_sleep: ReadWrite<u32, ClockGate2::Register>,

    // SSI Clock Gates for run, sleep, and deep sleep modes
    pub ssi_clk_gate_run: ReadWrite<u32, ClockGate::Register>,
    pub ssi_clk_gate_sleep: ReadWrite<u32, ClockGate::Register>,
    pub ssi_clk_gate_deep_sleep: ReadWrite<u32, ClockGate::Register>,

    // I2S Clock Gates for run, sleep, and deep sleep modes
    pub i2s_clk_gate_run: ReadWrite<u32, ClockGate::Register>,
    pub i2s_clk_gate_sleep: ReadWrite<u32, ClockGate::Register>,
    pub i2s_clk_gate_deep_sleep: ReadWrite<u32, ClockGate::Register>, // 0x8Ch offset

    //_reserved4: [ReadOnly<u8>; 0xB4],
    _reserved4: [ReadOnly<u8>; 0x9C],

    // Power Domain Control 0
    pub pd_ctl0: ReadWrite<u32, PowerDomain0::Register>, // 0x12Ch offset
    pub pd_ctl0_rfc: WriteOnly<u32, PowerDomainSingle::Register>,
    pub pd_ctl0_serial: WriteOnly<u32, PowerDomainSingle::Register>,
    pub pd_ctl0_peripheral: WriteOnly<u32, PowerDomainSingle::Register>,

    _reserved5: [ReadOnly<u8>; 0x04],

    // Power Domain Status 0
    pub pd_stat0: ReadOnly<u32, PowerDomainStatus0::Register>,
    pub pd_stat0_rfc: ReadOnly<u32, PowerDomainSingle::Register>,
    pub pd_stat0_serial: ReadOnly<u32, PowerDomainSingle::Register>,
    pub pd_stat0_periph: ReadOnly<u32, PowerDomainSingle::Register>,

    _reserved6: [ReadOnly<u8>; 0x2C],

    // Power Domain Control 1
    pub pd_ctl1: ReadWrite<u32, PowerDomain1::Register>,

    _reserved7: [ReadOnly<u8>; 0x04],

    pub pd_ctl1_cpu: WriteOnly<u32, PowerDomainSingle::Register>,
    pub pd_ctl1_rfc: WriteOnly<u32, PowerDomainSingle::Register>,
    pub pd_ctl1_vims: WriteOnly<u32, PowerDomainSingle::Register>,

    _reserved8: [ReadOnly<u8>; 0x04],

    // Power Domain Status 1
    pub pd_stat1: ReadOnly<u32, PowerDomainStatus1::Register>,
    pub pd_stat1_bus: ReadOnly<u32, PowerDomainSingle::Register>,
    pub pd_stat1_rfc: ReadOnly<u32, PowerDomainSingle::Register>,
    pub pd_stat1_cpu: ReadOnly<u32, PowerDomainSingle::Register>,
    pub pd_stat1_vims: ReadOnly<u32, PowerDomainSingle::Register>,

    _reserved9: [ReadOnly<u8>; 0x24],

    _rfc_bits: ReadOnly<u32, AutoControl::Register>, // CPE auto check at boot for immediate start up tasks

    // RF
    pub rfc_mode_sel: ReadWrite<u32>,
    pub rfc_mode_allowed: ReadOnly<u32>,

    _reserved10: [ReadOnly<u8>; 0xB8],
    // Enable/Disable interupt generation when OSC is qualified
    pub osc_imsc: ReadWrite<u32, OscInterrupt::Register>,
    // OSC raw interrupt status
    pub osc_ris: ReadOnly<u32, OscInterrupt::Register>,
    // OSC raw interupt clear
    pub osc_icr: WriteOnly<u32, OscInterrupt::Register>,
}

register_bitfields![
    u32,
    VDControl [
        // SPARE1 (bits 1-31)
        ULDO              OFFSET(0) NUMBITS(1) []
    ],
    ClockLoad [
        // RESERVED (bits 2-31)
        LOAD_DONE   OFFSET(1) NUMBITS(1) [],
        LOAD        OFFSET(0) NUMBITS(1) []
    ],
    SECDMAClockGate [
        //RESERVED (bits 25-31)
        // Force Clock Enable for Crypto, TRNG, DMA, PKA not implemented here (bits 16, 17, 18 , 24
        // respectively)
        DMA_CLK_EN      OFFSET(8) NUMBITS(1) [],
        // RESERVED (bits 3-7)
        // PKA Clock not implemented (bit 2)
        TRNG_CLK_EN     OFFSET(1) NUMBITS(1) [],
        CRYPTO_CLK_EN   OFFSET(0) NUMBITS(1) []
    ],
    ClockGate [
        AM_EN       OFFSET(8) NUMBITS(1) [],
        // RESERVED (bits 2-8, 10-31)
        CLK_EN      OFFSET(0) NUMBITS(1) []
    ],
    ClockGate2 [
        // RESERVED (bits 1-31)
        AM_EN OFFSET(8) NUMBITS(2) [
            Set0 = 0b01,
            Set1 = 0b10,
            SetAll = 0b11,
            ClearAll = 0b00
        ],
        CLK_EN          OFFSET(0) NUMBITS(2) [
            Set0 = 0b1,
            Set1 = 0b10,
            SetAll = 0b11,
            ClearAll = 0b0
        ]
    ],
    PowerDomain0 [
        // RESERVED (bits 3-31)
        PERIPH_ON   OFFSET(2) NUMBITS(1) [],
        SERIAL_ON   OFFSET(1) NUMBITS(1) [],
        RFC_ON      OFFSET(0) NUMBITS(1) []
    ],
    PowerDomainSingle [
        // RESERVED (bits 1-31)
        ON          OFFSET(0) NUMBITS(1) []
    ],
    PowerDomainStatus0 [
        // RESERVED (bits 3-31)
        PERIPH_ON   OFFSET(2) NUMBITS(1) [],
        SERIAL_ON   OFFSET(1) NUMBITS(1) [],
        RFC_ON      OFFSET(0) NUMBITS(1) []
    ],
    PowerDomain1 [
        // RESERVED (bits 5-31)
        VIMS_ON   OFFSET(3) NUMBITS(2) [],
        RFC_ON   OFFSET(2) NUMBITS(1) [],
        CPU_ON      OFFSET(1) NUMBITS(1) []
    ],
    PowerDomainStatus1 [
        // RESERVED (bits 0 & 5-31)
        BUS_ON      OFFSET(4) NUMBITS(1) [],
        VIMS_ON     OFFSET(3) NUMBITS(1) [],
        RFC_ON      OFFSET(2) NUMBITS(1) [],
        CPU_ON      OFFSET(1) NUMBITS(1) []
    ],
    AutoControl [
        Startup_Prefs OFFSET(0) NUMBITS(32) []
    ],
    OscInterrupt [
        HF_SRC OFFSET(7) NUMBITS(1) [],
        LF_SRC OFFSET(6) NUMBITS(1) [],
        XOSC_DLF OFFSET(5) NUMBITS(1) [],
        XOSC_LF OFFSET(4) NUMBITS(1) [],
        RCOSC_DLF OFFSET(3) NUMBITS(1) [],
        RCOSC_LF OFFSET(2) NUMBITS(1) [],
        XOSC_HF OFFSET(1) NUMBITS(1) [],
        RCOSC_HF OFFSET(0) NUMBITS(1) []
    ]
];

const PRCM_BASE: StaticRef<PrcmRegisters> =
    unsafe { StaticRef::new(0x4008_2000 as *mut PrcmRegisters) };

// In order to save changes to the PRCM, we need to
// trigger the load register

fn prcm_commit() {
    let regs = PRCM_BASE;
    regs.clk_load_ctl.write(ClockLoad::LOAD::SET);
    // Wait for the settings to take effect
    while !regs.clk_load_ctl.is_set(ClockLoad::LOAD_DONE) {}
}

pub fn force_disable_dma_and_crypto() {
    let regs = PRCM_BASE;
    if regs.sec_dma_clk_run.is_set(SECDMAClockGate::DMA_CLK_EN) {
        regs.sec_dma_clk_deep_sleep
            .modify(SECDMAClockGate::DMA_CLK_EN::CLEAR);
    }
    if regs.sec_dma_clk_run.is_set(SECDMAClockGate::CRYPTO_CLK_EN) {
        regs.sec_dma_clk_deep_sleep
            .modify(SECDMAClockGate::CRYPTO_CLK_EN::CLEAR);
    }

    prcm_commit();
}

/// The ULDO power source is a temporary power source
/// which could be enable to drive Peripherals in deep sleep.
pub fn acquire_uldo() {
    let regs = PRCM_BASE;
    regs.vd_ctl.modify(VDControl::ULDO::SET);
}

/// It is no use to enable the ULDO power source constantly,
/// and it would need to be released once we go out of deep sleep
pub fn release_uldo() {
    let regs = PRCM_BASE;
    regs.vd_ctl.modify(VDControl::ULDO::CLEAR);
}

pub enum PowerDomain {
    // Note: when RFC is to be enabled, you are required to use both
    // power domains (i.e enable RFC on both PowerDomain0 and PowerDomain1)
    RFC,
    Serial,
    Peripherals,
    VIMS,
    CPU,
}

impl From<u32> for PowerDomain {
    fn from(n: u32) -> Self {
        match n {
            0 => PowerDomain::RFC,
            1 => PowerDomain::Serial,
            2 => PowerDomain::Peripherals,
            3 => PowerDomain::VIMS,
            4 => PowerDomain::CPU,
            _ => unimplemented!(),
        }
    }
}

#[allow(non_camel_case_types)]
pub enum OscInt {
    HF_SRC,
    LF_SRC,
    XOSC_DLF,
    XOSC_LF,
    RCOSC_DLF,
    RCOSC_LF,
    XOSC_HF,
    RCOSC_HF,
}

pub struct Power(());

impl Power {
    pub fn enable_domain(domain: PowerDomain) {
        let regs = PRCM_BASE;

        match domain {
            PowerDomain::Peripherals => {
                regs.pd_ctl0.modify(PowerDomain0::PERIPH_ON::SET);
                while !Power::is_enabled(PowerDomain::Peripherals) {}
            }
            PowerDomain::Serial => {
                regs.pd_ctl0.modify(PowerDomain0::SERIAL_ON::SET);
                while !Power::is_enabled(PowerDomain::Serial) {}
            }
            PowerDomain::RFC => {
                regs.pd_ctl0.modify(PowerDomain0::RFC_ON::SET);
                regs.pd_ctl1.modify(PowerDomain1::RFC_ON::SET);
                while !Power::is_enabled(PowerDomain::RFC) {}
            }
            PowerDomain::CPU => {
                regs.pd_ctl1.modify(PowerDomain1::CPU_ON::SET);
                while !Power::is_enabled(PowerDomain::CPU) {}
            }
            PowerDomain::VIMS => {
                regs.pd_ctl1.modify(PowerDomain1::VIMS_ON.val(0x02));
                while !Power::is_enabled(PowerDomain::VIMS) {}
            }
        }
    }

    pub fn disable_domain(domain: PowerDomain) {
        let regs = PRCM_BASE;

        match domain {
            PowerDomain::Peripherals => {
                regs.pd_ctl0.modify(PowerDomain0::PERIPH_ON::CLEAR);
            }
            PowerDomain::Serial => {
                regs.pd_ctl0.modify(PowerDomain0::SERIAL_ON::CLEAR);
            }
            PowerDomain::RFC => {
                regs.pd_ctl0.modify(PowerDomain0::RFC_ON::CLEAR);
                regs.pd_ctl1.modify(PowerDomain1::RFC_ON::CLEAR);
            }
            PowerDomain::CPU => {
                regs.pd_ctl1.modify(PowerDomain1::CPU_ON::CLEAR);
            }
            PowerDomain::VIMS => {
                regs.pd_ctl1.modify(PowerDomain1::VIMS_ON::CLEAR);
            }
        }
    }

    pub fn is_enabled(domain: PowerDomain) -> bool {
        let regs = PRCM_BASE;
        match domain {
            PowerDomain::Peripherals => regs.pd_stat0_periph.is_set(PowerDomainSingle::ON),
            PowerDomain::Serial => regs.pd_stat0_serial.is_set(PowerDomainSingle::ON),
            PowerDomain::RFC => {
                regs.pd_stat1.is_set(PowerDomainStatus1::RFC_ON)
                    && regs.pd_stat0.is_set(PowerDomainStatus0::RFC_ON)
            }
            PowerDomain::VIMS => regs.pd_stat1.is_set(PowerDomainStatus1::VIMS_ON),
            PowerDomain::CPU => regs.pd_stat1.is_set(PowerDomainStatus1::CPU_ON),
        }
    }
}

pub struct Clock(());

impl Clock {
    pub fn enable_gpio() {
        let regs = PRCM_BASE;
        regs.gpio_clk_gate_run.modify(ClockGate::AM_EN::SET);
        regs.gpio_clk_gate_run.modify(ClockGate::CLK_EN::SET);
        regs.gpio_clk_gate_sleep.modify(ClockGate::CLK_EN::SET);
        regs.gpio_clk_gate_deep_sleep.modify(ClockGate::CLK_EN::SET);

        prcm_commit();
    }

    pub fn enable_trng() {
        let regs = PRCM_BASE;
        regs.sec_dma_clk_run
            .modify(SECDMAClockGate::TRNG_CLK_EN::SET);
        /*
        regs.sec_dma_clk_sleep
            .modify(SECDMAClockGate::TRNG_CLK_EN::SET);
        regs.sec_dma_clk_deep_sleep
            .modify(SECDMAClockGate::TRNG_CLK_EN::SET);
*/
        prcm_commit();
    }

    /// Enables UART clocks for run, sleep and deep sleep mode.
    pub fn enable_uart() {
        let regs = PRCM_BASE;
        regs.uart_clk_gate_run.modify(ClockGate2::AM_EN::SetAll);
        regs.uart_clk_gate_run.modify(ClockGate2::CLK_EN::SetAll);
        regs.uart_clk_gate_sleep.modify(ClockGate2::CLK_EN::SetAll);
        regs.uart_clk_gate_deep_sleep
            .modify(ClockGate2::CLK_EN::SetAll);

        prcm_commit();
    }

    pub fn disable_uart() {
        let regs = PRCM_BASE;
        regs.uart_clk_gate_run.modify(ClockGate2::AM_EN::ClearAll);
        regs.uart_clk_gate_run.modify(ClockGate2::CLK_EN::ClearAll);
        regs.uart_clk_gate_sleep
            .modify(ClockGate2::CLK_EN::ClearAll);
        regs.uart_clk_gate_deep_sleep
            .modify(ClockGate2::CLK_EN::ClearAll);

        prcm_commit();
    }

    pub fn enable_rfc() {
        let regs = PRCM_BASE;
        regs.rfc_clk_gate.modify(ClockGate::CLK_EN::SET);

        prcm_commit();
    }

    pub fn disable_rfc() {
        let regs = PRCM_BASE;
        regs.rfc_clk_gate.modify(ClockGate::CLK_EN::CLEAR);

        prcm_commit();
    }

    pub fn enable_vims() {
        let regs = PRCM_BASE;
        regs.vims_clk_gate.modify(ClockGate::CLK_EN::SET);

        prcm_commit();
    }

    pub fn disable_vims() {
        let regs = PRCM_BASE;
        regs.vims_clk_gate.modify(ClockGate::CLK_EN::SET);

        prcm_commit();
    }

    pub fn enable_gpt() {
        let regs = PRCM_BASE;
        regs.gpt_clk_gate_run.modify(ClockGate::CLK_EN::SET);
        regs.gpt_clk_gate_sleep.modify(ClockGate::CLK_EN::SET);
        regs.gpt_clk_gate_deep_sleep.modify(ClockGate::CLK_EN::SET);

        prcm_commit();
    }

    pub fn disable_gpt() {
        let regs = PRCM_BASE;
        regs.gpt_clk_gate_run.modify(ClockGate::CLK_EN::CLEAR);
        regs.gpt_clk_gate_sleep.modify(ClockGate::CLK_EN::CLEAR);
        regs.gpt_clk_gate_deep_sleep
            .modify(ClockGate::CLK_EN::CLEAR);

        prcm_commit();
    }

    /// Enables I2C clocks for run, sleep and deep sleep mode.
    pub fn enable_i2c() {
        let regs = PRCM_BASE;
        regs.i2c_clk_gate_run.modify(ClockGate::CLK_EN::SET);
        regs.i2c_clk_gate_sleep.modify(ClockGate::CLK_EN::SET);
        regs.i2c_clk_gate_deep_sleep.modify(ClockGate::CLK_EN::SET);

        prcm_commit();
    }

    pub fn enable_i2s() {
        let regs = PRCM_BASE;
        regs.i2s_clk_gate_run
            .modify(ClockGate::AM_EN::SET + ClockGate::CLK_EN::SET);
        regs.i2s_clk_gate_sleep.modify(ClockGate::CLK_EN::SET);
        regs.i2s_clk_gate_deep_sleep.modify(ClockGate::CLK_EN::SET);

        prcm_commit();
    }

    pub fn disable_i2s() {
        let regs = PRCM_BASE;
        regs.i2s_clk_gate_run
            .modify(ClockGate::AM_EN::CLEAR + ClockGate::CLK_EN::CLEAR);
        regs.i2s_clk_gate_sleep.modify(ClockGate::CLK_EN::CLEAR);
        regs.i2s_clk_gate_deep_sleep
            .modify(ClockGate::CLK_EN::CLEAR);

        prcm_commit();
    }
}

pub fn rf_mode_sel(mode: u32) {
    let regs = PRCM_BASE;
    regs.rfc_mode_sel.set(mode);
}

pub fn disable_osc_interrupt() {
    let regs = PRCM_BASE;
    regs.osc_imsc.write(
        OscInterrupt::HF_SRC::CLEAR
            + OscInterrupt::LF_SRC::CLEAR
            + OscInterrupt::RCOSC_DLF::CLEAR
            + OscInterrupt::RCOSC_LF::CLEAR
            + OscInterrupt::RCOSC_HF::CLEAR
            + OscInterrupt::XOSC_DLF::CLEAR
            + OscInterrupt::XOSC_HF::CLEAR
            + OscInterrupt::XOSC_LF::CLEAR,
    );
}

pub fn enable_osc_interrupt(osc: OscInt) {
    let regs = PRCM_BASE;
    match osc {
        OscInt::HF_SRC => regs.osc_imsc.modify(OscInterrupt::HF_SRC::SET),
        OscInt::LF_SRC => regs.osc_imsc.modify(OscInterrupt::LF_SRC::SET),
        OscInt::RCOSC_DLF => regs.osc_imsc.modify(OscInterrupt::RCOSC_DLF::SET),
        OscInt::RCOSC_LF => regs.osc_imsc.modify(OscInterrupt::RCOSC_LF::SET),
        OscInt::RCOSC_HF => regs.osc_imsc.modify(OscInterrupt::RCOSC_HF::SET),
        OscInt::XOSC_DLF => regs.osc_imsc.modify(OscInterrupt::XOSC_DLF::SET),
        OscInt::XOSC_HF => regs.osc_imsc.modify(OscInterrupt::XOSC_HF::SET),
        OscInt::XOSC_LF => regs.osc_imsc.modify(OscInterrupt::XOSC_LF::SET),
    }
}

pub fn handle_osc_interrupt() {
    let regs = PRCM_BASE;

    if regs.osc_ris.is_set(OscInterrupt::HF_SRC) {
        regs.osc_icr.write(OscInterrupt::HF_SRC::SET);
    }
    if regs.osc_ris.is_set(OscInterrupt::LF_SRC) {
        regs.osc_icr.write(OscInterrupt::LF_SRC::SET);
    }
    if regs.osc_ris.is_set(OscInterrupt::RCOSC_DLF) {
        regs.osc_icr.write(OscInterrupt::RCOSC_DLF::SET);
    }
    if regs.osc_ris.is_set(OscInterrupt::RCOSC_LF) {
        regs.osc_icr.write(OscInterrupt::RCOSC_LF::SET);
    }
    if regs.osc_ris.is_set(OscInterrupt::RCOSC_HF) {
        regs.osc_icr.write(OscInterrupt::RCOSC_HF::SET);
    }
    if regs.osc_ris.is_set(OscInterrupt::XOSC_HF) {
        regs.osc_icr.write(OscInterrupt::XOSC_HF::SET);
    }
    if regs.osc_ris.is_set(OscInterrupt::XOSC_LF) {
        regs.osc_icr.write(OscInterrupt::XOSC_LF::SET);
    }
    if regs.osc_ris.is_set(OscInterrupt::XOSC_DLF) {
        regs.osc_icr.write(OscInterrupt::XOSC_DLF::SET);
    }
}

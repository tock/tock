//! Implementation of the Freescale MK20 interrupt controller

use core::intrinsics;
use kernel::common::VolatileCell;

// TODO: This register format is common to all Cortex-M cores, so I think it
// should be moved to the cortexm crate
#[repr(C, packed)]
struct Nvic {
    iser: [VolatileCell<u32>; 7],
    _reserved0: [u32; 25],
    icer: [VolatileCell<u32>; 7],
    _reserved1: [u32; 25],
    ispr: [VolatileCell<u32>; 7],
    _reserved2: [u32; 25],
    icpr: [VolatileCell<u32>; 7],
}

#[repr(C)]
#[derive(Copy,Clone)]
#[allow(non_camel_case_types)]
pub enum NvicIdx {
    DMA0,
    DMA1,
    DMA2,
    DMA3,
    DMA4,
    DMA5,
    DMA6,
    DMA7,
    DMA8,
    DMA9,
    DMA10,
    DMA11,
    DMA12,
    DMA13,
    DMA14,
    DMA15,
    DMAERR,
    MCM,
    FLASHCC,
    FLASHRC,
    MODECTRL,
    LLWU,
    WDOG,
    RNG,
    I2C0,
    I2C1,
    SPI0,
    SPI1,
    I2S0_TX,
    I2S0_RX,
    _RESERVED0,
    UART0,
    UART0_ERR,
    UART1,
    UART1_ERR,
    UART2,
    UART2_ERR,
    UART3,
    UART3_ERR,
    ADC0,
    CMP0,
    CMP1,
    FTM0,
    FTM1,
    FTM2,
    CMT,
    RTC_ALARM,
    RTC_SECONDS,
    PIT0,
    PIT1,
    PIT2,
    PIT3,
    PDB,
    USBFS_OTG,
    USBFS_CHARGE,
    _RESERVED1,
    DAC0,
    MCG,
    LOWPOWERTIMER,
    PCMA,
    PCMB,
    PCMC,
    PCMD,
    PCME,
    SOFTWARE,
    SPI2,
    UART4,
    UART4_ERR,
    _RESERVED2,
    _RESERVED3,
    CMP2,
    FTM3,
    DAC1,
    ADC1,
    I2C2,
    CAN0_MSGBUF,
    CAN0_BUSOFF,
    CAN0_ERR,
    CAN0_TX,
    CAN0_RX,
    CAN0_WKUP,
    SDHC,
    EMAC_TIMER,
    EMAC_TX,
    EMAC_RX,
    EMAC_ERR,
    LPUART0,
    TSI0,
    TPM1,
    TPM2,
    USBHS,
    I2C3,
    CMP3,
    USBHS_OTG,
    CAN1_MSBBUF,
    CAN1_BUSOFF,
    CAN1_ERR,
    CAN1_TX,
    CAN1_RX,
    CAN1_WKUP,
}

impl ::core::default::Default for NvicIdx {
    fn default() -> NvicIdx {
        NvicIdx::DMA0
    }
}

// Defined by ARM Cortex-M bus architecture
// TODO: since these functions/constants are common to all ARM Cortex-M cores, I
// think they should be moved to the cortexm crate.
const BASE_ADDRESS: usize = 0xe000e100;

pub unsafe fn enable(signal: NvicIdx) {
    let nvic: &mut Nvic = intrinsics::transmute(BASE_ADDRESS);
    let interrupt = signal as usize;

    nvic.iser[interrupt / 32].set(1 << (interrupt & 31));
}

pub unsafe fn disable(signal: NvicIdx) {
    let nvic: &mut Nvic = intrinsics::transmute(BASE_ADDRESS);
    let interrupt = signal as usize;

    nvic.icer[interrupt / 32].set(1 << (interrupt & 31));
}

pub unsafe fn clear_pending(signal: NvicIdx) {
    let nvic: &mut Nvic = intrinsics::transmute(BASE_ADDRESS);
    let interrupt = signal as usize;

    nvic.icpr[interrupt / 32].set(1 << (interrupt & 31));
}

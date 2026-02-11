// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2025.

use core::cell::Cell;
use kernel::utilities::registers::{
    interfaces::{ReadWriteable, Readable, Writeable},
    register_bitfields, register_structs, ReadOnly, ReadWrite,
};
use kernel::utilities::StaticRef;

register_structs! {
    GpioClockRegisters {
        /// Clock control, can be changed on-the-fly (except for auxsrc)
        (0x000 => ctrl: ReadWrite<u32, CLK_GPOUTx_CTRL::Register>),
        /// Clock divisor, can be changed on-the-fly
        (0x004 => div: ReadWrite<u32, CLK_GPOUTx_DIV::Register>),
        /// Indicates which src is currently selected (one-hot)
        (0x008 => selected: ReadOnly<u32, CLK_GPOUTx_SELECTED::Register>),
        /// Clock control, can be changed on-the-fly (except for auxsrc)
        (0x00C => @END),
    },
    ClocksRegisters {
        (0x000 => clk_gpio: [GpioClockRegisters; 4]),
        /// Clock control, can be changed on-the-fly (except for auxsrc)
        (0x030 => clk_ref_ctrl: ReadWrite<u32, CLK_REF_CTRL::Register>),

        (0x034 => clk_ref_div: ReadWrite<u32>),
        /// Indicates which src is currently selected (one-hot)
        (0x038 => clk_ref_selected: ReadWrite<u32, CLK_REF_SELECTED::Register>),
        /// Clock control, can be changed on-the-fly (except for auxsrc)
        (0x03C => clk_sys_ctrl: ReadWrite<u32, CLK_SYS_CTRL::Register>),

        (0x040 => clk_sys_div: ReadWrite<u32, CLK_SYS_DIV::Register>),
        /// Indicates which src is currently selected (one-hot)
        (0x044 => clk_sys_selected: ReadWrite<u32, CLK_SYS_SELECTED::Register>),
        /// Clock control, can be changed on-the-fly (except for auxsrc)
        (0x048 => clk_peri_ctrl: ReadWrite<u32, CLK_PERI_CTRL::Register>),

        (0x04C => clk_peri_div: ReadWrite<u32>),
        /// Indicates which src is currently selected (one-hot)
        (0x050 => clk_peri_selected: ReadWrite<u32>),
        /// Clock control, can be changed on-the-fly (except for auxsrc)
        (0x054 => clk_hstx_ctrl: ReadWrite<u32, CLK_HSTX_CTRL::Register>),

        (0x058 => clk_hstx_div: ReadWrite<u32>),
        /// Indicates which src is currently selected (one-hot)
        (0x05C => clk_hstx_selected: ReadWrite<u32>),
        /// Clock control, can be changed on-the-fly (except for auxsrc)
        (0x060 => clk_usb_ctrl: ReadWrite<u32, CLK_USB_CTRL::Register>),

        (0x064 => clk_usb_div: ReadWrite<u32>),
        /// Indicates which src is currently selected (one-hot)
        (0x068 => clk_usb_selected: ReadWrite<u32>),
        /// Clock control, can be changed on-the-fly (except for auxsrc)
        (0x06C => clk_adc_ctrl: ReadWrite<u32, CLK_ADC_CTRL::Register>),

        (0x070 => clk_adc_div: ReadWrite<u32>),
        /// Indicates which src is currently selected (one-hot)
        (0x074 => clk_adc_selected: ReadWrite<u32>),

        (0x078 => dftclk_xosc_ctrl: ReadWrite<u32>),

        (0x07C => dftclk_rosc_ctrl: ReadWrite<u32>),

        (0x080 => dftclk_lposc_ctrl: ReadWrite<u32>),

        (0x084 => clk_sys_resus_ctrl: ReadWrite<u32, CLK_SYS_RESUS_CTRL::Register>),

        (0x088 => clk_sys_resus_status: ReadWrite<u32>),
        /// Reference clock frequency in kHz
        (0x08C => fc0_ref_khz: ReadWrite<u32>),
        /// Minimum pass frequency in kHz. This is optional. Set to 0 if you are not using t
        (0x090 => fc0_min_khz: ReadWrite<u32>),
        /// Maximum pass frequency in kHz. This is optional. Set to 0x1ffffff if you are not
        (0x094 => fc0_max_khz: ReadWrite<u32>),
        /// Delays the start of frequency counting to allow the mux to settle
        /// Delay is measured in multiples of the reference clock period
        (0x098 => fc0_delay: ReadWrite<u32>),
        /// The test interval is 0.98us * 2**interval, but let's call it 1us * 2**interval
        /// The default gives a test interval of 250us
        (0x09C => fc0_interval: ReadWrite<u32>),
        /// Clock sent to frequency counter, set to 0 when not required
        /// Writing to this register initiates the frequency count
        (0x0A0 => fc0_src: ReadWrite<u32>),
        /// Frequency counter status
        (0x0A4 => fc0_status: ReadWrite<u32, FC0_STATUS::Register>),
        /// Result of frequency measurement, only valid when status_done=1
        (0x0A8 => fc0_result: ReadWrite<u32, FC0_RESULT::Register>),
        /// enable clock in wake mode
        (0x0AC => wake_en0: ReadWrite<u32, WAKE_EN0::Register>),
        /// enable clock in wake mode
        (0x0B0 => wake_en1: ReadWrite<u32, WAKE_EN1::Register>),
        /// enable clock in sleep mode
        (0x0B4 => sleep_en0: ReadWrite<u32, SLEEP_EN0::Register>),
        /// enable clock in sleep mode
        (0x0B8 => sleep_en1: ReadWrite<u32, SLEEP_EN1::Register>),
        /// indicates the state of the clock enable
        (0x0BC => enabled0: ReadWrite<u32, ENABLED0::Register>),
        /// indicates the state of the clock enable
        (0x0C0 => enabled1: ReadWrite<u32, ENABLED1::Register>),
        /// Raw Interrupts
        (0x0C4 => intr: ReadWrite<u32>),
        /// Interrupt Enable
        (0x0C8 => inte: ReadWrite<u32>),
        /// Interrupt Force
        (0x0CC => intf: ReadWrite<u32>),
        /// Interrupt status after masking & forcing
        (0x0D0 => ints: ReadWrite<u32>),
        (0x0D4 => @END),
    },
     PllRegisters {
        /// Control and Status
        /// GENERAL CONSTRAINTS:
        /// Reference clock frequency min=5MHz, max=800MHz
        /// Feedback divider min=16, max=320
        /// VCO frequency min=750MHz, max=1600MHz
        (0x000 => cs: ReadWrite<u32, CS::Register>),
        /// Controls the PLL power modes.
        (0x004 => pwr: ReadWrite<u32, PWR::Register>),
        /// Feedback divisor
        /// (note: this PLL does not support fractional division)
        (0x008 => fbdiv_int: ReadWrite<u32, FBDIV_INT::Register>),
        /// Controls the PLL post dividers for the primary output
        /// (note: this PLL does not have a secondary output)
        /// the primary output is driven from VCO divided by postdiv1*po
        (0x00C => prim: ReadWrite<u32, PRIM::Register>),
        /// Raw Interrupts
        (0x010 => intr: ReadWrite<u32>),
        /// Interrupt Enable
        (0x014 => inte: ReadWrite<u32>),
        /// Interrupt Force
        (0x018 => intf: ReadWrite<u32>),
        /// Interrupt status after masking & forcing
        (0x01C => ints: ReadWrite<u32>),
        (0x020 => @END),
    }
}
register_bitfields![u32,
CLK_GPOUTx_CTRL [
    /// clock generator is enabled
    ENABLED OFFSET(28) NUMBITS(1) [],
    /// An edge on this signal shifts the phase of the output by 1 cycle of the input cl
        /// This can be done at any time
    NUDGE OFFSET(20) NUMBITS(1) [],
    /// This delays the enable signal by up to 3 cycles of the input clock
    /// This must be set before the clock is enabled to have
    PHASE OFFSET(16) NUMBITS(2) [],
    /// Enables duty cycle correction for odd divisors, can be changed on-the-fly
    DC50 OFFSET(12) NUMBITS(1) [],
    /// Starts and stops the clock generator cleanly
    ENABLE OFFSET(11) NUMBITS(1) [],
    /// Asynchronously kills the clock generator, enable must be set low before deassert
    KILL OFFSET(10) NUMBITS(1) [],
    /// Selects the auxiliary clock source, will glitch when switching
    AUXSRC OFFSET(5) NUMBITS(4) [

        Clksrc_pll_sys = 0
    ]
],
CLK_GPOUTx_DIV [
    /// Integer part of clock divisor, 0 -> max+1, can be changed on-the-fly
    INT OFFSET(16) NUMBITS(16) [],
    /// Fractional component of the divisor, can be changed on-the-fly
    FRAC OFFSET(0) NUMBITS(16) []
],
CLK_GPOUTx_SELECTED [
    /// This slice does not have a glitchless mux (only the AUX_SRC field is present, no
    CLK_GPOUT0_SELECTED OFFSET(0) NUMBITS(1) []
],
CLK_REF_CTRL [
    /// Selects the auxiliary clock source, will glitch when switching
    AUXSRC OFFSET(5) NUMBITS(2) [

        Clksrc_pll_usb = 0
    ],
    /// Selects the clock source glitchlessly, can be changed on-the-fly
    SRC OFFSET(0) NUMBITS(2) [

        Rosc_clksrc_ph = 0
    ]
],
CLK_REF_DIV [
    /// Integer part of clock divisor, 0 -> max+1, can be changed on-the-fly
    INT OFFSET(16) NUMBITS(8) []
],
CLK_REF_SELECTED [
    /// The glitchless multiplexer does not switch instantaneously (to avoid glitches),
    CLK_REF_SELECTED OFFSET(0) NUMBITS(4) []
],
CLK_SYS_CTRL [
    /// Selects the auxiliary clock source, will glitch when switching
    AUXSRC OFFSET(5) NUMBITS(3) [

        Clksrc_pll_sys = 0
    ],
    /// Selects the clock source glitchlessly, can be changed on-the-fly
    SRC OFFSET(0) NUMBITS(1) [

        Clk_ref = 0
    ]
],
CLK_SYS_DIV [
    /// Integer part of clock divisor, 0 -> max+1, can be changed on-the-fly
    INT OFFSET(16) NUMBITS(16) [],
    /// Fractional component of the divisor, can be changed on-the-fly
    FRAC OFFSET(0) NUMBITS(16) []
],
CLK_SYS_SELECTED [
    /// The glitchless multiplexer does not switch instantaneously (to avoid glitches),
    CLK_SYS_SELECTED OFFSET(0) NUMBITS(2) []
],
CLK_PERI_CTRL [
    /// clock generator is enabled
    ENABLED OFFSET(28) NUMBITS(1) [],
    /// Starts and stops the clock generator cleanly
    ENABLE OFFSET(11) NUMBITS(1) [],
    /// Asynchronously kills the clock generator, enable must be set low before deassert
    KILL OFFSET(10) NUMBITS(1) [],
    /// Selects the auxiliary clock source, will glitch when switching
    AUXSRC OFFSET(5) NUMBITS(3) [

        Clk_sys = 0
    ]
],
CLK_PERI_DIV [
    /// Integer part of clock divisor, 0 -> max+1, can be changed on-the-fly
    INT OFFSET(16) NUMBITS(2) []
],
CLK_PERI_SELECTED [
    /// This slice does not have a glitchless mux (only the AUX_SRC field is present, no
    CLK_PERI_SELECTED OFFSET(0) NUMBITS(1) []
],
CLK_HSTX_CTRL [
    /// clock generator is enabled
    ENABLED OFFSET(28) NUMBITS(1) [],
    /// An edge on this signal shifts the phase of the output by 1 cycle of the input cl
    /// This can be done at any time
    NUDGE OFFSET(20) NUMBITS(1) [],
    /// This delays the enable signal by up to 3 cycles of the input clock
    /// This must be set before the clock is enabled to have
    PHASE OFFSET(16) NUMBITS(2) [],
    /// Starts and stops the clock generator cleanly
    ENABLE OFFSET(11) NUMBITS(1) [],
    /// Asynchronously kills the clock generator, enable must be set low before deassert
    KILL OFFSET(10) NUMBITS(1) [],
    /// Selects the auxiliary clock source, will glitch when switching
    AUXSRC OFFSET(5) NUMBITS(3) [

        Clk_sys = 0
    ]
],
CLK_HSTX_DIV [
    /// Integer part of clock divisor, 0 -> max+1, can be changed on-the-fly
    INT OFFSET(16) NUMBITS(2) []
],
CLK_HSTX_SELECTED [
    /// This slice does not have a glitchless mux (only the AUX_SRC field is present, no
    CLK_HSTX_SELECTED OFFSET(0) NUMBITS(1) []
],
CLK_USB_CTRL [
    /// clock generator is enabled
    ENABLED OFFSET(28) NUMBITS(1) [],
    /// An edge on this signal shifts the phase of the output by 1 cycle of the input cl
    /// This can be done at any time
    NUDGE OFFSET(20) NUMBITS(1) [],
    /// This delays the enable signal by up to 3 cycles of the input clock
    /// This must be set before the clock is enabled to have
    PHASE OFFSET(16) NUMBITS(2) [],
    /// Starts and stops the clock generator cleanly
    ENABLE OFFSET(11) NUMBITS(1) [],
    /// Asynchronously kills the clock generator, enable must be set low before deassert
    KILL OFFSET(10) NUMBITS(1) [],
    /// Selects the auxiliary clock source, will glitch when switching
    AUXSRC OFFSET(5) NUMBITS(3) [

        Clksrc_pll_usb = 0
    ]
],
CLK_USB_DIV [
    /// Integer part of clock divisor, 0 -> max+1, can be changed on-the-fly
    INT OFFSET(16) NUMBITS(4) []
],
CLK_USB_SELECTED [
    /// This slice does not have a glitchless mux (only the AUX_SRC field is present, no
    CLK_USB_SELECTED OFFSET(0) NUMBITS(1) []
],
CLK_ADC_CTRL [
    /// clock generator is enabled
    ENABLED OFFSET(28) NUMBITS(1) [],
    /// An edge on this signal shifts the phase of the output by 1 cycle of the input cl
    /// This can be done at any time
    NUDGE OFFSET(20) NUMBITS(1) [],
    /// This delays the enable signal by up to 3 cycles of the input clock
    /// This must be set before the clock is enabled to have
    PHASE OFFSET(16) NUMBITS(2) [],
    /// Starts and stops the clock generator cleanly
    ENABLE OFFSET(11) NUMBITS(1) [],
    /// Asynchronously kills the clock generator, enable must be set low before deassert
    KILL OFFSET(10) NUMBITS(1) [],
    /// Selects the auxiliary clock source, will glitch when switching
    AUXSRC OFFSET(5) NUMBITS(3) [

        Clksrc_pll_usb = 0
    ]
],
CLK_ADC_DIV [
    /// Integer part of clock divisor, 0 -> max+1, can be changed on-the-fly
    INT OFFSET(16) NUMBITS(4) []
],
CLK_ADC_SELECTED [
    /// This slice does not have a glitchless mux (only the AUX_SRC field is present, no
    CLK_ADC_SELECTED OFFSET(0) NUMBITS(1) []
],
DFTCLK_XOSC_CTRL [

    SRC OFFSET(0) NUMBITS(2) [

        NULL = 0
    ]
],
DFTCLK_ROSC_CTRL [

    SRC OFFSET(0) NUMBITS(2) [

        NULL = 0
    ]
],
DFTCLK_LPOSC_CTRL [

    SRC OFFSET(0) NUMBITS(2) [

        NULL = 0
    ]
],
CLK_SYS_RESUS_CTRL [
    /// For clearing the resus after the fault that triggered it has been corrected
    CLEAR OFFSET(16) NUMBITS(1) [],
    /// Force a resus, for test purposes only
    FRCE OFFSET(12) NUMBITS(1) [],
    /// Enable resus
    ENABLE OFFSET(8) NUMBITS(1) [],
    /// This is expressed as a number of clk_ref cycles
    /// and must be >= 2x clk_ref_freq/min_clk_tst_freq
    TIMEOUT OFFSET(0) NUMBITS(8) []
],
CLK_SYS_RESUS_STATUS [
    /// Clock has been resuscitated, correct the error then send ctrl_clear=1
    RESUSSED OFFSET(0) NUMBITS(1) []
],
FC0_REF_KHZ [

    FC0_REF_KHZ OFFSET(0) NUMBITS(20) []
],
FC0_MIN_KHZ [

    FC0_MIN_KHZ OFFSET(0) NUMBITS(25) []
],
FC0_MAX_KHZ [

    FC0_MAX_KHZ OFFSET(0) NUMBITS(25) []
],
FC0_DELAY [

    FC0_DELAY OFFSET(0) NUMBITS(3) []
],
FC0_INTERVAL [

    FC0_INTERVAL OFFSET(0) NUMBITS(4) []
],
FC0_SRC [

    FC0_SRC OFFSET(0) NUMBITS(8) [

        NULL = 0
    ]
],
FC0_STATUS [
    /// Test clock stopped during test
    DIED OFFSET(28) NUMBITS(1) [],
    /// Test clock faster than expected, only valid when status_done=1
    FAST OFFSET(24) NUMBITS(1) [],
    /// Test clock slower than expected, only valid when status_done=1
    SLOW OFFSET(20) NUMBITS(1) [],
    /// Test failed
    FAIL OFFSET(16) NUMBITS(1) [],
    /// Waiting for test clock to start
    WAITING OFFSET(12) NUMBITS(1) [],
    /// Test running
    RUNNING OFFSET(8) NUMBITS(1) [],
    /// Test complete
    DONE OFFSET(4) NUMBITS(1) [],
    /// Test passed
    PASS OFFSET(0) NUMBITS(1) []
],
FC0_RESULT [

    KHZ OFFSET(5) NUMBITS(25) [],

    FRAC OFFSET(0) NUMBITS(5) []
],
WAKE_EN0 [

    CLK_SYS_SIO OFFSET(31) NUMBITS(1) [],

    CLK_SYS_SHA256 OFFSET(30) NUMBITS(1) [],

    CLK_SYS_PSM OFFSET(29) NUMBITS(1) [],

    CLK_SYS_ROSC OFFSET(28) NUMBITS(1) [],

    CLK_SYS_ROM OFFSET(27) NUMBITS(1) [],

    CLK_SYS_RESETS OFFSET(26) NUMBITS(1) [],

    CLK_SYS_PWM OFFSET(25) NUMBITS(1) [],

    CLK_SYS_POWMAN OFFSET(24) NUMBITS(1) [],

    CLK_REF_POWMAN OFFSET(23) NUMBITS(1) [],

    CLK_SYS_PLL_USB OFFSET(22) NUMBITS(1) [],

    CLK_SYS_PLL_SYS OFFSET(21) NUMBITS(1) [],

    CLK_SYS_PIO2 OFFSET(20) NUMBITS(1) [],

    CLK_SYS_PIO1 OFFSET(19) NUMBITS(1) [],

    CLK_SYS_PIO0 OFFSET(18) NUMBITS(1) [],

    CLK_SYS_PADS OFFSET(17) NUMBITS(1) [],

    CLK_SYS_OTP OFFSET(16) NUMBITS(1) [],

    CLK_REF_OTP OFFSET(15) NUMBITS(1) [],

    CLK_SYS_JTAG OFFSET(14) NUMBITS(1) [],

    CLK_SYS_IO OFFSET(13) NUMBITS(1) [],

    CLK_SYS_I2C1 OFFSET(12) NUMBITS(1) [],

    CLK_SYS_I2C0 OFFSET(11) NUMBITS(1) [],

    CLK_SYS_HSTX OFFSET(10) NUMBITS(1) [],

    CLK_HSTX OFFSET(9) NUMBITS(1) [],

    CLK_SYS_GLITCH_DETECTOR OFFSET(8) NUMBITS(1) [],

    CLK_SYS_DMA OFFSET(7) NUMBITS(1) [],

    CLK_SYS_BUSFABRIC OFFSET(6) NUMBITS(1) [],

    CLK_SYS_BUSCTRL OFFSET(5) NUMBITS(1) [],

    CLK_SYS_BOOTRAM OFFSET(4) NUMBITS(1) [],

    CLK_SYS_ADC OFFSET(3) NUMBITS(1) [],

    CLK_ADC OFFSET(2) NUMBITS(1) [],

    CLK_SYS_ACCESSCTRL OFFSET(1) NUMBITS(1) [],

    CLK_SYS_CLOCKS OFFSET(0) NUMBITS(1) []
],
WAKE_EN1 [

    CLK_SYS_XOSC OFFSET(30) NUMBITS(1) [],

    CLK_SYS_XIP OFFSET(29) NUMBITS(1) [],

    CLK_SYS_WATCHDOG OFFSET(28) NUMBITS(1) [],

    CLK_USB OFFSET(27) NUMBITS(1) [],

    CLK_SYS_USBCTRL OFFSET(26) NUMBITS(1) [],

    CLK_SYS_UART1 OFFSET(25) NUMBITS(1) [],

    CLK_PERI_UART1 OFFSET(24) NUMBITS(1) [],

    CLK_SYS_UART0 OFFSET(23) NUMBITS(1) [],

    CLK_PERI_UART0 OFFSET(22) NUMBITS(1) [],

    CLK_SYS_TRNG OFFSET(21) NUMBITS(1) [],

    CLK_SYS_TIMER1 OFFSET(20) NUMBITS(1) [],

    CLK_SYS_TIMER0 OFFSET(19) NUMBITS(1) [],

    CLK_SYS_TICKS OFFSET(18) NUMBITS(1) [],

    CLK_REF_TICKS OFFSET(17) NUMBITS(1) [],

    CLK_SYS_TBMAN OFFSET(16) NUMBITS(1) [],

    CLK_SYS_SYSINFO OFFSET(15) NUMBITS(1) [],

    CLK_SYS_SYSCFG OFFSET(14) NUMBITS(1) [],

    CLK_SYS_SRAM9 OFFSET(13) NUMBITS(1) [],

    CLK_SYS_SRAM8 OFFSET(12) NUMBITS(1) [],

    CLK_SYS_SRAM7 OFFSET(11) NUMBITS(1) [],

    CLK_SYS_SRAM6 OFFSET(10) NUMBITS(1) [],

    CLK_SYS_SRAM5 OFFSET(9) NUMBITS(1) [],

    CLK_SYS_SRAM4 OFFSET(8) NUMBITS(1) [],

    CLK_SYS_SRAM3 OFFSET(7) NUMBITS(1) [],

    CLK_SYS_SRAM2 OFFSET(6) NUMBITS(1) [],

    CLK_SYS_SRAM1 OFFSET(5) NUMBITS(1) [],

    CLK_SYS_SRAM0 OFFSET(4) NUMBITS(1) [],

    CLK_SYS_SPI1 OFFSET(3) NUMBITS(1) [],

    CLK_PERI_SPI1 OFFSET(2) NUMBITS(1) [],

    CLK_SYS_SPI0 OFFSET(1) NUMBITS(1) [],

    CLK_PERI_SPI0 OFFSET(0) NUMBITS(1) []
],
SLEEP_EN0 [

    CLK_SYS_SIO OFFSET(31) NUMBITS(1) [],

    CLK_SYS_SHA256 OFFSET(30) NUMBITS(1) [],

    CLK_SYS_PSM OFFSET(29) NUMBITS(1) [],

    CLK_SYS_ROSC OFFSET(28) NUMBITS(1) [],

    CLK_SYS_ROM OFFSET(27) NUMBITS(1) [],

    CLK_SYS_RESETS OFFSET(26) NUMBITS(1) [],

    CLK_SYS_PWM OFFSET(25) NUMBITS(1) [],

    CLK_SYS_POWMAN OFFSET(24) NUMBITS(1) [],

    CLK_REF_POWMAN OFFSET(23) NUMBITS(1) [],

    CLK_SYS_PLL_USB OFFSET(22) NUMBITS(1) [],

    CLK_SYS_PLL_SYS OFFSET(21) NUMBITS(1) [],

    CLK_SYS_PIO2 OFFSET(20) NUMBITS(1) [],

    CLK_SYS_PIO1 OFFSET(19) NUMBITS(1) [],

    CLK_SYS_PIO0 OFFSET(18) NUMBITS(1) [],

    CLK_SYS_PADS OFFSET(17) NUMBITS(1) [],

    CLK_SYS_OTP OFFSET(16) NUMBITS(1) [],

    CLK_REF_OTP OFFSET(15) NUMBITS(1) [],

    CLK_SYS_JTAG OFFSET(14) NUMBITS(1) [],

    CLK_SYS_IO OFFSET(13) NUMBITS(1) [],

    CLK_SYS_I2C1 OFFSET(12) NUMBITS(1) [],

    CLK_SYS_I2C0 OFFSET(11) NUMBITS(1) [],

    CLK_SYS_HSTX OFFSET(10) NUMBITS(1) [],

    CLK_HSTX OFFSET(9) NUMBITS(1) [],

    CLK_SYS_GLITCH_DETECTOR OFFSET(8) NUMBITS(1) [],

    CLK_SYS_DMA OFFSET(7) NUMBITS(1) [],

    CLK_SYS_BUSFABRIC OFFSET(6) NUMBITS(1) [],

    CLK_SYS_BUSCTRL OFFSET(5) NUMBITS(1) [],

    CLK_SYS_BOOTRAM OFFSET(4) NUMBITS(1) [],

    CLK_SYS_ADC OFFSET(3) NUMBITS(1) [],

    CLK_ADC OFFSET(2) NUMBITS(1) [],

    CLK_SYS_ACCESSCTRL OFFSET(1) NUMBITS(1) [],

    CLK_SYS_CLOCKS OFFSET(0) NUMBITS(1) []
],
SLEEP_EN1 [

    CLK_SYS_XOSC OFFSET(30) NUMBITS(1) [],

    CLK_SYS_XIP OFFSET(29) NUMBITS(1) [],

    CLK_SYS_WATCHDOG OFFSET(28) NUMBITS(1) [],

    CLK_USB OFFSET(27) NUMBITS(1) [],

    CLK_SYS_USBCTRL OFFSET(26) NUMBITS(1) [],

    CLK_SYS_UART1 OFFSET(25) NUMBITS(1) [],

    CLK_PERI_UART1 OFFSET(24) NUMBITS(1) [],

    CLK_SYS_UART0 OFFSET(23) NUMBITS(1) [],

    CLK_PERI_UART0 OFFSET(22) NUMBITS(1) [],

    CLK_SYS_TRNG OFFSET(21) NUMBITS(1) [],

    CLK_SYS_TIMER1 OFFSET(20) NUMBITS(1) [],

    CLK_SYS_TIMER0 OFFSET(19) NUMBITS(1) [],

    CLK_SYS_TICKS OFFSET(18) NUMBITS(1) [],

    CLK_REF_TICKS OFFSET(17) NUMBITS(1) [],

    CLK_SYS_TBMAN OFFSET(16) NUMBITS(1) [],

    CLK_SYS_SYSINFO OFFSET(15) NUMBITS(1) [],

    CLK_SYS_SYSCFG OFFSET(14) NUMBITS(1) [],

    CLK_SYS_SRAM9 OFFSET(13) NUMBITS(1) [],

    CLK_SYS_SRAM8 OFFSET(12) NUMBITS(1) [],

    CLK_SYS_SRAM7 OFFSET(11) NUMBITS(1) [],

    CLK_SYS_SRAM6 OFFSET(10) NUMBITS(1) [],

    CLK_SYS_SRAM5 OFFSET(9) NUMBITS(1) [],

    CLK_SYS_SRAM4 OFFSET(8) NUMBITS(1) [],

    CLK_SYS_SRAM3 OFFSET(7) NUMBITS(1) [],

    CLK_SYS_SRAM2 OFFSET(6) NUMBITS(1) [],

    CLK_SYS_SRAM1 OFFSET(5) NUMBITS(1) [],

    CLK_SYS_SRAM0 OFFSET(4) NUMBITS(1) [],

    CLK_SYS_SPI1 OFFSET(3) NUMBITS(1) [],

    CLK_PERI_SPI1 OFFSET(2) NUMBITS(1) [],

    CLK_SYS_SPI0 OFFSET(1) NUMBITS(1) [],

    CLK_PERI_SPI0 OFFSET(0) NUMBITS(1) []
],
ENABLED0 [

    CLK_SYS_SIO OFFSET(31) NUMBITS(1) [],

    CLK_SYS_SHA256 OFFSET(30) NUMBITS(1) [],

    CLK_SYS_PSM OFFSET(29) NUMBITS(1) [],

    CLK_SYS_ROSC OFFSET(28) NUMBITS(1) [],

    CLK_SYS_ROM OFFSET(27) NUMBITS(1) [],

    CLK_SYS_RESETS OFFSET(26) NUMBITS(1) [],

    CLK_SYS_PWM OFFSET(25) NUMBITS(1) [],

    CLK_SYS_POWMAN OFFSET(24) NUMBITS(1) [],

    CLK_REF_POWMAN OFFSET(23) NUMBITS(1) [],

    CLK_SYS_PLL_USB OFFSET(22) NUMBITS(1) [],

    CLK_SYS_PLL_SYS OFFSET(21) NUMBITS(1) [],

    CLK_SYS_PIO2 OFFSET(20) NUMBITS(1) [],

    CLK_SYS_PIO1 OFFSET(19) NUMBITS(1) [],

    CLK_SYS_PIO0 OFFSET(18) NUMBITS(1) [],

    CLK_SYS_PADS OFFSET(17) NUMBITS(1) [],

    CLK_SYS_OTP OFFSET(16) NUMBITS(1) [],

    CLK_REF_OTP OFFSET(15) NUMBITS(1) [],

    CLK_SYS_JTAG OFFSET(14) NUMBITS(1) [],

    CLK_SYS_IO OFFSET(13) NUMBITS(1) [],

    CLK_SYS_I2C1 OFFSET(12) NUMBITS(1) [],

    CLK_SYS_I2C0 OFFSET(11) NUMBITS(1) [],

    CLK_SYS_HSTX OFFSET(10) NUMBITS(1) [],

    CLK_HSTX OFFSET(9) NUMBITS(1) [],

    CLK_SYS_GLITCH_DETECTOR OFFSET(8) NUMBITS(1) [],

    CLK_SYS_DMA OFFSET(7) NUMBITS(1) [],

    CLK_SYS_BUSFABRIC OFFSET(6) NUMBITS(1) [],

    CLK_SYS_BUSCTRL OFFSET(5) NUMBITS(1) [],

    CLK_SYS_BOOTRAM OFFSET(4) NUMBITS(1) [],

    CLK_SYS_ADC OFFSET(3) NUMBITS(1) [],

    CLK_ADC OFFSET(2) NUMBITS(1) [],

    CLK_SYS_ACCESSCTRL OFFSET(1) NUMBITS(1) [],

    CLK_SYS_CLOCKS OFFSET(0) NUMBITS(1) []
],
ENABLED1 [

    CLK_SYS_XOSC OFFSET(30) NUMBITS(1) [],

    CLK_SYS_XIP OFFSET(29) NUMBITS(1) [],

    CLK_SYS_WATCHDOG OFFSET(28) NUMBITS(1) [],

    CLK_USB OFFSET(27) NUMBITS(1) [],

    CLK_SYS_USBCTRL OFFSET(26) NUMBITS(1) [],

    CLK_SYS_UART1 OFFSET(25) NUMBITS(1) [],

    CLK_PERI_UART1 OFFSET(24) NUMBITS(1) [],

    CLK_SYS_UART0 OFFSET(23) NUMBITS(1) [],

    CLK_PERI_UART0 OFFSET(22) NUMBITS(1) [],

    CLK_SYS_TRNG OFFSET(21) NUMBITS(1) [],

    CLK_SYS_TIMER1 OFFSET(20) NUMBITS(1) [],

    CLK_SYS_TIMER0 OFFSET(19) NUMBITS(1) [],

    CLK_SYS_TICKS OFFSET(18) NUMBITS(1) [],

    CLK_REF_TICKS OFFSET(17) NUMBITS(1) [],

    CLK_SYS_TBMAN OFFSET(16) NUMBITS(1) [],

    CLK_SYS_SYSINFO OFFSET(15) NUMBITS(1) [],

    CLK_SYS_SYSCFG OFFSET(14) NUMBITS(1) [],

    CLK_SYS_SRAM9 OFFSET(13) NUMBITS(1) [],

    CLK_SYS_SRAM8 OFFSET(12) NUMBITS(1) [],

    CLK_SYS_SRAM7 OFFSET(11) NUMBITS(1) [],

    CLK_SYS_SRAM6 OFFSET(10) NUMBITS(1) [],

    CLK_SYS_SRAM5 OFFSET(9) NUMBITS(1) [],

    CLK_SYS_SRAM4 OFFSET(8) NUMBITS(1) [],

    CLK_SYS_SRAM3 OFFSET(7) NUMBITS(1) [],

    CLK_SYS_SRAM2 OFFSET(6) NUMBITS(1) [],

    CLK_SYS_SRAM1 OFFSET(5) NUMBITS(1) [],

    CLK_SYS_SRAM0 OFFSET(4) NUMBITS(1) [],

    CLK_SYS_SPI1 OFFSET(3) NUMBITS(1) [],

    CLK_PERI_SPI1 OFFSET(2) NUMBITS(1) [],

    CLK_SYS_SPI0 OFFSET(1) NUMBITS(1) [],

    CLK_PERI_SPI0 OFFSET(0) NUMBITS(1) []
],
INTR [

    CLK_SYS_RESUS OFFSET(0) NUMBITS(1) []
],
INTE [

    CLK_SYS_RESUS OFFSET(0) NUMBITS(1) []
],
INTF [

    CLK_SYS_RESUS OFFSET(0) NUMBITS(1) []
],
INTS [

    CLK_SYS_RESUS OFFSET(0) NUMBITS(1) []
],
CS [
    /// PLL is locked
    LOCK OFFSET(31) NUMBITS(1) [],
    /// PLL is not locked
    /// Ideally this is cleared when PLL lock is seen and th
    LOCK_N OFFSET(30) NUMBITS(1) [],
    /// Passes the reference clock to the output instead of the divided VCO. The VCO con
    BYPASS OFFSET(8) NUMBITS(1) [],
    /// Divides the PLL input reference clock.
    /// Behaviour is undefined for div=0.
    /// PLL output will be unpredictable during refdiv chang
    REFDIV OFFSET(0) NUMBITS(6) []
],
PWR [
    /// PLL VCO powerdown
    /// To save power set high when PLL output not required
    VCOPD OFFSET(5) NUMBITS(1) [],
    /// PLL post divider powerdown
    /// To save power set high when PLL output not required
    POSTDIVPD OFFSET(3) NUMBITS(1) [],
    /// PLL DSM powerdown
    /// Nothing is achieved by setting this low.
    DSMPD OFFSET(2) NUMBITS(1) [],
    /// PLL powerdown
    /// To save power set high when PLL output not required.
    PD OFFSET(0) NUMBITS(1) []
],
FBDIV_INT [
    /// see ctrl reg description for constraints
    FBDIV_INT OFFSET(0) NUMBITS(12) []
],
PRIM [
    /// divide by 1-7
    POSTDIV1 OFFSET(16) NUMBITS(3) [],
    /// divide by 1-7
    POSTDIV2 OFFSET(12) NUMBITS(3) []
],
];

#[derive(Copy, Clone, PartialEq, Debug)]
#[repr(usize)]
pub enum Clock {
    GpioOut0 = 0,
    GpioOut1 = 1,
    GpioOut2 = 2,
    GpioOut3 = 3,
    Reference = 4,
    System = 5,
    Peripheral = 6,
    Hstx = 7,
    Usb = 8,
    Adc = 9,
}

const CLOCKS_BASE: StaticRef<ClocksRegisters> =
    unsafe { StaticRef::new(0x40010000 as *const ClocksRegisters) };
const PLL_SYS_BASE: StaticRef<PllRegisters> =
    unsafe { StaticRef::new(0x40050000 as *const PllRegisters) };
const PLL_USB_BASE: StaticRef<PllRegisters> =
    unsafe { StaticRef::new(0x40058000 as *const PllRegisters) };

pub enum PllClock {
    Sys = 0,
    Usb = 1,
}

#[derive(Copy, Clone, PartialEq, Debug)]
#[repr(u8)]
pub enum GpioAuxiliaryClockSource {
    PllSys = 0,
    Gpio0 = 1,
    Gpio1 = 2,
    PllUsb = 3,
    Rsoc = 4,
    Xosc = 5,
    Sys = 6,
    Usb = 7,
    Adc = 8,
    Rtc = 9,
    Ref = 10,
}

#[derive(Copy, Clone, PartialEq, Debug)]
#[repr(u8)]
pub enum ReferenceClockSource {
    Rsoc = 0,
    Auxiliary = 1,
    Xosc = 2,
}

#[derive(Copy, Clone, PartialEq, Debug)]
#[repr(u8)]
pub enum ReferenceAuxiliaryClockSource {
    PllUsb = 0,
    Gpio0 = 1,
    Gpio1 = 2,
}

#[derive(Copy, Clone, PartialEq, Debug)]
#[repr(u8)]
pub enum SystemClockSource {
    Reference = 0,
    Auxiliary = 1,
}

#[derive(Copy, Clone, PartialEq, Debug)]
#[repr(u8)]
pub enum SystemAuxiliaryClockSource {
    PllSys = 0,
    PllUsb = 1,
    Rsoc = 2,
    Xsoc = 3,
    Gpio0 = 4,
    Gpio1 = 5,
}

#[derive(Copy, Clone, PartialEq, Debug)]
#[repr(u8)]
pub enum PeripheralAuxiliaryClockSource {
    System = 0,
    PllSys = 1,
    PllUsb = 2,
    Rsoc = 3,
    Xsoc = 4,
    Gpio0 = 5,
    Gpio1 = 6,
}

#[derive(Copy, Clone, PartialEq, Debug)]
#[repr(u8)]
pub enum UsbAuxiliaryClockSource {
    PllSys = 0,
    PllUsb = 1,
    Rsoc = 2,
    Xsoc = 3,
    Gpio0 = 4,
    Gpio1 = 5,
}

#[derive(Copy, Clone, PartialEq, Debug)]
#[repr(u8)]
pub enum AdcAuxiliaryClockSource {
    PllSys = 0,
    PllUsb = 1,
    Rsoc = 2,
    Xsoc = 3,
    Gpio0 = 4,
    Gpio1 = 5,
}

#[derive(Copy, Clone, PartialEq, Debug)]
#[repr(u8)]
pub enum HstxAuxiliaryClockSource {
    PllSys = 0,
    PllUsb = 1,
    Rsoc = 2,
    Xsoc = 3,
    Gpio0 = 4,
    Gpio1 = 5,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum ClockSource {
    GpioOut,
    Reference(ReferenceClockSource),
    System(SystemClockSource),
    Peripheral,
    Usb,
    Adc,
    Rtc,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum ClockAuxiliarySource {
    GpioOut(GpioAuxiliaryClockSource),
    Reference(ReferenceAuxiliaryClockSource),
    System(SystemAuxiliaryClockSource),
    Peripheral(PeripheralAuxiliaryClockSource),
    Usb(UsbAuxiliaryClockSource),
    Adc(AdcAuxiliaryClockSource),
    Hstx(HstxAuxiliaryClockSource),
}

const NUM_CLOCKS: usize = 10;

pub struct Clocks {
    registers: StaticRef<ClocksRegisters>,
    pll_registers: &'static [StaticRef<PllRegisters>],
    frequencies: [Cell<u32>; NUM_CLOCKS],
}

impl Clocks {
    pub const fn new() -> Self {
        Self {
            registers: CLOCKS_BASE,
            pll_registers: &[PLL_SYS_BASE, PLL_USB_BASE],
            frequencies: [
                Cell::new(0),
                Cell::new(0),
                Cell::new(0),
                Cell::new(0),
                Cell::new(0),
                Cell::new(0),
                Cell::new(0),
                Cell::new(0),
                Cell::new(0),
                Cell::new(0),
            ],
        }
    }

    pub fn enable_resus(&self) {
        self.registers
            .clk_sys_resus_ctrl
            .modify(CLK_SYS_RESUS_CTRL::ENABLE::SET);
    }

    pub fn disable_resus(&self) {
        self.registers
            .clk_sys_resus_ctrl
            .modify(CLK_SYS_RESUS_CTRL::ENABLE::CLEAR);
    }

    pub fn disable_sys_aux(&self) {
        self.registers
            .clk_sys_ctrl
            .modify(CLK_SYS_CTRL::SRC::Clk_ref);
        while self
            .registers
            .clk_sys_selected
            .read(CLK_SYS_SELECTED::CLK_SYS_SELECTED)
            != 0x1
        {}
    }

    pub fn disable_ref_aux(&self) {
        self.registers
            .clk_ref_ctrl
            .modify(CLK_REF_CTRL::SRC::Rosc_clksrc_ph);
        while self
            .registers
            .clk_ref_selected
            .read(CLK_REF_SELECTED::CLK_REF_SELECTED)
            != 0x1
        {}
    }

    pub fn pll_init(
        &self,
        clock: PllClock,
        xosc_freq: u32,
        refdiv: u32,
        vco_freq: u32,
        post_div1: u32,
        post_div2: u32,
    ) {
        let registers = self.pll_registers[clock as usize];

        // Turn off PLL
        registers
            .pwr
            .modify(PWR::PD::SET + PWR::DSMPD::SET + PWR::POSTDIVPD::SET + PWR::VCOPD::SET);
        registers.fbdiv_int.modify(FBDIV_INT::FBDIV_INT.val(0));

        let ref_mhz = xosc_freq / refdiv;
        registers.cs.modify(CS::REFDIV.val(refdiv));

        // Calculate feedback divider
        let fbdiv = vco_freq / (ref_mhz * 1000000);

        // Should we use assert instead of if and panic! ?
        if fbdiv < 16 || fbdiv > 320 {
            panic!("Invalid feedback divider number {} not in [16, 320]", fbdiv)
        }

        if post_div1 < 1 || post_div1 > 7 || post_div2 < 1 || post_div2 > 7 {
            panic!(
                "Invalid post_div number {} or {} not in [1, 7]",
                post_div1, post_div2
            );
        }

        if post_div2 > post_div1 {
            panic!(
                "post_div2 must be less than post_div1 ({} >= {})",
                post_div1, post_div2
            );
        }

        if ref_mhz > vco_freq / 16 {
            panic!(
                "ref_mhz must be less than vco_freq / 16 ({} <= {})",
                ref_mhz,
                vco_freq / 16
            );
        }

        // Set feedback divider
        registers.fbdiv_int.modify(FBDIV_INT::FBDIV_INT.val(fbdiv));

        // Turn on PLL
        registers.pwr.modify(PWR::PD::CLEAR + PWR::VCOPD::CLEAR);

        // Wait for PLL to lock
        while !registers.cs.is_set(CS::LOCK) {}

        // Set up post divider
        registers
            .prim
            .modify(PRIM::POSTDIV1.val(post_div1) + PRIM::POSTDIV2.val(post_div2));

        // Turn on post divider
        registers.pwr.modify(PWR::POSTDIVPD::CLEAR);
    }

    pub fn pll_deinit(&self, clock: PllClock) {
        self.pll_registers[clock as usize]
            .pwr
            .modify(PWR::PD::SET + PWR::DSMPD::SET + PWR::POSTDIVPD::SET + PWR::VCOPD::SET);
    }

    pub fn set_frequency(&self, clock: Clock, freq: u32) {
        self.frequencies[clock as usize].set(freq);
    }

    pub fn get_frequency(&self, clock: Clock) -> u32 {
        self.frequencies[clock as usize].get()
    }

    fn set_divider(&self, clock: Clock, div: u32) {
        match clock {
            Clock::GpioOut0 | Clock::GpioOut1 | Clock::GpioOut2 | Clock::GpioOut3 => {
                self.registers.clk_gpio[clock as usize].div.set(div)
            }
            Clock::System => self.registers.clk_sys_div.set(div),
            Clock::Reference => self.registers.clk_ref_div.set(div),
            Clock::Usb => self.registers.clk_usb_div.set(div),
            Clock::Adc => self.registers.clk_adc_div.set(div),
            Clock::Hstx => self.registers.clk_hstx_div.set(div),
            // Clock::Reference
            _ => panic!("failed to set div"),
        }
    }

    fn get_divider(&self, source_freq: u32, freq: u32) -> u32 {
        // pico-sdk: Div register is 24.8 int.frac divider so multiply by 2^8 (left shift by 8)
        (((source_freq as u64) << 16) / freq as u64) as u32
    }

    #[cfg(any(doc, all(target_arch = "arm", target_os = "none")))]
    #[inline]
    fn loop_3_cycles(&self, clock: Clock) {
        if self.get_frequency(clock) > 0 {
            let _delay_cyc: u32 = self.get_frequency(Clock::System) / self.get_frequency(clock) + 1;
            unsafe {
                use core::arch::asm;
                asm! (
                    "1:",
                    "subs {0}, #1",
                    "bne 1b",
                    in (reg) _delay_cyc
                );
            }
        }
    }

    #[cfg(not(any(doc, all(target_arch = "arm", target_os = "none"))))]
    fn loop_3_cycles(&self, _clock: Clock) {
        unimplemented!()
    }

    pub fn configure_gpio_out(
        &self,
        clock: Clock,
        auxiliary_source: GpioAuxiliaryClockSource,
        source_freq: u32,
        freq: u32,
    ) {
        match clock {
            Clock::GpioOut0 | Clock::GpioOut1 | Clock::GpioOut2 | Clock::GpioOut3 => {
                if freq > source_freq {
                    panic!(
                        "freq is greater than source freq ({} > {})",
                        freq, source_freq
                    );
                }

                let div = self.get_divider(source_freq, freq);

                // pico-sdk:
                // If increasing divisor, set divisor before source. Otherwise set source
                // before divisor. This avoids a momentary overspeed when e.g. switching
                // to a faster source and increasing divisor to compensate.
                if div > self.registers.clk_gpio[clock as usize].div.get() {
                    self.set_divider(clock, div);
                }

                self.registers.clk_gpio[clock as usize]
                    .ctrl
                    .modify(CLK_GPOUTx_CTRL::ENABLE::CLEAR);
                // pico-sdk:
                // Delay for 3 cycles of the target clock, for ENABLE propagation.
                // Note XOSC_COUNT is not helpful here because XOSC is not
                // necessarily running, nor is timer... so, 3 cycles per loop:
                self.loop_3_cycles(clock);

                self.registers.clk_gpio[clock as usize]
                    .ctrl
                    .modify(CLK_GPOUTx_CTRL::AUXSRC.val(auxiliary_source as u32));

                self.registers.clk_gpio[clock as usize]
                    .ctrl
                    .modify(CLK_GPOUTx_CTRL::ENABLE::SET);

                // pico-sdk:
                // Now that the source is configured, we can trust that the user-supplied
                // divisor is a safe value.
                self.set_divider(clock, div);

                self.set_frequency(clock, freq);
            }
            _ => panic!("trying to set a non gpio clock"),
        }
    }

    pub fn configure_system(
        &self,
        source: SystemClockSource,
        auxiliary_source: SystemAuxiliaryClockSource,
        source_freq: u32,
        freq: u32,
    ) {
        if freq > source_freq {
            panic!(
                "freq is greater than source freq ({} > {})",
                freq, source_freq
            );
        }
        let div = self.get_divider(source_freq, freq);

        // pico-sdk:
        // If increasing divisor, set divisor before source. Otherwise set source
        // before divisor. This avoids a momentary overspeed when e.g. switching
        // to a faster source and increasing divisor to compensate.
        if div > self.registers.clk_sys_div.get() {
            self.set_divider(Clock::System, div);
        }

        // pico-sdk:
        // If switching a glitchless slice (ref or sys) to an aux source, switch
        // away from aux *first* to avoid passing glitches when changing aux mux.
        // Assume (!!!) glitchless source 0 is no faster than the aux source.
        if source == SystemClockSource::Auxiliary {
            self.registers
                .clk_sys_ctrl
                .modify(CLK_SYS_CTRL::SRC::Clk_ref);
            while self
                .registers
                .clk_sys_selected
                .read(CLK_SYS_SELECTED::CLK_SYS_SELECTED)
                != 0x1
            {}
        }

        self.registers
            .clk_sys_ctrl
            .modify(CLK_SYS_CTRL::AUXSRC.val(auxiliary_source as u32));
        self.registers
            .clk_sys_ctrl
            .modify(CLK_SYS_CTRL::SRC.val(source as u32));
        while self
            .registers
            .clk_sys_selected
            .read(CLK_SYS_SELECTED::CLK_SYS_SELECTED)
            & (1 << (source as u32))
            == 0x0
        {}

        // pico-sdk:
        // Now that the source is configured, we can trust that the user-supplied
        // divisor is a safe value.
        self.set_divider(Clock::System, div);

        self.set_frequency(Clock::System, freq);
    }

    pub fn configure_reference(
        &self,
        source: ReferenceClockSource,
        auxiliary_source: ReferenceAuxiliaryClockSource,
        source_freq: u32,
        freq: u32,
    ) {
        if freq > source_freq {
            panic!(
                "freq is greater than source freq ({} > {})",
                freq, source_freq
            );
        }
        let div = self.get_divider(source_freq, freq);

        // pico-sdk:
        // If increasing divisor, set divisor before source. Otherwise set source
        // before divisor. This avoids a momentary overspeed when e.g. switching
        // to a faster source and increasing divisor to compensate.
        if div > self.registers.clk_ref_div.get() {
            self.set_divider(Clock::Reference, div);
        }

        // pico-sdk:
        // If switching a glitchless slice (ref or sys) to an aux source, switch
        // away from aux *first* to avoid passing glitches when changing aux mux.
        // Assume (!!!) glitchless source 0 is no faster than the aux source.
        if source == ReferenceClockSource::Auxiliary {
            self.registers
                .clk_ref_ctrl
                .modify(CLK_REF_CTRL::SRC::Rosc_clksrc_ph);
            while self
                .registers
                .clk_ref_selected
                .read(CLK_REF_SELECTED::CLK_REF_SELECTED)
                != 0x1
            {}
        }

        self.registers
            .clk_ref_ctrl
            .modify(CLK_REF_CTRL::AUXSRC.val(auxiliary_source as u32));
        self.registers
            .clk_ref_ctrl
            .modify(CLK_REF_CTRL::SRC.val(source as u32));
        while self
            .registers
            .clk_ref_selected
            .read(CLK_REF_SELECTED::CLK_REF_SELECTED)
            & (1 << (source as u32))
            == 0x0
        {}

        // pico-sdk:
        // Now that the source is configured, we can trust that the user-supplied
        // divisor is a safe value.
        self.set_divider(Clock::Reference, div);

        self.set_frequency(Clock::Reference, freq);
    }

    pub fn configure_peripheral(
        &self,
        auxiliary_source: PeripheralAuxiliaryClockSource,
        freq: u32,
    ) {
        self.registers
            .clk_peri_ctrl
            .modify(CLK_PERI_CTRL::ENABLE::CLEAR);

        // pico-sdk:
        // Delay for 3 cycles of the target clock, for ENABLE propagation.
        // Note XOSC_COUNT is not helpful here because XOSC is not
        // necessarily running, nor is timer... so, 3 cycles per loop:
        self.loop_3_cycles(Clock::Peripheral);

        self.registers
            .clk_peri_ctrl
            .modify(CLK_PERI_CTRL::AUXSRC.val(auxiliary_source as u32));

        self.registers
            .clk_peri_ctrl
            .modify(CLK_PERI_CTRL::ENABLE::SET);

        self.set_frequency(Clock::Peripheral, freq);
    }

    pub fn configure_usb(
        &self,
        auxiliary_source: UsbAuxiliaryClockSource,
        source_freq: u32,
        freq: u32,
    ) {
        if freq > source_freq {
            panic!(
                "freq is greater than source freq ({} > {})",
                freq, source_freq
            );
        }
        let div = self.get_divider(source_freq, freq);

        // pico-sdk:
        // If increasing divisor, set divisor before source. Otherwise set source
        // before divisor. This avoids a momentary overspeed when e.g. switching
        // to a faster source and increasing divisor to compensate.
        if div > self.registers.clk_usb_div.get() {
            self.set_divider(Clock::Usb, div);
        }

        self.registers
            .clk_usb_ctrl
            .modify(CLK_USB_CTRL::ENABLE::CLEAR);
        // pico-sdk:
        // Delay for 3 cycles of the target clock, for ENABLE propagation.
        // Note XOSC_COUNT is not helpful here because XOSC is not
        // necessarily running, nor is timer... so, 3 cycles per loop:
        self.loop_3_cycles(Clock::Usb);

        self.registers
            .clk_usb_ctrl
            .modify(CLK_USB_CTRL::AUXSRC.val(auxiliary_source as u32));

        self.registers
            .clk_usb_ctrl
            .modify(CLK_USB_CTRL::ENABLE::SET);

        // pico-sdk:
        // Now that the source is configured, we can trust that the user-supplied
        // divisor is a safe value.
        self.set_divider(Clock::Usb, div);

        self.set_frequency(Clock::Usb, freq);
    }

    pub fn configure_hstx(
        &self,
        auxiliary_source: HstxAuxiliaryClockSource,
        source_freq: u32,
        freq: u32,
    ) {
        if freq > source_freq {
            panic!(
                "freq is greater than source freq ({} > {})",
                freq, source_freq
            );
        }
        let div = self.get_divider(source_freq, freq);

        // pico-sdk:
        // If increasing divisor, set divisor before source. Otherwise set source
        // before divisor. This avoids a momentary overspeed when e.g. switching
        // to a faster source and increasing divisor to compensate.
        if div > self.registers.clk_hstx_div.get() {
            self.set_divider(Clock::Hstx, div);
        }

        self.registers
            .clk_hstx_ctrl
            .modify(CLK_HSTX_CTRL::ENABLE::CLEAR);
        // pico-sdk:
        // Delay for 3 cycles of the target clock, for ENABLE propagation.
        // Note XOSC_COUNT is not helpful here because XOSC is not
        // necessarily running, nor is timer... so, 3 cycles per loop:
        self.loop_3_cycles(Clock::Hstx);

        self.registers
            .clk_hstx_ctrl
            .modify(CLK_HSTX_CTRL::AUXSRC.val(auxiliary_source as u32));

        self.registers
            .clk_hstx_ctrl
            .modify(CLK_HSTX_CTRL::ENABLE::SET);

        // pico-sdk:
        // Now that the source is configured, we can trust that the user-supplied
        // divisor is a safe value.
        self.set_divider(Clock::Hstx, div);

        self.set_frequency(Clock::Hstx, freq);
    }

    pub fn configure_adc(
        &self,
        auxiliary_source: AdcAuxiliaryClockSource,
        source_freq: u32,
        freq: u32,
    ) {
        if freq > source_freq {
            panic!(
                "freq is greater than source freq ({} > {})",
                freq, source_freq
            );
        }
        let div = self.get_divider(source_freq, freq);

        // pico-sdk:
        // If increasing divisor, set divisor before source. Otherwise set source
        // before divisor. This avoids a momentary overspeed when e.g. switching
        // to a faster source and increasing divisor to compensate.
        if div > self.registers.clk_adc_div.get() {
            self.set_divider(Clock::Adc, div);
        }

        self.registers
            .clk_adc_ctrl
            .modify(CLK_ADC_CTRL::ENABLE::CLEAR);
        // pico-sdk:
        // Delay for 3 cycles of the target clock, for ENABLE propagation.
        // Note XOSC_COUNT is not helpful here because XOSC is not
        // necessarily running, nor is timer... so, 3 cycles per loop:
        self.loop_3_cycles(Clock::Adc);

        self.registers
            .clk_adc_ctrl
            .modify(CLK_ADC_CTRL::AUXSRC.val(auxiliary_source as u32));

        self.registers
            .clk_adc_ctrl
            .modify(CLK_ADC_CTRL::ENABLE::SET);

        // pico-sdk:
        // Now that the source is configured, we can trust that the user-supplied
        // divisor is a safe value.
        self.set_divider(Clock::Adc, div);

        self.set_frequency(Clock::Adc, freq);
    }
}

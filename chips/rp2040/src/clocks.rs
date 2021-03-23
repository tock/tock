use kernel::common::registers::{register_bitfields, register_structs, ReadOnly, ReadWrite};
use kernel::common::StaticRef;

register_structs! {

    ClocksRegisters {
        /// Clock control, can be changed on-the-fly (except for auxsrc)
        (0x000 => clk_gpout0_ctrl: ReadWrite<u32, CLK_GPOUT0_CTRL::Register>),
        /// Clock divisor, can be changed on-the-fly
        (0x004 => clk_gpout0_div: ReadWrite<u32, CLK_GPOUT0_DIV::Register>),
        /// Indicates which src is currently selected (one-hot)
        (0x008 => clk_gpout0_selected: ReadOnly<u32, CLK_GPOUT0_SELECTED::Register>),
        /// Clock control, can be changed on-the-fly (except for auxsrc)
        (0x00C => clk_gpout1_ctrl: ReadWrite<u32, CLK_GPOUT1_CTRL::Register>),
        /// Clock divisor, can be changed on-the-fly
        (0x010 => clk_gpout1_div: ReadWrite<u32, CLK_GPOUT1_DIV::Register>),
        /// Indicates which src is currently selected (one-hot)
        (0x014 => clk_gpout1_selected: ReadOnly<u32, CLK_GPOUT1_SELECTED::Register>),
        /// Clock control, can be changed on-the-fly (except for auxsrc)
        (0x018 => clk_gpout2_ctrl: ReadWrite<u32, CLK_GPOUT2_CTRL::Register>),
        /// Clock divisor, can be changed on-the-fly
        (0x01C => clk_gpout2_div: ReadWrite<u32, CLK_GPOUT2_DIV::Register>),
        /// Indicates which src is currently selected (one-hot)
        (0x020 => clk_gpout2_selected: ReadOnly<u32, CLK_GPOUT2_SELECTED::Register>),
        /// Clock control, can be changed on-the-fly (except for auxsrc)
        (0x024 => clk_gpout3_ctrl: ReadWrite<u32, CLK_GPOUT3_CTRL::Register>),
        /// Clock divisor, can be changed on-the-fly
        (0x028 => clk_gpout3_div: ReadWrite<u32, CLK_GPOUT3_DIV::Register>),
        /// Indicates which src is currently selected (one-hot)
        (0x02C => clk_gpout3_selected: ReadOnly<u32, CLK_GPOUT3_SELECTED::Register>),
        /// Clock control, can be changed on-the-fly (except for auxsrc)
        (0x030 => clk_ref_ctrl: ReadWrite<u32, CLK_REF_CTRL::Register>),
        /// Clock divisor, can be changed on-the-fly
        (0x034 => clk_ref_div: ReadWrite<u32>),
        /// Indicates which src is currently selected (one-hot)
        (0x038 => clk_ref_selected: ReadOnly<u32, CLK_REF_SELECTED::Register>),
        /// Clock control, can be changed on-the-fly (except for auxsrc)
        (0x03C => clk_sys_ctrl: ReadWrite<u32, CLK_SYS_CTRL::Register>),
        /// Clock divisor, can be changed on-the-fly
        (0x040 => clk_sys_div: ReadWrite<u32, CLK_SYS_DIV::Register>),
        /// Indicates which src is currently selected (one-hot)
        (0x044 => clk_sys_selected: ReadOnly<u32, CLK_SYS_SELECTED::Register>),
        /// Clock control, can be changed on-the-fly (except for auxsrc)
        (0x048 => clk_peri_ctrl: ReadWrite<u32, CLK_PERI_CTRL::Register>),
        (0x04C => _reserved0),
        /// Indicates which src is currently selected (one-hot)
        (0x050 => clk_peri_selected: ReadOnly<u32, CLK_PERI_SELECTED::Register>),
        /// Clock control, can be changed on-the-fly (except for auxsrc)
        (0x054 => clk_usb_ctrl: ReadWrite<u32, CLK_USB_CTRL::Register>),
        /// Clock divisor, can be changed on-the-fly
        (0x058 => clk_usb_div: ReadWrite<u32>),
        /// Indicates which src is currently selected (one-hot)
        (0x05C => clk_usb_selected: ReadOnly<u32, CLK_USB_SELECTED::Register>),
        /// Clock control, can be changed on-the-fly (except for auxsrc)
        (0x060 => clk_adc_ctrl: ReadWrite<u32, CLK_ADC_CTRL::Register>),
        /// Clock divisor, can be changed on-the-fly
        (0x064 => clk_adc_div: ReadWrite<u32>),
        /// Indicates which src is currently selected (one-hot)
        (0x068 => clk_adc_selected: ReadOnly<u32, CLK_ADC_SELECTED::Register>),
        /// Clock control, can be changed on-the-fly (except for auxsrc)
        (0x06C => clk_rtc_ctrl: ReadWrite<u32, CLK_RTC_CTRL::Register>),
        /// Clock divisor, can be changed on-the-fly
        (0x070 => clk_rtc_div: ReadWrite<u32, CLK_RTC_DIV::Register>),
        /// Indicates which src is currently selected (one-hot)
        (0x074 => clk_rtc_selected: ReadOnly<u32, CLK_RTC_SELECTED::Register>),

        (0x078 => clk_sys_resus_ctrl: ReadWrite<u32, CLK_SYS_RESUS_CTRL::Register>),

        (0x07C => clk_sys_resus_status: ReadWrite<u32>),
        /// Reference clock frequency in kHz
        (0x080 => fc0_ref_khz: ReadWrite<u32>),
        /// Minimum pass frequency in kHz. This is optional. Set to 0 if you are not using t
        (0x084 => fc0_min_khz: ReadWrite<u32>),
        /// Maximum pass frequency in kHz. This is optional. Set to 0x1ffffff if you are not
        (0x088 => fc0_max_khz: ReadWrite<u32>),
        /// Delays the start of frequency counting to allow the mux to settle\n
        /// Delay is measured in multiples of the reference clock period
        (0x08C => fc0_delay: ReadWrite<u32>),
        /// The test interval is 0.98us * 2**interval, but let's call it 1us * 2**interval\n
        /// The default gives a test interval of 250us
        (0x090 => fc0_interval: ReadWrite<u32>),
        /// Clock sent to frequency counter, set to 0 when not required\n
        /// Writing to this register initiates the frequency count
        (0x094 => fc0_src: ReadWrite<u32>),
        /// Frequency counter status
        (0x098 => fc0_status: ReadWrite<u32, FC0_STATUS::Register>),
        /// Result of frequency measurement, only valid when status_done=1
        (0x09C => fc0_result: ReadWrite<u32, FC0_RESULT::Register>),
        /// enable clock in wake mode
        (0x0A0 => wake_en0: ReadWrite<u32, WAKE_EN0::Register>),
        /// enable clock in wake mode
        (0x0A4 => wake_en1: ReadWrite<u32, WAKE_EN1::Register>),
        /// enable clock in sleep mode
        (0x0A8 => sleep_en0: ReadWrite<u32, SLEEP_EN0::Register>),
        /// enable clock in sleep mode
        (0x0AC => sleep_en1: ReadWrite<u32, SLEEP_EN1::Register>),
        /// indicates the state of the clock enable
        (0x0B0 => enabled0: ReadWrite<u32, ENABLED0::Register>),
        /// indicates the state of the clock enable
        (0x0B4 => enabled1: ReadWrite<u32, ENABLED1::Register>),
        /// Raw Interrupts
        (0x0B8 => intr: ReadWrite<u32>),
        /// Interrupt Enable
        (0x0BC => inte: ReadWrite<u32>),
        /// Interrupt Force
        (0x0C0 => intf: ReadWrite<u32>),
        /// Interrupt status after masking & forcing
        (0x0C4 => ints: ReadWrite<u32>),
        (0x0C8 => @END),
    }
}

register_bitfields![u32,
    CLK_GPOUT0_CTRL [
        /// An edge on this signal shifts the phase of the output by 1 cycle of the input cl
        /// This can be done at any time
        NUDGE OFFSET(20) NUMBITS(1) [],
        /// This delays the enable signal by up to 3 cycles of the input clock\n
        /// This must be set before the clock is enabled to have any effect
        PHASE OFFSET(16) NUMBITS(2) [],
        /// Enables duty cycle correction for odd divisors
        DC50 OFFSET(12) NUMBITS(1) [],
        /// Starts and stops the clock generator cleanly
        ENABLE OFFSET(11) NUMBITS(1) [],
        /// Asynchronously kills the clock generator
        KILL OFFSET(10) NUMBITS(1) [],
        /// Selects the auxiliary clock source, will glitch when switching
        AUXSRC OFFSET(5) NUMBITS(4) [

            Clksrc_pll_sys = 0
        ]
    ],
    CLK_GPOUT0_DIV [
        /// Integer component of the divisor, 0 -> divide by 2^16
        INT OFFSET(8) NUMBITS(24) [],
        /// Fractional component of the divisor
        FRAC OFFSET(0) NUMBITS(8) []
    ],
    CLK_GPOUT0_SELECTED [
        VALUE OFFSET (0) NUMBITS (32) []
    ],
    CLK_GPOUT1_CTRL [
        /// An edge on this signal shifts the phase of the output by 1 cycle of the input cl
        /// This can be done at any time
        NUDGE OFFSET(20) NUMBITS(1) [],
        /// This delays the enable signal by up to 3 cycles of the input clock\n
        /// This must be set before the clock is enabled to have any effect
        PHASE OFFSET(16) NUMBITS(2) [],
        /// Enables duty cycle correction for odd divisors
        DC50 OFFSET(12) NUMBITS(1) [],
        /// Starts and stops the clock generator cleanly
        ENABLE OFFSET(11) NUMBITS(1) [],
        /// Asynchronously kills the clock generator
        KILL OFFSET(10) NUMBITS(1) [],
        /// Selects the auxiliary clock source, will glitch when switching
        AUXSRC OFFSET(5) NUMBITS(4) [

            Clksrc_pll_sys = 0
        ]
    ],
    CLK_GPOUT1_DIV [
        /// Integer component of the divisor, 0 -> divide by 2^16
        INT OFFSET(8) NUMBITS(24) [],
        /// Fractional component of the divisor
        FRAC OFFSET(0) NUMBITS(8) []
    ],
    CLK_GPOUT1_SELECTED [
        VALUE OFFSET (0) NUMBITS (32) []
    ],
    CLK_GPOUT2_CTRL [
        /// An edge on this signal shifts the phase of the output by 1 cycle of the input cl
        /// This can be done at any time
        NUDGE OFFSET(20) NUMBITS(1) [],
        /// This delays the enable signal by up to 3 cycles of the input clock\n
        /// This must be set before the clock is enabled to have any effect
        PHASE OFFSET(16) NUMBITS(2) [],
        /// Enables duty cycle correction for odd divisors
        DC50 OFFSET(12) NUMBITS(1) [],
        /// Starts and stops the clock generator cleanly
        ENABLE OFFSET(11) NUMBITS(1) [],
        /// Asynchronously kills the clock generator
        KILL OFFSET(10) NUMBITS(1) [],
        /// Selects the auxiliary clock source, will glitch when switching
        AUXSRC OFFSET(5) NUMBITS(4) [

            Clksrc_pll_sys = 0
        ]
    ],
    CLK_GPOUT2_DIV [
        /// Integer component of the divisor, 0 -> divide by 2^16
        INT OFFSET(8) NUMBITS(24) [],
        /// Fractional component of the divisor
        FRAC OFFSET(0) NUMBITS(8) []
    ],
    CLK_GPOUT2_SELECTED [
        VALUE OFFSET (0) NUMBITS (32) []
    ],
    CLK_GPOUT3_CTRL [
        /// An edge on this signal shifts the phase of the output by 1 cycle of the input cl
        /// This can be done at any time
        NUDGE OFFSET(20) NUMBITS(1) [],
        /// This delays the enable signal by up to 3 cycles of the input clock\n
        /// This must be set before the clock is enabled to have any effect
        PHASE OFFSET(16) NUMBITS(2) [],
        /// Enables duty cycle correction for odd divisors
        DC50 OFFSET(12) NUMBITS(1) [],
        /// Starts and stops the clock generator cleanly
        ENABLE OFFSET(11) NUMBITS(1) [],
        /// Asynchronously kills the clock generator
        KILL OFFSET(10) NUMBITS(1) [],
        /// Selects the auxiliary clock source, will glitch when switching
        AUXSRC OFFSET(5) NUMBITS(4) [

            Clksrc_pll_sys = 0
        ]
    ],
    CLK_GPOUT3_DIV [
        /// Integer component of the divisor, 0 -> divide by 2^16
        INT OFFSET(8) NUMBITS(24) [],
        /// Fractional component of the divisor
        FRAC OFFSET(0) NUMBITS(8) []
    ],
    CLK_GPOUT3_SELECTED [
        VALUE OFFSET (0) NUMBITS (32) []
    ],
    CLK_REF_CTRL [
        /// Selects the auxiliary clock source, will glitch when switching
        AUXSRC OFFSET(5) NUMBITS(2) [

            CLKSRC_PLL_USB = 0x0,
            CLKSRC_GPIN0 = 0x1,
            CLKSRC_GPIN1 = 0x2
        ],
        /// Selects the clock source glitchlessly, can be changed on-the-fly
        SRC OFFSET(0) NUMBITS(2) [

            ROSC_CLKSRC_PH = 0x0,
            CLKSRC_CLK_REF_AUX = 0x1,
            XOSC_CLKSRC = 0x2
        ]
    ],
    CLK_REF_DIV [
        /// Integer component of the divisor, 0 -> divide by 2^16
        INT OFFSET(8) NUMBITS(2) []
    ],
    CLK_REF_SELECTED [
        VALUE OFFSET (0) NUMBITS (32) []
    ],
    CLK_SYS_CTRL [
        /// Selects the auxiliary clock source, will glitch when switching
        AUXSRC OFFSET(5) NUMBITS(3) [

            CLKSRC_PLL_SYS = 0x0,
            CLKSRC_PLL_USB = 0x1,
            ROSC_CLKSRC = 0x2,
            XOSC_CLKSRC = 0x3,
            CLKSRC_GPIN0 = 0x4,
            CLKSRC_GPIN1 = 0x5
        ],
        /// Selects the clock source glitchlessly, can be changed on-the-fly
        SRC OFFSET(0) NUMBITS(1) [
            CLKSRC_CLK_SYS_AUX = 1,
            CLK_REF = 0,
        ]
    ],
    CLK_SYS_DIV [
        /// Integer component of the divisor, 0 -> divide by 2^16
        INT OFFSET(8) NUMBITS(24) [],
        /// Fractional component of the divisor
        FRAC OFFSET(0) NUMBITS(8) []
    ],
    CLK_SYS_SELECTED [
        VALUE OFFSET (0) NUMBITS (32) []
    ],
    CLK_PERI_CTRL [
        /// Starts and stops the clock generator cleanly
        ENABLE OFFSET(11) NUMBITS(1) [],
        /// Asynchronously kills the clock generator
        KILL OFFSET(10) NUMBITS(1) [],
        /// Selects the auxiliary clock source, will glitch when switching
        AUXSRC OFFSET(5) NUMBITS(3) [

            Clk_sys = 0
        ]
    ],
    CLK_PERI_SELECTED [
        VALUE OFFSET (0) NUMBITS (32) []
    ],
    CLK_USB_CTRL [
        /// An edge on this signal shifts the phase of the output by 1 cycle of the input cl
        /// This can be done at any time
        NUDGE OFFSET(20) NUMBITS(1) [],
        /// This delays the enable signal by up to 3 cycles of the input clock\n
        /// This must be set before the clock is enabled to have any effect
        PHASE OFFSET(16) NUMBITS(2) [],
        /// Starts and stops the clock generator cleanly
        ENABLE OFFSET(11) NUMBITS(1) [],
        /// Asynchronously kills the clock generator
        KILL OFFSET(10) NUMBITS(1) [],
        /// Selects the auxiliary clock source, will glitch when switching
        AUXSRC OFFSET(5) NUMBITS(3) [

            Clksrc_pll_usb = 0
        ]
    ],
    CLK_USB_DIV [
        /// Integer component of the divisor, 0 -> divide by 2^16
        INT OFFSET(8) NUMBITS(2) []
    ],
    CLK_USB_SELECTED [
        VALUE OFFSET (0) NUMBITS (32) []
    ],
    CLK_ADC_CTRL [
        /// An edge on this signal shifts the phase of the output by 1 cycle of the input cl
        /// This can be done at any time
        NUDGE OFFSET(20) NUMBITS(1) [],
        /// This delays the enable signal by up to 3 cycles of the input clock\n
        /// This must be set before the clock is enabled to have any effect
        PHASE OFFSET(16) NUMBITS(2) [],
        /// Starts and stops the clock generator cleanly
        ENABLE OFFSET(11) NUMBITS(1) [],
        /// Asynchronously kills the clock generator
        KILL OFFSET(10) NUMBITS(1) [],
        /// Selects the auxiliary clock source, will glitch when switching
        AUXSRC OFFSET(5) NUMBITS(3) [

            Clksrc_pll_usb = 0
        ]
    ],
    CLK_ADC_DIV [
        /// Integer component of the divisor, 0 -> divide by 2^16
        INT OFFSET(8) NUMBITS(2) []
    ],
    CLK_ADC_SELECTED [
        VALUE OFFSET (0) NUMBITS (32) []
    ],
    CLK_RTC_CTRL [
        /// An edge on this signal shifts the phase of the output by 1 cycle of the input cl
        /// This can be done at any time
        NUDGE OFFSET(20) NUMBITS(1) [],
        /// This delays the enable signal by up to 3 cycles of the input clock\n
        /// This must be set before the clock is enabled to have any effect
        PHASE OFFSET(16) NUMBITS(2) [],
        /// Starts and stops the clock generator cleanly
        ENABLE OFFSET(11) NUMBITS(1) [],
        /// Asynchronously kills the clock generator
        KILL OFFSET(10) NUMBITS(1) [],
        /// Selects the auxiliary clock source, will glitch when switching
        AUXSRC OFFSET(5) NUMBITS(3) [

            Clksrc_pll_usb = 0
        ]
    ],
    CLK_RTC_DIV [
        /// Integer component of the divisor, 0 -> divide by 2^16
        INT OFFSET(8) NUMBITS(24) [],
        /// Fractional component of the divisor
        FRAC OFFSET(0) NUMBITS(8) []
    ],
    CLK_RTC_SELECTED [
        VALUE OFFSET (0) NUMBITS (32) []
    ],
    CLK_SYS_RESUS_CTRL [
        /// For clearing the resus after the fault that triggered it has been corrected
        CLEAR OFFSET(16) NUMBITS(1) [],
        /// Force a resus, for test purposes only
        FRCE OFFSET(12) NUMBITS(1) [],
        /// Enable resus
        ENABLE OFFSET(8) NUMBITS(1) [],
        /// This is expressed as a number of clk_ref cycles\n
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

        clk_sys_sram3 OFFSET(31) NUMBITS(1) [],

        clk_sys_sram2 OFFSET(30) NUMBITS(1) [],

        clk_sys_sram1 OFFSET(29) NUMBITS(1) [],

        clk_sys_sram0 OFFSET(28) NUMBITS(1) [],

        clk_sys_spi1 OFFSET(27) NUMBITS(1) [],

        clk_peri_spi1 OFFSET(26) NUMBITS(1) [],

        clk_sys_spi0 OFFSET(25) NUMBITS(1) [],

        clk_peri_spi0 OFFSET(24) NUMBITS(1) [],

        clk_sys_sio OFFSET(23) NUMBITS(1) [],

        clk_sys_rtc OFFSET(22) NUMBITS(1) [],

        clk_rtc_rtc OFFSET(21) NUMBITS(1) [],

        clk_sys_rosc OFFSET(20) NUMBITS(1) [],

        clk_sys_rom OFFSET(19) NUMBITS(1) [],

        clk_sys_resets OFFSET(18) NUMBITS(1) [],

        clk_sys_pwm OFFSET(17) NUMBITS(1) [],

        clk_sys_psm OFFSET(16) NUMBITS(1) [],

        clk_sys_pll_usb OFFSET(15) NUMBITS(1) [],

        clk_sys_pll_sys OFFSET(14) NUMBITS(1) [],

        clk_sys_pio1 OFFSET(13) NUMBITS(1) [],

        clk_sys_pio0 OFFSET(12) NUMBITS(1) [],

        clk_sys_pads OFFSET(11) NUMBITS(1) [],

        clk_sys_vreg_and_chip_reset OFFSET(10) NUMBITS(1) [],

        clk_sys_jtag OFFSET(9) NUMBITS(1) [],

        clk_sys_io OFFSET(8) NUMBITS(1) [],

        clk_sys_i2c1 OFFSET(7) NUMBITS(1) [],

        clk_sys_i2c0 OFFSET(6) NUMBITS(1) [],

        clk_sys_dma OFFSET(5) NUMBITS(1) [],

        clk_sys_busfabric OFFSET(4) NUMBITS(1) [],

        clk_sys_busctrl OFFSET(3) NUMBITS(1) [],

        clk_sys_adc OFFSET(2) NUMBITS(1) [],

        clk_adc_adc OFFSET(1) NUMBITS(1) [],

        clk_sys_clocks OFFSET(0) NUMBITS(1) []
    ],
    WAKE_EN1 [

        clk_sys_xosc OFFSET(14) NUMBITS(1) [],

        clk_sys_xip OFFSET(13) NUMBITS(1) [],

        clk_sys_watchdog OFFSET(12) NUMBITS(1) [],

        clk_usb_usbctrl OFFSET(11) NUMBITS(1) [],

        clk_sys_usbctrl OFFSET(10) NUMBITS(1) [],

        clk_sys_uart1 OFFSET(9) NUMBITS(1) [],

        clk_peri_uart1 OFFSET(8) NUMBITS(1) [],

        clk_sys_uart0 OFFSET(7) NUMBITS(1) [],

        clk_peri_uart0 OFFSET(6) NUMBITS(1) [],

        clk_sys_timer OFFSET(5) NUMBITS(1) [],

        clk_sys_tbman OFFSET(4) NUMBITS(1) [],

        clk_sys_sysinfo OFFSET(3) NUMBITS(1) [],

        clk_sys_syscfg OFFSET(2) NUMBITS(1) [],

        clk_sys_sram5 OFFSET(1) NUMBITS(1) [],

        clk_sys_sram4 OFFSET(0) NUMBITS(1) []
    ],
    SLEEP_EN0 [

        clk_sys_sram3 OFFSET(31) NUMBITS(1) [],

        clk_sys_sram2 OFFSET(30) NUMBITS(1) [],

        clk_sys_sram1 OFFSET(29) NUMBITS(1) [],

        clk_sys_sram0 OFFSET(28) NUMBITS(1) [],

        clk_sys_spi1 OFFSET(27) NUMBITS(1) [],

        clk_peri_spi1 OFFSET(26) NUMBITS(1) [],

        clk_sys_spi0 OFFSET(25) NUMBITS(1) [],

        clk_peri_spi0 OFFSET(24) NUMBITS(1) [],

        clk_sys_sio OFFSET(23) NUMBITS(1) [],

        clk_sys_rtc OFFSET(22) NUMBITS(1) [],

        clk_rtc_rtc OFFSET(21) NUMBITS(1) [],

        clk_sys_rosc OFFSET(20) NUMBITS(1) [],

        clk_sys_rom OFFSET(19) NUMBITS(1) [],

        clk_sys_resets OFFSET(18) NUMBITS(1) [],

        clk_sys_pwm OFFSET(17) NUMBITS(1) [],

        clk_sys_psm OFFSET(16) NUMBITS(1) [],

        clk_sys_pll_usb OFFSET(15) NUMBITS(1) [],

        clk_sys_pll_sys OFFSET(14) NUMBITS(1) [],

        clk_sys_pio1 OFFSET(13) NUMBITS(1) [],

        clk_sys_pio0 OFFSET(12) NUMBITS(1) [],

        clk_sys_pads OFFSET(11) NUMBITS(1) [],

        clk_sys_vreg_and_chip_reset OFFSET(10) NUMBITS(1) [],

        clk_sys_jtag OFFSET(9) NUMBITS(1) [],

        clk_sys_io OFFSET(8) NUMBITS(1) [],

        clk_sys_i2c1 OFFSET(7) NUMBITS(1) [],

        clk_sys_i2c0 OFFSET(6) NUMBITS(1) [],

        clk_sys_dma OFFSET(5) NUMBITS(1) [],

        clk_sys_busfabric OFFSET(4) NUMBITS(1) [],

        clk_sys_busctrl OFFSET(3) NUMBITS(1) [],

        clk_sys_adc OFFSET(2) NUMBITS(1) [],

        clk_adc_adc OFFSET(1) NUMBITS(1) [],

        clk_sys_clocks OFFSET(0) NUMBITS(1) []
    ],
    SLEEP_EN1 [

        clk_sys_xosc OFFSET(14) NUMBITS(1) [],

        clk_sys_xip OFFSET(13) NUMBITS(1) [],

        clk_sys_watchdog OFFSET(12) NUMBITS(1) [],

        clk_usb_usbctrl OFFSET(11) NUMBITS(1) [],

        clk_sys_usbctrl OFFSET(10) NUMBITS(1) [],

        clk_sys_uart1 OFFSET(9) NUMBITS(1) [],

        clk_peri_uart1 OFFSET(8) NUMBITS(1) [],

        clk_sys_uart0 OFFSET(7) NUMBITS(1) [],

        clk_peri_uart0 OFFSET(6) NUMBITS(1) [],

        clk_sys_timer OFFSET(5) NUMBITS(1) [],

        clk_sys_tbman OFFSET(4) NUMBITS(1) [],

        clk_sys_sysinfo OFFSET(3) NUMBITS(1) [],

        clk_sys_syscfg OFFSET(2) NUMBITS(1) [],

        clk_sys_sram5 OFFSET(1) NUMBITS(1) [],

        clk_sys_sram4 OFFSET(0) NUMBITS(1) []
    ],
    ENABLED0 [

        clk_sys_sram3 OFFSET(31) NUMBITS(1) [],

        clk_sys_sram2 OFFSET(30) NUMBITS(1) [],

        clk_sys_sram1 OFFSET(29) NUMBITS(1) [],

        clk_sys_sram0 OFFSET(28) NUMBITS(1) [],

        clk_sys_spi1 OFFSET(27) NUMBITS(1) [],

        clk_peri_spi1 OFFSET(26) NUMBITS(1) [],

        clk_sys_spi0 OFFSET(25) NUMBITS(1) [],

        clk_peri_spi0 OFFSET(24) NUMBITS(1) [],

        clk_sys_sio OFFSET(23) NUMBITS(1) [],

        clk_sys_rtc OFFSET(22) NUMBITS(1) [],

        clk_rtc_rtc OFFSET(21) NUMBITS(1) [],

        clk_sys_rosc OFFSET(20) NUMBITS(1) [],

        clk_sys_rom OFFSET(19) NUMBITS(1) [],

        clk_sys_resets OFFSET(18) NUMBITS(1) [],

        clk_sys_pwm OFFSET(17) NUMBITS(1) [],

        clk_sys_psm OFFSET(16) NUMBITS(1) [],

        clk_sys_pll_usb OFFSET(15) NUMBITS(1) [],

        clk_sys_pll_sys OFFSET(14) NUMBITS(1) [],

        clk_sys_pio1 OFFSET(13) NUMBITS(1) [],

        clk_sys_pio0 OFFSET(12) NUMBITS(1) [],

        clk_sys_pads OFFSET(11) NUMBITS(1) [],

        clk_sys_vreg_and_chip_reset OFFSET(10) NUMBITS(1) [],

        clk_sys_jtag OFFSET(9) NUMBITS(1) [],

        clk_sys_io OFFSET(8) NUMBITS(1) [],

        clk_sys_i2c1 OFFSET(7) NUMBITS(1) [],

        clk_sys_i2c0 OFFSET(6) NUMBITS(1) [],

        clk_sys_dma OFFSET(5) NUMBITS(1) [],

        clk_sys_busfabric OFFSET(4) NUMBITS(1) [],

        clk_sys_busctrl OFFSET(3) NUMBITS(1) [],

        clk_sys_adc OFFSET(2) NUMBITS(1) [],

        clk_adc_adc OFFSET(1) NUMBITS(1) [],

        clk_sys_clocks OFFSET(0) NUMBITS(1) []
    ],
    ENABLED1 [

        clk_sys_xosc OFFSET(14) NUMBITS(1) [],

        clk_sys_xip OFFSET(13) NUMBITS(1) [],

        clk_sys_watchdog OFFSET(12) NUMBITS(1) [],

        clk_usb_usbctrl OFFSET(11) NUMBITS(1) [],

        clk_sys_usbctrl OFFSET(10) NUMBITS(1) [],

        clk_sys_uart1 OFFSET(9) NUMBITS(1) [],

        clk_peri_uart1 OFFSET(8) NUMBITS(1) [],

        clk_sys_uart0 OFFSET(7) NUMBITS(1) [],

        clk_peri_uart0 OFFSET(6) NUMBITS(1) [],

        clk_sys_timer OFFSET(5) NUMBITS(1) [],

        clk_sys_tbman OFFSET(4) NUMBITS(1) [],

        clk_sys_sysinfo OFFSET(3) NUMBITS(1) [],

        clk_sys_syscfg OFFSET(2) NUMBITS(1) [],

        clk_sys_sram5 OFFSET(1) NUMBITS(1) [],

        clk_sys_sram4 OFFSET(0) NUMBITS(1) []
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
    ]
];

register_structs! {

    PllRegisters {
        /// Control and Status\n
        /// GENERAL CONSTRAINTS:\n
        /// Reference clock frequency min=5MHz, max=800MHz\n
        /// Feedback divider min=16, max=320\n
        /// VCO frequency min=400MHz, max=1600MHz
        (0x000 => cs: ReadWrite<u32, CS::Register>),
        /// Controls the PLL power modes.
        (0x004 => pwr: ReadWrite<u32, PWR::Register>),
        /// Feedback divisor\n
        /// (note: this PLL does not support fractional division)
        (0x008 => fbdiv_int: ReadWrite<u32, FBDIV_INT::Register>),
        /// Controls the PLL post dividers for the primary output\n
        /// (note: this PLL does not have a secondary output)\n
        /// the primary output is driven from VCO divided by postdiv1*postdiv2
        (0x00C => prim: ReadWrite<u32, PRIM::Register>),
        (0x010 => @END),
    }
}
register_bitfields![u32,
    CS [
        /// PLL is locked
        LOCK OFFSET(31) NUMBITS(1) [],
        /// Passes the reference clock to the output instead of the divided VCO. The VCO con
        BYPASS OFFSET(8) NUMBITS(1) [],
        /// Divides the PLL input reference clock.\n
        /// Behaviour is undefined for div=0.\n
        /// PLL output will be unpredictable during refdiv changes, wait for
        REFDIV OFFSET(0) NUMBITS(6) []
    ],
    PWR [
        /// PLL VCO powerdown\n
        /// To save power set high when PLL output not required or bypass=1.
        VCOPD OFFSET(5) NUMBITS(1) [],
        /// PLL post divider powerdown\n
        /// To save power set high when PLL output not required or bypass=1.
        POSTDIVPD OFFSET(3) NUMBITS(1) [],
        /// PLL DSM powerdown\n
        /// Nothing is achieved by setting this low.
        DSMPD OFFSET(2) NUMBITS(1) [],
        /// PLL powerdown\n
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
    ]
];

const PLL_SYS_BASE: StaticRef<PllRegisters> =
    unsafe { StaticRef::new(0x40028000 as *const PllRegisters) };

const PLL_USB_BASE: StaticRef<PllRegisters> =
    unsafe { StaticRef::new(0x4002C000 as *const PllRegisters) };

const CLOCKS_BASE: StaticRef<ClocksRegisters> =
    unsafe { StaticRef::new(0x40008000 as *const ClocksRegisters) };

pub struct Clocks {
    registers: StaticRef<ClocksRegisters>,
    pll_registers: &'static [StaticRef<PllRegisters>],
}

pub enum PllClock {
    Sys = 0,
    Usb = 1,
}

impl Clocks {
    pub const fn new() -> Self {
        Self {
            registers: CLOCKS_BASE,
            pll_registers: &[PLL_SYS_BASE, PLL_USB_BASE],
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
            .modify(CLK_SYS_CTRL::SRC::CLK_REF);
        while self
            .registers
            .clk_sys_selected
            .read(CLK_SYS_SELECTED::VALUE)
            != 0x1
        {}
    }

    pub fn disable_ref_aux(&self) {
        self.registers
            .clk_ref_ctrl
            .modify(CLK_REF_CTRL::SRC::ROSC_CLKSRC_PH);
        while self
            .registers
            .clk_ref_selected
            .read(CLK_REF_SELECTED::VALUE)
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
}

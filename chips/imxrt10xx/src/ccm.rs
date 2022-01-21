use kernel::platform::chip::ClockInterface;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{register_bitfields, register_structs, ReadOnly, ReadWrite};
use kernel::utilities::StaticRef;

register_structs! {
    /// Clock Controller Module
    CcmRegisters {
        /// CCM Control Register
        (0x000 => ccr: ReadWrite<u32, CCR::Register>),
        (0x004 => _reserved0),
        /// CCM Status Register
        (0x008 => csr: ReadOnly<u32, CSR::Register>),
        /// CCM Clock Switcher Register
        (0x00C => ccsr: ReadWrite<u32>),
        /// CCM Arm Clock Root Register
        (0x010 => cacrr: ReadWrite<u32>),
        /// CCM Bus Clock Divider Register
        (0x014 => cbcdr: ReadWrite<u32, CBCDR::Register>),
        /// CCM Bus Clock Multiplexer Register
        (0x018 => cbcmr: ReadWrite<u32, CBCMR::Register>),
        /// CCM Serial Clock Multiplexer Register 1
        (0x01C => cscmr1: ReadWrite<u32, CSCMR1::Register>),
        /// CCM Serial Clock Multiplexer Register 2
        (0x020 => cscmr2: ReadWrite<u32>),
        /// CCM Serial Clock Divider Register 1
        (0x024 => cscdr1: ReadWrite<u32, CSCDR1::Register>),
        /// CCM Clock Divider Register
        (0x028 => cs1cdr: ReadWrite<u32>),
        /// CCM Clock Divider Register
        (0x02C => cs2cdr: ReadWrite<u32>),
        /// CCM D1 Clock Divider Register
        (0x030 => cdcdr: ReadWrite<u32>),
        (0x034 => _reserved1),
        /// CCM Serial Clock Divider Register 2
        (0x038 => cscdr2: ReadWrite<u32>),
        /// CCM Serial Clock Divider Register 3
        (0x03C => cscdr3: ReadWrite<u32>),
        (0x040 => _reserved2),
        /// CCM Divider Handshake In-Process Register
        (0x048 => cdhipr: ReadOnly<u32>),
        (0x04C => _reserved3),
        /// CCM Low Power Control Register
        (0x054 => clpcr: ReadWrite<u32, CLPCR::Register>),
        /// CCM Interrupt Status Register
        (0x058 => cisr: ReadWrite<u32>),
        /// CCM Interrupt Mask Register
        (0x05C => cimr: ReadWrite<u32>),
        /// CCM Clock Output Source Register
        (0x060 => ccosr: ReadWrite<u32>),
        /// CCM General Purpose Register
        (0x064 => cgpr: ReadWrite<u32>),
        /// CCM Clock Gating Registers
        (0x068 => ccgr: [ReadWrite<u32, CCGR::Register>; 8]),
        /// CCM Module Enable Overide Register
        (0x088 => cmeor: ReadWrite<u32>),
        (0x08C => @END),
    }
}

register_bitfields![u32,
    CCR [
        /// Enable for REG_BYPASS_COUNTER
        RBC_EN OFFSET(27) NUMBITS(1) [],
        /// Counter for analog_reg_bypass
        REG_BYPASS_COUNT OFFSET(21) NUMBITS(6) [],
        /// On chip oscillator enable bit
        COSC_EN OFFSET(12) NUMBITS(1) [],
        /// Oscillator ready counter value
        OSCNT OFFSET(0) NUMBITS(8) []
    ],
    CSR [
        // Status indication of on board oscillator
        COSC_READY OFFSET(5) NUMBITS(1) [],
        // Status indication of CAMP2
        CAMP2_READY OFFSET(3) NUMBITS(1) [],
        // Status of the value of CCM_REF_EN_B output of ccm
        REF_EN_B OFFSET(0) NUMBITS(1) []
    ],

    CBCDR [
        /// SEMC clock source select
        SEMC_CLK_SEL OFFSET(6) NUMBITS(1) [],
        /// SEMC alternative clock select
        SEMC_ALT_CLK_SEL OFFSET(7) NUMBITS(1) [],
        /// Divider for ipg podf.
        IPG_PODF OFFSET(8) NUMBITS(2) [],
        /// Divider for AHB PODF
        AHB_PODF OFFSET(10) NUMBITS(3) [],
        /// Post divider for SEMC clock
        SEMC_PODF OFFSET(16) NUMBITS(3) [],
        /// Selector for peripheral main clock
        PERIPH_CLK_SEL OFFSET(25) NUMBITS(1) [
            PrePeriphClkSel = 0,
            PeriphClk2Divided = 1
        ],
        /// Divider for periph_clk2_podf.
        PERIPH_CLK2_PODF OFFSET(27) NUMBITS(3) []
    ],

    CBCMR [
        /// Selector for lpspi clock multiplexer
        LPSPI_CLK_SEL OFFSET(4) NUMBITS(2) [],
        /// Selector for flexspi2 clock multiplexer
        FLEXSPI2_CLK_SEL OFFSET(8) NUMBITS(2) [],
        /// Selector for peripheral clk2 clock multiplexer
        PERIPH_CLK2_SEL OFFSET(12) NUMBITS(2) [
            PLL3Sw = 0,
            Oscillator = 1,
            PLL2Bypass = 2
        ],
        /// Selector for Trace clock multiplexer
        TRACE_CLK_SEL OFFSET(14) NUMBITS(2) [],
        /// Selector for pre_periph clock multiplexer
        PRE_PERIPH_CLK_SEL OFFSET(18) NUMBITS(2) [
            PLL2 = 0,
            PLL2_PFD2 = 1,
            PLL2_PFD0 = 2,
            PLL1 = 3
        ],
        /// Post-divider for LCDIF clock.
        LCDIF_PODF OFFSET(23) NUMBITS(3) [],
        /// Divider for LPSPI. Divider should be updated when output clock is gated.
        LPSPI_PODF OFFSET(26) NUMBITS(3) [],
        /// Divider for flexspi2 clock root.
        FLEXSPI2_PODF OFFSET(29) NUMBITS(3) []
    ],

    CCSR [
        PLL3_SW_CLK_SEL OFFSET(0) NUMBITS(1) []
    ],

    CSCMR1 [
        // Selector for the PERCLK clock multiplexer
        PERCLK_CLK_SEL OFFSET(6) NUMBITS(1) [
            // Derive clock from IPG CLK root
            IpgClockRoot = 0,
            // Derive clock from OSCILLATOR
            Oscillator = 1
        ],
        // Divider for PERCLK PODF
        //
        // 0 = divide by 1
        // 1 = divide by 2
        // 2 = divide by 3
        // ...
        // 63 = divide by 64
        PERCLK_PODF OFFSET(0) NUMBITS(6) []
    ],

    CSCDR1 [
        // Divider for trace clock
        TRACE_PODF OFFSET(25) NUMBITS(2) [],
        // Divider for usdhc2 clock
        USDHC2_PODF OFFSET(16) NUMBITS(3) [],
        // Divider for usdhc2 clock
        USDHC1_PODF OFFSET(11) NUMBITS(3) [],
        // Selector for the UART clock multiplexor
        UART_CLK_SEL OFFSET(6) NUMBITS(1) [
            Pll3 = 0,
            Oscillator = 1
        ],
        // Divider for uart clock podf
        UART_CLK_PODF OFFSET(0) NUMBITS(6) []
    ],

    CLPCR [
        WHATEVER OFFSET(2) NUMBITS(30) [],
        LPM OFFSET(0) NUMBITS(2) []
    ],

    // Supports al clock gate registers
    CCGR [
        CG15 OFFSET(30) NUMBITS(2) [],
        CG14 OFFSET(28) NUMBITS(2) [],
        CG13 OFFSET(26) NUMBITS(2) [],
        CG12 OFFSET(24) NUMBITS(2) [],
        CG11 OFFSET(22) NUMBITS(2) [],
        CG10 OFFSET(20) NUMBITS(2) [],
        CG9 OFFSET(18) NUMBITS(2) [],
        CG8 OFFSET(16) NUMBITS(2) [],
        CG7 OFFSET(14) NUMBITS(2) [],
        CG6 OFFSET(12) NUMBITS(2) [],
        CG5 OFFSET(10) NUMBITS(2) [],
        CG4 OFFSET(8) NUMBITS(2) [],
        CG3 OFFSET(6) NUMBITS(2) [],
        CG2 OFFSET(4) NUMBITS(2) [],
        CG1 OFFSET(2) NUMBITS(2) [],
        CG0 OFFSET(0) NUMBITS(2) []
    ],
];

const CCM_BASE: StaticRef<CcmRegisters> =
    unsafe { StaticRef::new(0x400FC000 as *const CcmRegisters) };

pub struct Ccm {
    registers: StaticRef<CcmRegisters>,
}

/// Describes the UART clock selection
#[repr(u32)]
pub enum UartClockSelection {
    /// PLL3 80M
    PLL3 = 0,
    /// osc_clk
    Oscillator = 1,
}

impl Ccm {
    pub const fn new() -> Ccm {
        Ccm {
            registers: CCM_BASE,
        }
    }

    pub fn set_low_power_mode(&self) {
        self.registers.clpcr.modify(CLPCR::LPM.val(0b00 as u32));
    }

    // Iomuxc_snvs clock
    pub fn is_enabled_iomuxc_snvs_clock(&self) -> bool {
        self.registers.ccgr[2].is_set(CCGR::CG2)
    }

    pub fn enable_iomuxc_snvs_clock(&self) {
        self.registers.ccgr[2].modify(CCGR::CG2.val(0b01 as u32));
        self.registers.ccgr[3].modify(CCGR::CG15.val(0b01 as u32));
    }

    pub fn disable_iomuxc_snvs_clock(&self) {
        self.registers.ccgr[2].modify(CCGR::CG2::CLEAR);
        self.registers.ccgr[3].modify(CCGR::CG15::CLEAR);
    }

    /// Iomuxc clock
    pub fn is_enabled_iomuxc_clock(&self) -> bool {
        self.registers.ccgr[4].is_set(CCGR::CG0) && self.registers.ccgr[4].is_set(CCGR::CG1)
    }

    pub fn enable_iomuxc_clock(&self) {
        self.registers.ccgr[4].modify(CCGR::CG0.val(0b01 as u32));
        self.registers.ccgr[4].modify(CCGR::CG1.val(0b01 as u32));
    }

    pub fn disable_iomuxc_clock(&self) {
        self.registers.ccgr[4].modify(CCGR::CG0::CLEAR);
        self.registers.ccgr[4].modify(CCGR::CG1::CLEAR)
    }

    /// GPIO1 clock
    pub fn is_enabled_gpio1_clock(&self) -> bool {
        self.registers.ccgr[1].is_set(CCGR::CG13)
    }

    pub fn enable_gpio1_clock(&self) {
        self.registers.ccgr[1].modify(CCGR::CG13.val(0b11 as u32))
    }

    pub fn disable_gpio1_clock(&self) {
        self.registers.ccgr[1].modify(CCGR::CG13::CLEAR)
    }

    /// GPIO2 clock
    pub fn is_enabled_gpio2_clock(&self) -> bool {
        self.registers.ccgr[0].is_set(CCGR::CG15)
    }

    pub fn enable_gpio2_clock(&self) {
        self.registers.ccgr[0].modify(CCGR::CG15.val(0b11 as u32))
    }

    pub fn disable_gpio2_clock(&self) {
        self.registers.ccgr[0].modify(CCGR::CG15::CLEAR)
    }

    /// GPIO3 clock
    pub fn is_enabled_gpio3_clock(&self) -> bool {
        self.registers.ccgr[2].is_set(CCGR::CG13)
    }

    pub fn enable_gpio3_clock(&self) {
        self.registers.ccgr[2].modify(CCGR::CG13.val(0b11 as u32))
    }

    pub fn disable_gpio3_clock(&self) {
        self.registers.ccgr[2].modify(CCGR::CG13::CLEAR)
    }

    /// GPIO4 clock
    pub fn is_enabled_gpio4_clock(&self) -> bool {
        self.registers.ccgr[3].is_set(CCGR::CG6)
    }

    pub fn enable_gpio4_clock(&self) {
        self.registers.ccgr[3].modify(CCGR::CG6.val(0b11 as u32))
    }

    pub fn disable_gpio4_clock(&self) {
        self.registers.ccgr[3].modify(CCGR::CG6::CLEAR)
    }

    /// GPIO5 clock
    pub fn is_enabled_gpio5_clock(&self) -> bool {
        self.registers.ccgr[1].is_set(CCGR::CG15)
    }

    pub fn enable_gpio5_clock(&self) {
        self.registers.ccgr[1].modify(CCGR::CG15.val(0b11 as u32))
    }

    pub fn disable_gpio5_clock(&self) {
        self.registers.ccgr[1].modify(CCGR::CG15::CLEAR)
    }

    // GPT1 clock
    pub fn is_enabled_gpt1_clock(&self) -> bool {
        self.registers.ccgr[1].is_set(CCGR::CG11)
    }

    pub fn enable_gpt1_clock(&self) {
        self.registers.ccgr[1].modify(CCGR::CG10.val(0b11 as u32));
        self.registers.ccgr[1].modify(CCGR::CG11.val(0b11 as u32));
    }

    pub fn disable_gpt1_clock(&self) {
        self.registers.ccgr[1].modify(CCGR::CG10::CLEAR);
        self.registers.ccgr[1].modify(CCGR::CG11::CLEAR);
    }

    // GPT2 clock
    pub fn is_enabled_gpt2_clock(&self) -> bool {
        self.registers.ccgr[0].is_set(CCGR::CG13)
    }

    pub fn enable_gpt2_clock(&self) {
        self.registers.ccgr[0].modify(CCGR::CG12.val(0b11 as u32));
        self.registers.ccgr[0].modify(CCGR::CG13.val(0b11 as u32));
    }

    pub fn disable_gpt2_clock(&self) {
        self.registers.ccgr[0].modify(CCGR::CG12::CLEAR);
        self.registers.ccgr[0].modify(CCGR::CG13::CLEAR);
    }

    // LPI2C1 clock
    pub fn is_enabled_lpi2c1_clock(&self) -> bool {
        self.registers.ccgr[2].is_set(CCGR::CG3)
    }

    pub fn enable_lpi2c1_clock(&self) {
        self.registers.ccgr[2].modify(CCGR::CG3.val(0b11 as u32));
    }

    pub fn disable_lpi2c1_clock(&self) {
        self.registers.ccgr[2].modify(CCGR::CG3::CLEAR);
    }

    // LPUART1 clock
    pub fn is_enabled_lpuart1_clock(&self) -> bool {
        self.registers.ccgr[5].is_set(CCGR::CG12)
    }

    pub fn enable_lpuart1_clock(&self) {
        self.registers.ccgr[5].modify(CCGR::CG12.val(0b11 as u32));
    }

    pub fn disable_lpuart1_clock(&self) {
        self.registers.ccgr[5].modify(CCGR::CG12::CLEAR);
    }

    // LPUART2 clock
    pub fn is_enabled_lpuart2_clock(&self) -> bool {
        self.registers.ccgr[0].is_set(CCGR::CG14)
    }

    pub fn enable_lpuart2_clock(&self) {
        self.registers.ccgr[0].modify(CCGR::CG14.val(0b11 as u32));
    }

    pub fn disable_lpuart2_clock(&self) {
        self.registers.ccgr[0].modify(CCGR::CG14::CLEAR);
    }

    // UART clock multiplexor
    pub fn is_enabled_uart_clock_mux(&self) -> bool {
        self.registers.cscdr1.is_set(CSCDR1::UART_CLK_SEL)
    }

    /// Set the UART clock selection
    ///
    /// Should only be called when *all* UART clock gates are disabled
    pub fn set_uart_clock_sel(&self, selection: UartClockSelection) {
        self.registers
            .cscdr1
            .modify(CSCDR1::UART_CLK_SEL.val(selection as u32));
    }

    /// Returns the UART clock selection
    pub fn uart_clock_sel(&self) -> UartClockSelection {
        use CSCDR1::UART_CLK_SEL::Value;
        match self.registers.cscdr1.read_as_enum(CSCDR1::UART_CLK_SEL) {
            Some(Value::Oscillator) => UartClockSelection::Oscillator,
            Some(Value::Pll3) => UartClockSelection::PLL3,
            None => unreachable!("Implemented all UART clock selections"),
        }
    }

    /// Set the UART clock divider
    ///
    /// `divider` is a value bound by [1, 2^6].
    pub fn set_uart_clock_podf(&self, divider: u32) {
        let divider = divider.max(1).min(1 << 6) - 1;
        self.registers
            .cscdr1
            .modify(CSCDR1::UART_CLK_PODF.val(divider as u32));
    }

    /// Returns the UART clock divider
    ///
    /// The return is a value bound by [1, 2^6].
    pub fn uart_clock_podf(&self) -> u32 {
        (self.registers.cscdr1.read(CSCDR1::UART_CLK_PODF) + 1) as u32
    }
    //
    // PERCLK
    //

    /// Returns the selection for the periodic clock
    pub fn perclk_sel(&self) -> PerclkClockSel {
        use CSCMR1::PERCLK_CLK_SEL::Value;
        match self.registers.cscmr1.read_as_enum(CSCMR1::PERCLK_CLK_SEL) {
            Some(Value::Oscillator) => PerclkClockSel::Oscillator,
            Some(Value::IpgClockRoot) => PerclkClockSel::IPG,
            None => unreachable!("Implemented all periodic clock selections"),
        }
    }

    /// Set the periodic clock selection
    pub fn set_perclk_sel(&self, sel: PerclkClockSel) {
        let sel = match sel {
            PerclkClockSel::IPG => CSCMR1::PERCLK_CLK_SEL::IpgClockRoot,
            PerclkClockSel::Oscillator => CSCMR1::PERCLK_CLK_SEL::Oscillator,
        };
        self.registers.cscmr1.modify(sel);
    }

    /// Set the periodic clock selection and divider
    ///
    /// This should only be called when all associated clock gates are disabled.
    ///
    /// `divider` will be clamped between 1 and 64.
    pub fn set_perclk_divider(&self, divider: u8) {
        let divider: u32 = divider.min(64).max(1).into();
        self.registers
            .cscmr1
            .modify(CSCMR1::PERCLK_PODF.val(divider - 1));
    }

    /// Returns the periodic clock divider, guaranteed to be non-zero
    pub fn perclk_divider(&self) -> u8 {
        (self.registers.cscmr1.read(CSCMR1::PERCLK_PODF) as u8) + 1
    }

    /// Blocks until *all* handshakes are complete
    fn wait_for_handshakes(&self) {
        while self.registers.cdhipr.get() != 0 {}
    }

    /// Set the ARM clock root divider
    ///
    /// The ARM clock divider is just after the PLL1 output.
    ///
    /// Clamps `divider` between [1, 8].
    pub fn set_arm_divider(&self, divider: u32) {
        let podf = divider.min(8).max(1) - 1;
        self.registers.cacrr.set(podf);
        self.wait_for_handshakes();
    }

    /// Returns the ARM clock root divider
    pub fn arm_divider(&self) -> u32 {
        self.registers.cacrr.get() + 1
    }

    /// Set the PERIPH_CLK2 divider
    ///
    /// Clamps `divider` between [1, 8].
    pub fn set_peripheral_clock2_divider(&self, divider: u32) {
        let podf = divider.min(8).max(1) - 1;
        self.registers
            .cbcdr
            .modify(CBCDR::PERIPH_CLK2_PODF.val(podf));
    }

    /// Returns the PERIPH_CLK2 divider
    pub fn peripheral_clock2_divider(&self) -> u32 {
        self.registers.cbcdr.read(CBCDR::PERIPH_CLK2_PODF) + 1
    }

    /// Set the AHB clock divider
    ///
    /// Clamps `divider` between [1, 8].
    pub fn set_ahb_divider(&self, divider: u32) {
        let podf = divider.min(8).max(1) - 1;
        self.registers.cbcdr.modify(CBCDR::AHB_PODF.val(podf));
        self.wait_for_handshakes();
    }

    /// Returns the AHB clock divider
    pub fn ahb_divider(&self) -> u32 {
        self.registers.cbcdr.read(CBCDR::AHB_PODF) + 1
    }

    /// Sets the IPG clock divider
    ///
    /// Clamps `divider` between [1, 4].
    pub fn set_ipg_divider(&self, divider: u32) {
        let podf = divider.min(4).max(1) - 1;
        self.registers.cbcdr.modify(CBCDR::IPG_PODF.val(podf));
    }

    /// Set the peripheral clock selection
    pub fn set_peripheral_clock_selection(&self, selection: PeripheralClockSelection) {
        let selection = match selection {
            PeripheralClockSelection::PrePeripheralClock => CBCDR::PERIPH_CLK_SEL::PrePeriphClkSel,
            PeripheralClockSelection::PeripheralClock2Divided => {
                CBCDR::PERIPH_CLK_SEL::PeriphClk2Divided
            }
        };
        self.registers.cbcdr.modify(selection);
        self.wait_for_handshakes();
    }

    /// Returns the peripheral clock selection
    pub fn peripheral_clock_selection(&self) -> PeripheralClockSelection {
        use CBCDR::PERIPH_CLK_SEL::Value;
        match self.registers.cbcdr.read_as_enum(CBCDR::PERIPH_CLK_SEL) {
            Some(Value::PrePeriphClkSel) => PeripheralClockSelection::PrePeripheralClock,
            Some(Value::PeriphClk2Divided) => PeripheralClockSelection::PeripheralClock2Divided,
            None => unreachable!(),
        }
    }

    /// Set the pre-peripheral clock selection
    pub fn set_pre_peripheral_clock_selection(&self, selection: PrePeripheralClockSelection) {
        let selection = match selection {
            PrePeripheralClockSelection::Pll2 => CBCMR::PRE_PERIPH_CLK_SEL::PLL2,
            PrePeripheralClockSelection::Pll2Pfd2 => CBCMR::PRE_PERIPH_CLK_SEL::PLL2_PFD2,
            PrePeripheralClockSelection::Pll2Pfd0 => CBCMR::PRE_PERIPH_CLK_SEL::PLL2_PFD0,
            PrePeripheralClockSelection::Pll1 => CBCMR::PRE_PERIPH_CLK_SEL::PLL1,
        };
        self.registers.cbcmr.modify(selection);
    }

    /// Returns the pre-peripheral clock selection
    pub fn pre_peripheral_clock_selection(&self) -> PrePeripheralClockSelection {
        use CBCMR::PRE_PERIPH_CLK_SEL::Value;
        match self.registers.cbcmr.read_as_enum(CBCMR::PRE_PERIPH_CLK_SEL) {
            Some(Value::PLL2) => PrePeripheralClockSelection::Pll2,
            Some(Value::PLL2_PFD0) => PrePeripheralClockSelection::Pll2Pfd0,
            Some(Value::PLL2_PFD2) => PrePeripheralClockSelection::Pll2Pfd2,
            Some(Value::PLL1) => PrePeripheralClockSelection::Pll1,
            None => unreachable!(),
        }
    }

    /// Set the peripheral clock 2 selection
    pub fn set_peripheral_clock2_selection(&self, selection: PeripheralClock2Selection) {
        let selection = match selection {
            PeripheralClock2Selection::Pll3 => CBCMR::PERIPH_CLK2_SEL::PLL3Sw,
            PeripheralClock2Selection::Oscillator => CBCMR::PERIPH_CLK2_SEL::Oscillator,
            PeripheralClock2Selection::Pll2Bypass => CBCMR::PERIPH_CLK2_SEL::PLL2Bypass,
        };
        self.registers.cbcmr.modify(selection);
        self.wait_for_handshakes();
    }

    /// Returns the selection for peripheral clock 2
    pub fn peripheral_clock2_selection(&self) -> PeripheralClock2Selection {
        use CBCMR::PERIPH_CLK2_SEL::Value;
        match self.registers.cbcmr.read_as_enum(CBCMR::PERIPH_CLK2_SEL) {
            Some(Value::PLL3Sw) => PeripheralClock2Selection::Pll3,
            Some(Value::PLL2Bypass) => PeripheralClock2Selection::Pll2Bypass,
            Some(Value::Oscillator) => PeripheralClock2Selection::Oscillator,
            None => unreachable!(),
        }
    }

    /// Enable the DCDC clock gate
    pub fn enable_dcdc_clock(&self) {
        self.registers.ccgr[6].modify(CCGR::CG3.val(0b11));
    }

    /// Disable the DCDC clock gate
    pub fn disable_dcdc_clock(&self) {
        self.registers.ccgr[6].modify(CCGR::CG3.val(0b00));
    }

    /// Indicates if the DCDC clock gate is enaled
    pub fn is_enabled_dcdc_clock(&self) -> bool {
        self.registers.ccgr[6].read(CCGR::CG3) != 0
    }

    /// Enable the DMA clock gate
    pub fn enable_dma_clock(&self) {
        self.registers.ccgr[5].modify(CCGR::CG3.val(0b11));
    }

    /// Disable the DMA clock gate
    pub fn disable_dma_clock(&self) {
        self.registers.ccgr[5].modify(CCGR::CG3.val(0b00));
    }

    /// Indicates if the DMA clock gate is enabled
    pub fn is_enabled_dma_clock(&self) -> bool {
        self.registers.ccgr[5].read(CCGR::CG3) != 0
    }
}

/// Clock selections for the main peripheral
#[derive(PartialEq, Eq)]
#[repr(u32)]
pub enum PeripheralClockSelection {
    /// Pre peripheral clock
    PrePeripheralClock,
    /// Peripheral clock 2, with some division
    PeripheralClock2Divided,
}

/// Pre-peripheral clock selections
#[derive(PartialEq, Eq)]
#[repr(u32)]
pub enum PrePeripheralClockSelection {
    Pll2,
    Pll2Pfd2,
    Pll2Pfd0,
    Pll1,
}

/// Peripheral clock 2 selection
#[derive(PartialEq, Eq)]
#[repr(u32)]
pub enum PeripheralClock2Selection {
    Pll3,
    Oscillator,
    Pll2Bypass,
}

enum ClockGate {
    CCGR0(HCLK0),
    CCGR1(HCLK1),
    CCGR2(HCLK2),
    CCGR3(HCLK3),
    CCGR4(HCLK4),
    CCGR5(HCLK5),
    CCGR6(HCLK6),
}

/// A peripheral clock gate
///
/// `PeripheralClock` provides a LPCG API for controlling peripheral
/// clock gates.
pub struct PeripheralClock<'a> {
    ccm: &'a Ccm,
    clock_gate: ClockGate,
}

impl<'a> PeripheralClock<'a> {
    pub const fn ccgr0(ccm: &'a Ccm, gate: HCLK0) -> Self {
        Self {
            ccm,
            clock_gate: ClockGate::CCGR0(gate),
        }
    }
    pub const fn ccgr1(ccm: &'a Ccm, gate: HCLK1) -> Self {
        Self {
            ccm,
            clock_gate: ClockGate::CCGR1(gate),
        }
    }
    pub const fn ccgr2(ccm: &'a Ccm, gate: HCLK2) -> Self {
        Self {
            ccm,
            clock_gate: ClockGate::CCGR2(gate),
        }
    }
    pub const fn ccgr3(ccm: &'a Ccm, gate: HCLK3) -> Self {
        Self {
            ccm,
            clock_gate: ClockGate::CCGR3(gate),
        }
    }
    pub const fn ccgr4(ccm: &'a Ccm, gate: HCLK4) -> Self {
        Self {
            ccm,
            clock_gate: ClockGate::CCGR4(gate),
        }
    }
    pub const fn ccgr5(ccm: &'a Ccm, gate: HCLK5) -> Self {
        Self {
            ccm,
            clock_gate: ClockGate::CCGR5(gate),
        }
    }
    pub const fn ccgr6(ccm: &'a Ccm, gate: HCLK6) -> Self {
        Self {
            ccm,
            clock_gate: ClockGate::CCGR6(gate),
        }
    }
}

pub enum HCLK0 {
    GPIO2,
    LPUART2,
    GPT2,
}

pub enum HCLK1 {
    GPIO1,
    GPIO5,
    GPT1, // and others ...
}
pub enum HCLK2 {
    LPI2C1,
    GPIO3,
    IOMUXCSNVS, // and others ...
}

pub enum HCLK3 {
    GPIO4,
    // and others ...
}

pub enum HCLK4 {
    IOMUXC,
    // and others ...
}

pub enum HCLK5 {
    LPUART1,
    DMA,
    // and others ...
}

pub enum HCLK6 {
    DCDC,
}

/// Periodic clock selection for GPTs and PITs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerclkClockSel {
    /// IPG clock selection (default)
    IPG,
    /// Crystal oscillator
    Oscillator,
}

impl ClockInterface for PeripheralClock<'_> {
    fn is_enabled(&self) -> bool {
        match self.clock_gate {
            ClockGate::CCGR0(ref v) => match v {
                HCLK0::GPIO2 => self.ccm.is_enabled_gpio2_clock(),
                HCLK0::GPT2 => self.ccm.is_enabled_gpt2_clock(),
                HCLK0::LPUART2 => self.ccm.is_enabled_lpuart2_clock(),
            },
            ClockGate::CCGR1(ref v) => match v {
                HCLK1::GPIO1 => self.ccm.is_enabled_gpio1_clock(),
                HCLK1::GPIO5 => self.ccm.is_enabled_gpio5_clock(),
                HCLK1::GPT1 => self.ccm.is_enabled_gpt1_clock(),
            },
            ClockGate::CCGR2(ref v) => match v {
                HCLK2::LPI2C1 => self.ccm.is_enabled_lpi2c1_clock(),
                HCLK2::GPIO3 => self.ccm.is_enabled_gpio3_clock(),
                HCLK2::IOMUXCSNVS => self.ccm.is_enabled_iomuxc_snvs_clock(),
            },
            ClockGate::CCGR3(ref v) => match v {
                HCLK3::GPIO4 => self.ccm.is_enabled_gpio4_clock(),
            },
            ClockGate::CCGR4(ref v) => match v {
                HCLK4::IOMUXC => self.ccm.is_enabled_iomuxc_clock(),
            },
            ClockGate::CCGR5(ref v) => match v {
                HCLK5::LPUART1 => self.ccm.is_enabled_lpuart1_clock(),
                HCLK5::DMA => self.ccm.is_enabled_dma_clock(),
            },
            ClockGate::CCGR6(ref v) => match v {
                HCLK6::DCDC => self.ccm.is_enabled_dcdc_clock(),
            },
        }
    }

    fn enable(&self) {
        match self.clock_gate {
            ClockGate::CCGR0(ref v) => match v {
                HCLK0::GPIO2 => self.ccm.enable_gpio2_clock(),
                HCLK0::GPT2 => self.ccm.enable_gpt2_clock(),
                HCLK0::LPUART2 => self.ccm.enable_lpuart2_clock(),
            },
            ClockGate::CCGR1(ref v) => match v {
                HCLK1::GPIO1 => self.ccm.enable_gpio1_clock(),
                HCLK1::GPIO5 => self.ccm.enable_gpio5_clock(),
                HCLK1::GPT1 => self.ccm.enable_gpt1_clock(),
            },
            ClockGate::CCGR2(ref v) => match v {
                HCLK2::LPI2C1 => self.ccm.enable_lpi2c1_clock(),
                HCLK2::GPIO3 => self.ccm.enable_gpio3_clock(),
                HCLK2::IOMUXCSNVS => self.ccm.enable_iomuxc_snvs_clock(),
            },
            ClockGate::CCGR3(ref v) => match v {
                HCLK3::GPIO4 => self.ccm.enable_gpio4_clock(),
            },
            ClockGate::CCGR4(ref v) => match v {
                HCLK4::IOMUXC => self.ccm.enable_iomuxc_clock(),
            },
            ClockGate::CCGR5(ref v) => match v {
                HCLK5::LPUART1 => self.ccm.enable_lpuart1_clock(),
                HCLK5::DMA => self.ccm.enable_dma_clock(),
            },
            ClockGate::CCGR6(ref v) => match v {
                HCLK6::DCDC => self.ccm.enable_dcdc_clock(),
            },
        }
    }

    fn disable(&self) {
        match self.clock_gate {
            ClockGate::CCGR0(ref v) => match v {
                HCLK0::GPIO2 => self.ccm.disable_gpio2_clock(),
                HCLK0::GPT2 => self.ccm.disable_gpt2_clock(),
                HCLK0::LPUART2 => self.ccm.disable_lpuart2_clock(),
            },
            ClockGate::CCGR1(ref v) => match v {
                HCLK1::GPIO1 => self.ccm.disable_gpio1_clock(),
                HCLK1::GPIO5 => self.ccm.disable_gpio5_clock(),
                HCLK1::GPT1 => self.ccm.disable_gpt1_clock(),
            },
            ClockGate::CCGR2(ref v) => match v {
                HCLK2::LPI2C1 => self.ccm.disable_lpi2c1_clock(),
                HCLK2::GPIO3 => self.ccm.disable_gpio3_clock(),
                HCLK2::IOMUXCSNVS => self.ccm.disable_iomuxc_snvs_clock(),
            },
            ClockGate::CCGR3(ref v) => match v {
                HCLK3::GPIO4 => self.ccm.disable_gpio4_clock(),
            },
            ClockGate::CCGR4(ref v) => match v {
                HCLK4::IOMUXC => self.ccm.disable_iomuxc_clock(),
            },
            ClockGate::CCGR5(ref v) => match v {
                HCLK5::LPUART1 => self.ccm.disable_lpuart1_clock(),
                HCLK5::DMA => self.ccm.disable_dma_clock(),
            },
            ClockGate::CCGR6(ref v) => match v {
                HCLK6::DCDC => self.ccm.disable_dcdc_clock(),
            },
        }
    }
}

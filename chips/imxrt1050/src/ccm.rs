use kernel::common::registers::{register_bitfields, ReadWrite, ReadOnly};
use kernel::common::StaticRef;
use kernel::ClockInterface;

// Clock Controller Module
#[repr(C)]
struct CcmRegisters {
    // CCM control register
    ccr: ReadWrite<u32, CCR::Register>,
    _reserved1: [u8; 4],
    // CCM status register
    csr: ReadOnly<u32, CSR::Register>,
    // CCM Clock Switcher Register
    ccsr: ReadWrite<u32, CCSR::Register>,
    // unimplemented
    _reserved2: [u8; 20],
    cscdr1: ReadWrite<u32, CSCDR1::Register>,
    _reserved3: [u8; 44],
    clpcr: ReadWrite<u32, CLPCR::Register>,
    _reserved4: [u8; 16],
    // clock gating register 0
    ccgr0: ReadWrite<u32, CCGR0::Register>,
    // clock gating register 1
    ccgr1: ReadWrite<u32, CCGR1::Register>,
    // clock gating register 2
    ccgr2: ReadWrite<u32, CCGR2::Register>,
    // clock gating register 3
    ccgr3: ReadWrite<u32, CCGR3::Register>,
    // clock gating register 4
    ccgr4: ReadWrite<u32, CCGR4::Register>,
    // clock gating register 5
    ccgr5: ReadWrite<u32, CCGR5::Register>,
    _reserved6: [u8; 8],
}

register_bitfields![u32,
    CCR [
    	/// Enable for REG_BYPASS_COUNTER
    	RBC_EN OFFSET(27) NUMBITS(1) [],

    	/// Counter for analog_reg_bypass
    	REG_BYPASS_COUNT OFFSET(21) NUMBITS(6) [],

    	/// On chip oscilator enable bit
    	COSC_EN OFFSET(12) NUMBITS(1) [],

        /// Oscilator ready counter value
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

    CCSR [
        PLL3_SW_CLK_SEL OFFSET(0) NUMBITS(1) []
    ],

    CSCDR1 [
        // Divider for trace clock
        TRACE_PODF OFFSET(25) NUMBITS(2) [],

        // Divider for usdhc2 clock
        USDHC2_PODF OFFSET(16) NUMBITS(3) [],

        // Divider for usdhc2 clock
        USDHC1_PODF OFFSET(11) NUMBITS(3) [],

        // Selector for the UART clock multiplexor
        UART_CLK_SEL OFFSET(6) NUMBITS(1) [],

        // Divider for uart clock podf
        UART_CLK_PODF OFFSET(0) NUMBITS(6) []
    ],

    CLPCR [
        WHATEVER OFFSET(2) NUMBITS(30) [],
        LPM OFFSET(0) NUMBITS(2) []
    ],

    CCGR0 [
        // gpio2_clocks (gpio2_clk_enable)
        CG15 OFFSET(30) NUMBITS(2) [],
   
        // lpuart2 clock (lpuart2_clk_enable)
        CG14 OFFSET(28) NUMBITS(2) [],

        // gpt2 serial clocks (gpt2_serial_clk_enable)
        CG13 OFFSET(26) NUMBITS(2) [],
        
        // gpt2 bus clocks (gpt2_bus_clk_enable)
        CG12 OFFSET(24) NUMBITS(2) [],

        // trace clock (trace_clk_enable)
        CG11 OFFSET(22) NUMBITS(2) [],

        // can2_serial clock (can2_serial_clk_enable)
        CG10 OFFSET(20) NUMBITS(2) [],

        // can2 clock (can2_clk_enable)
        CG9 OFFSET(18) NUMBITS(2) [],

        // can1_serial clock (can1_serial_clk_enable)
        CG8 OFFSET(16) NUMBITS(2) [],

        // can1 clock (can1_clk_enable)
        CG7 OFFSET(14) NUMBITS(2) [],
   
        // lpuart3 clock (lpuart3_clk_enable)
        CG6 OFFSET(12) NUMBITS(2) [],

        // dcp clock (dcp_clk_enable)
        CG5 OFFSET(10) NUMBITS(2) [],
        
        // sim_m or sim_main register access clock (sim_m_mainclk_r_enable)
        CG4 OFFSET(8) NUMBITS(2) [],

        // Reserved
        CG3 OFFSET(6) NUMBITS(2) [],

        // mqs clock ( mqs_hmclk_clock_enable)
        CG2 OFFSET(4) NUMBITS(2) [],

        // aips_tz2 clocks (aips_tz2_clk_enable)
        CG1 OFFSET(2) NUMBITS(2) [],

        // aips_tz1 clocks (aips_tz1_clk_enable)
        CG0 OFFSET(0) NUMBITS(2) []
    ],

    CCGR1 [
    	// gpio5 clock (gpio5_clk_enable)
    	CG15 OFFSET(30) NUMBITS(2) [],
   
   		// csu clock (csu_clk_enable)
    	CG14 OFFSET(28) NUMBITS(2) [],

		// gpio1 clock (gpio1_clk_enable)
    	CG13 OFFSET(26) NUMBITS(2) [],
		
		// lpuart4 clock (lpuart4_clk_enable)
    	CG12 OFFSET(24) NUMBITS(2) [],

    	// gpt1 serial clock (gpt_serial_clk_enable)
    	CG11 OFFSET(22) NUMBITS(2) [],

    	// gpt1 bus clock (gpt_clk_enable)
    	CG10 OFFSET(20) NUMBITS(2) [],

    	// semc_exsc clock (semc_exsc_clk_enable)
    	CG9 OFFSET(18) NUMBITS(2) [],

    	// adc1 clock (adc1_clk_enable)
    	CG8 OFFSET(16) NUMBITS(2) [],

    	// aoi2 clocks (aoi2_clk_enable)
    	CG7 OFFSET(14) NUMBITS(2) [],
   
   		// pit clocks (pit_clk_enable)
    	CG6 OFFSET(12) NUMBITS(2) [],

		// enet clock (enet_clk_enable)
    	CG5 OFFSET(10) NUMBITS(2) [],
		
		// adc2 clock (adc2_clk_enable)
    	CG4 OFFSET(8) NUMBITS(2) [],

    	// lpspi4 clocks (lpspi4_clk_enable)
    	CG3 OFFSET(6) NUMBITS(2) [],

    	// lpspi3 clocks (lpspi3_clk_enable)
    	CG2 OFFSET(4) NUMBITS(2) [],

    	// lpspi2 clocks (lpspi2_clk_enable)
    	CG1 OFFSET(2) NUMBITS(2) [],

    	// lpspi1 clocks (lpspi1_clk_enable)
    	CG0 OFFSET(0) NUMBITS(2) []
    ],

    CCGR2 [
        // pxp clocks (pxp_clk_enable)
        CG15 OFFSET(30) NUMBITS(2) [],
   
        // lcd clocks (lcd_clk_enable)
        CG14 OFFSET(28) NUMBITS(2) [],

        // gpio3 clock (gpio3_clk_enable)
        CG13 OFFSET(26) NUMBITS(2) [],
        
        // xbar2 clock (xbar2_clk_enable)
        CG12 OFFSET(24) NUMBITS(2) [],

        // xbar1 clock (xbar1_clk_enable)
        CG11 OFFSET(22) NUMBITS(2) [],

        // ipmux3 clock (ipmux3_clk_enable)
        CG10 OFFSET(20) NUMBITS(2) [],

        // ipmux2 clock (ipmux2_clk_enable)
        CG9 OFFSET(18) NUMBITS(2) [],

        // ipmux1 clock (ipmux1_clk_enable)
        CG8 OFFSET(16) NUMBITS(2) [],

        // xbar3 clock (xbar3_clk_enable)
        CG7 OFFSET(14) NUMBITS(2) [],
   
        // OCOTP_CTRL clock (iim_clk_enable)
        CG6 OFFSET(12) NUMBITS(2) [],

        // lpi2c3 clock (lpi2c3_clk_enable)
        CG5 OFFSET(10) NUMBITS(2) [],
        
        // lpi2c2 clock (lpi2c2_clk_enable)
        CG4 OFFSET(8) NUMBITS(2) [],

        // lpi2c1 clock (lpi2c1_clk_enable)
        CG3 OFFSET(6) NUMBITS(2) [],

        // iomuxc_snvs clock (iomuxc_snvs_clk_enable)
        CG2 OFFSET(4) NUMBITS(2) [],

        // csi clock (csi_clk_enable)
        CG1 OFFSET(2) NUMBITS(2) [],

        // ocram_exsc clock (ocram_exsc_clk_enable)
        CG0 OFFSET(0) NUMBITS(2) []
    ],

    CCGR3 [
        // iomuxc_snvs_gpr clock (iomuxc_snvs_gpr_clk_enable)
        CG15 OFFSET(30) NUMBITS(2) [],
   
        // ocram clock(ocram_clk_enable)
        CG14 OFFSET(28) NUMBITS(2) [],

        // acmp4 clocks (acmp4_clk_enable)
        CG13 OFFSET(26) NUMBITS(2) [],
        
        // acmp3 clocks (acmp3_clk_enable)
        CG12 OFFSET(24) NUMBITS(2) [],

        // acmp2 clocks (acmp2_clk_enable)
        CG11 OFFSET(22) NUMBITS(2) [],

        // acmp1 clocks (acmp1_clk_enable)
        CG10 OFFSET(20) NUMBITS(2) [],

        // flexram clock (flexram_clk_enable)
        CG9 OFFSET(18) NUMBITS(2) [],

        // wdog1 clock (wdog1_clk_enable)
        CG8 OFFSET(16) NUMBITS(2) [],

        // ewm clocks (ewm_clk_enable)
        CG7 OFFSET(14) NUMBITS(2) [],
   
        // gpio4 clock (gpio4_clk_enable)
        CG6 OFFSET(12) NUMBITS(2) [],

        // lcdif pix clock (lcdif_pix_clk_enable)
        CG5 OFFSET(10) NUMBITS(2) [],
        
        // aoi1 clock (aoi1_clk_enable)
        CG4 OFFSET(8) NUMBITS(2) [],

        // lpuart6 clock (lpuart6_clk_enable)
        CG3 OFFSET(6) NUMBITS(2) [],

        // semc clocks (semc_clk_enable)
        CG2 OFFSET(4) NUMBITS(2) [],

        // lpuart5 clock (lpuart5_clk_enable)
        CG1 OFFSET(2) NUMBITS(2) [],

        // flexio2 clocks (flexio2_clk_enable)
        CG0 OFFSET(0) NUMBITS(2) []
    ],

    CCGR4 [
        // enc4 clocks (enc4_clk_enable)
        CG15 OFFSET(30) NUMBITS(2) [],
   
        // enc2 clocks (enc2_clk_enable)
        CG14 OFFSET(28) NUMBITS(2) [],

        // enc2 clocks (enc2_clk_enable)
        CG13 OFFSET(26) NUMBITS(2) [],
        
        // enc1 clocks (enc1_clk_enable)
        CG12 OFFSET(24) NUMBITS(2) [],

        // pwm4 clocks (pwm4_clk_enable)
        CG11 OFFSET(22) NUMBITS(2) [],

        // pwm3 clocks (pwm3_clk_enable)
        CG10 OFFSET(20) NUMBITS(2) [],

        // pwm2 clocks (pwm2_clk_enable)
        CG9 OFFSET(18) NUMBITS(2) [],

        // pwm1 clocks (pwm1_clk_enable)
        CG8 OFFSET(16) NUMBITS(2) [],

        // sim_ems clocks (sim_ems_clk_enable)
        CG7 OFFSET(14) NUMBITS(2) [],
   
        // sim_m clocks (sim_m_clk_enable)
        CG6 OFFSET(12) NUMBITS(2) [],

        // tsc_dig clock (tsc_clk_enable)
        CG5 OFFSET(10) NUMBITS(2) [],
        
        // sim_m7 clock (sim_m7_clk_enable)
        CG4 OFFSET(8) NUMBITS(2) [],

        // bee clock(bee_clk_enable)
        CG3 OFFSET(6) NUMBITS(2) [],

        // iomuxc gpr clock (iomuxc_gpr_clk_enable)
        CG2 OFFSET(4) NUMBITS(2) [],

        // iomuxc clock (iomuxc_clk_enable)
        CG1 OFFSET(2) NUMBITS(2) [],

        // sim_m7 register access clock (sim_m7_mainclk_r_enable)
        CG0 OFFSET(0) NUMBITS(2) []
    ],

    CCGR5 [
         // snvs_lp clock (snvs_lp_clk_enable)
        CG15 OFFSET(30) NUMBITS(2) [],
   
        // snvs_hp clock (snvs_hp_clk_enable)
        CG14 OFFSET(28) NUMBITS(2) [],

        // lpuart7 clock (lpuart7_clk_enable)
        CG13 OFFSET(26) NUMBITS(2) [],
        
        // lpuart1 clock (lpuart1_clk_enable)
        CG12 OFFSET(24) NUMBITS(2) [],

        // sai3 clock (sai3_clk_enable)
        CG11 OFFSET(22) NUMBITS(2) [],

        // sai2 clock (sai2_clk_enable)
        CG10 OFFSET(20) NUMBITS(2) [],

        // sai1 clock (sai1_clk_enable)
        CG9 OFFSET(18) NUMBITS(2) [],

        // sim_main clock (sim_main_clk_enable)
        CG8 OFFSET(16) NUMBITS(2) [],

        // spdif clock (spdif_clk_enable)
        CG7 OFFSET(14) NUMBITS(2) [],
   
        // aipstz4 clocks (aips_tz4_clk_enable)
        CG6 OFFSET(12) NUMBITS(2) [],

        // wdog2 clock (wdog2_clk_enable)
        CG5 OFFSET(10) NUMBITS(2) [],
        
        // kpp clock (kpp_clk_enable)
        CG4 OFFSET(8) NUMBITS(2) [],

        // dma clock (dma_clk_enable)
        CG3 OFFSET(6) NUMBITS(2) [],

        // wdog3 clock (wdog3_clk_enable)
        CG2 OFFSET(4) NUMBITS(2) [],

        // flexio1 clock (flexio1_clk_enable)
        CG1 OFFSET(2) NUMBITS(2) [],

        // rom clock (rom_clk_enable)
        CG0 OFFSET(0) NUMBITS(2) []
    ]
];

const CCM_BASE: StaticRef<CcmRegisters> =
    unsafe { StaticRef::new(0x400FC000 as *const CcmRegisters) };

pub struct Ccm {
    registers: StaticRef<CcmRegisters>,
}

pub static mut CCM: Ccm = Ccm::new();

impl Ccm {
    const fn new() -> Ccm {
        Ccm {
            registers: CCM_BASE,
        }
    }

    pub fn set_low_power_mode(&self) {
        self.registers.clpcr.modify(CLPCR::LPM.val(0b00 as u32));
    }

    /// Iomuxc clock
    pub fn is_enabled_iomuxc_clock(&self) -> bool {
        self.registers.ccgr4.is_set(CCGR4::CG0) &&
        self.registers.ccgr4.is_set(CCGR4::CG1)
    }

    pub fn enable_iomuxc_clock(&self) {
        self.registers.ccgr4.modify(CCGR4::CG0.val(0b01 as u32));
        self.registers.ccgr4.modify(CCGR4::CG1.val(0b01 as u32));
    }

    pub fn disable_iomuxc_clock(&self) {
        self.registers.ccgr4.modify(CCGR4::CG0::CLEAR);
        self.registers.ccgr4.modify(CCGR4::CG1::CLEAR)
    }

    /// GPIO1 clock 
    pub fn is_enabled_gpio1_clock(&self) -> bool {
        self.registers.ccgr1.is_set(CCGR1::CG13)
    }

    pub fn enable_gpio1_clock(&self) {
        self.registers.ccgr1.modify(CCGR1::CG13.val(0b11 as u32))
    }

    pub fn disable_gpio1_clock(&self) {
        self.registers.ccgr1.modify(CCGR1::CG13::CLEAR)
    }

    /// GPIO2 clock 
    pub fn is_enabled_gpio2_clock(&self) -> bool {
        self.registers.ccgr0.is_set(CCGR0::CG15)
    }

    pub fn enable_gpio2_clock(&self) {
        self.registers.ccgr0.modify(CCGR0::CG15.val(0b11 as u32))
    }

    pub fn disable_gpio2_clock(&self) {
        self.registers.ccgr0.modify(CCGR0::CG15::CLEAR)
    }

    /// GPIO3 clock 
    pub fn is_enabled_gpio3_clock(&self) -> bool {
        self.registers.ccgr2.is_set(CCGR2::CG13)
    }

    pub fn enable_gpio3_clock(&self) {
        self.registers.ccgr2.modify(CCGR2::CG13.val(0b11 as u32))
    }

    pub fn disable_gpio3_clock(&self) {
        self.registers.ccgr2.modify(CCGR2::CG13::CLEAR)
    }

    /// GPIO4 clock 
    pub fn is_enabled_gpio4_clock(&self) -> bool {
        self.registers.ccgr3.is_set(CCGR3::CG6)
    }

    pub fn enable_gpio4_clock(&self) {
        self.registers.ccgr3.modify(CCGR3::CG6.val(0b11 as u32))
    }

    pub fn disable_gpio4_clock(&self) {
        self.registers.ccgr3.modify(CCGR3::CG6::CLEAR)
    }

    /// GPIO5 clock 
    pub fn is_enabled_gpio5_clock(&self) -> bool {
        self.registers.ccgr1.is_set(CCGR1::CG15)
    }

    pub fn enable_gpio5_clock(&self) {
        self.registers.ccgr1.modify(CCGR1::CG15.val(0b11 as u32))
    }

    pub fn disable_gpio5_clock(&self) {
        self.registers.ccgr1.modify(CCGR1::CG15::CLEAR)
    }

    // GPT1 clock 
    pub fn is_enabled_gpt1_clock(&self) -> bool {
        self.registers.ccgr1.is_set(CCGR1::CG11)
    }

    pub fn enable_gpt1_clock(&self) {
        self.registers.ccgr1.modify(CCGR1::CG10.val(0b11 as u32));
        self.registers.ccgr1.modify(CCGR1::CG11.val(0b11 as u32));
    }

    pub fn disable_gpt1_clock(&self) {
        self.registers.ccgr1.modify(CCGR1::CG10::CLEAR);
        self.registers.ccgr1.modify(CCGR1::CG11::CLEAR);
    }

    // LPI2C1 clock
    pub fn is_enabled_lpi2c1_clock(&self) -> bool {
        self.registers.ccgr2.is_set(CCGR2::CG3)
    }

    pub fn enable_lpi2c1_clock(&self) {
        self.registers.ccgr2.modify(CCGR2::CG3.val(0b11 as u32));
    }

    pub fn disable_lpi2c1_clock(&self) {
        self.registers.ccgr2.modify(CCGR2::CG3::CLEAR);
    }

    // LPUART1 clock
    pub fn is_enabled_lpuart1_clock(&self) -> bool {
        self.registers.ccgr5.is_set(CCGR5::CG12)
    }

    pub fn enable_lpuart1_clock(&self) {
        self.registers.ccgr5.modify(CCGR5::CG12.val(0b11 as u32));
    }

    pub fn disable_lpuart1_clock(&self) {
        self.registers.ccgr5.modify(CCGR5::CG12::CLEAR);
    }

    // UART clock multiplexor
    pub fn is_enabled_uart_clock_mux(&self) -> bool {
        self.registers.cscdr1.is_set(CSCDR1::UART_CLK_SEL)
    }

    pub fn enable_uart_clock_mux(&self) {
        self.registers.cscdr1.modify(CSCDR1::UART_CLK_SEL::SET);
    }

    pub fn disable_uart_clock_mux(&self) {
        self.registers.cscdr1.modify(CSCDR1::UART_CLK_SEL::CLEAR);
    }

    // UART_CLK_PODF
    pub fn is_enabled_uart_clock_podf(&self) -> bool {
        self.registers.cscdr1.is_set(CSCDR1::UART_CLK_PODF)
    }

    pub fn enable_uart_clock_podf(&self) {
        self.registers.cscdr1.modify(CSCDR1::UART_CLK_PODF.val(0b111111 as u32));
    }

    pub fn disable_uart_clock_podf(&self) {
        self.registers.cscdr1.modify(CSCDR1::UART_CLK_PODF::CLEAR);
    }

}

pub enum PeripheralClock {
    CCGR0(HCLK0),
    CCGR1(HCLK1),
    CCGR2(HCLK2),
    CCGR3(HCLK3),
    CCGR4(HCLK4),
    CCGR5(HCLK5),
}

pub enum HCLK0 {
    GPIO2,
}

pub enum HCLK1 {
    GPIO1,
    GPIO5,
    GPT1
    // and others ...
}
pub enum HCLK2 {
    LPI2C1,
    GPIO3
    // and others ...
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
    // and others ...
}

impl ClockInterface for PeripheralClock {
    fn is_enabled(&self) -> bool {
        match self {
            &PeripheralClock::CCGR0(ref v) => match v {
                HCLK0::GPIO2 => unsafe { CCM.is_enabled_gpio2_clock() },
            },
            &PeripheralClock::CCGR1(ref v) => match v {
                HCLK1::GPIO1 => unsafe { CCM.is_enabled_gpio1_clock() },
                HCLK1::GPIO5 => unsafe { CCM.is_enabled_gpio5_clock() },
                HCLK1::GPT1 => unsafe { CCM.is_enabled_gpt1_clock() },
            },
            &PeripheralClock::CCGR2(ref v) => match v {
                HCLK2::LPI2C1 => unsafe { CCM.is_enabled_lpi2c1_clock() },
                HCLK2::GPIO3 => unsafe { CCM.is_enabled_gpio3_clock() },
            },
            &PeripheralClock::CCGR3(ref v) => match v {
                HCLK3::GPIO4 => unsafe { CCM.is_enabled_gpio4_clock() },
            },
            &PeripheralClock::CCGR4(ref v) => match v {
                HCLK4::IOMUXC => unsafe { CCM.is_enabled_iomuxc_clock() },
            },
            &PeripheralClock::CCGR5(ref v) => match v {
                HCLK5::LPUART1 => unsafe { CCM.is_enabled_lpuart1_clock() },
            },
        }
    }

    fn enable(&self) {
        match self {
            &PeripheralClock::CCGR0(ref v) => match v {
                HCLK0::GPIO2 => unsafe {
                    CCM.enable_gpio2_clock();
                },
            },
            &PeripheralClock::CCGR1(ref v) => match v {
                HCLK1::GPIO1 => unsafe {
                    CCM.enable_gpio1_clock();
                },
                HCLK1::GPIO5 => unsafe {
                    CCM.enable_gpio5_clock();
                },
                HCLK1::GPT1 => unsafe {
                    CCM.enable_gpt1_clock();
                },
            },
            &PeripheralClock::CCGR2(ref v) => match v {
                HCLK2::LPI2C1 => unsafe {
                    CCM.enable_lpi2c1_clock();
                },
                HCLK2::GPIO3 => unsafe {
                    CCM.enable_gpio3_clock();
                },
            },
            &PeripheralClock::CCGR3(ref v) => match v {
                HCLK3::GPIO4 => unsafe {
                    CCM.enable_gpio4_clock();
                },
            }
            &PeripheralClock::CCGR4(ref v) => match v {
                HCLK4::IOMUXC => unsafe {
                    CCM.enable_iomuxc_clock();
                },
            },
            &PeripheralClock::CCGR5(ref v) => match v {
                HCLK5::LPUART1 => unsafe {
                    CCM.enable_lpuart1_clock();
                },
            },
        }
    }

    fn disable(&self) {
        match self {
            &PeripheralClock::CCGR0(ref v) => match v {
                HCLK0::GPIO2 => unsafe {
                    CCM.disable_gpio2_clock();
                },
            },
            &PeripheralClock::CCGR1(ref v) => match v {
                HCLK1::GPIO1 => unsafe {
                    CCM.disable_gpio1_clock();
                },
                HCLK1::GPIO5 => unsafe {
                    CCM.disable_gpio5_clock();
                },
                HCLK1::GPT1 => unsafe {
                    CCM.disable_gpt1_clock();
                },
            },
            &PeripheralClock::CCGR2(ref v) => match v {
                HCLK2::LPI2C1 => unsafe {
                    CCM.disable_lpi2c1_clock();
                },
                HCLK2::GPIO3 => unsafe {
                    CCM.disable_gpio3_clock();
                },
            },
            &PeripheralClock::CCGR3(ref v) => match v {
                HCLK3::GPIO4 => unsafe {
                    CCM.disable_gpio4_clock();
                },
            },
            &PeripheralClock::CCGR4(ref v) => match v {
                HCLK4::IOMUXC => unsafe {
                    CCM.disable_iomuxc_clock();
                },
            },
            &PeripheralClock::CCGR5(ref v) => match v {
                HCLK5::LPUART1 => unsafe {
                    CCM.disable_lpuart1_clock();
                },
            },
        }
    }
}

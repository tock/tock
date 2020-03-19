use kernel::common::registers::{register_bitfields, ReadWrite, ReadOnly};
use kernel::common::StaticRef;
use kernel::ClockInterface;

// Clock Controller Module
// CCGR1
// CCGR4
#[repr(C)]
struct CcmRegisters {
    // CCM control register
    ccr: ReadWrite<u32, CCR::Register>,
    _reserved1: [u8; 4],
    // CCM status register
    csr: ReadOnly<u32, CSR::Register>,
    // unimplemented
    _reserved2: [u8; 96],
    // clock gating register 1
    ccgr1: ReadWrite<u32, CCGR1::Register>,
    _reserved3: [u8; 12],
    // clock gating register 4
    ccgr4: ReadWrite<u32, CCGR4::Register>,
    _reserved4: [u8; 12],
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

    // Iomuxc clock
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

    // // GPIO1 clock 
    pub fn is_enabled_gpio1_clock(&self) -> bool {
        self.registers.ccgr1.is_set(CCGR1::CG13)
    }

    pub fn enable_gpio1_clock(&self) {
        self.registers.ccgr1.modify(CCGR1::CG13.val(0b11 as u32))
    }

    pub fn disable_gpio1_clock(&self) {
        self.registers.ccgr1.modify(CCGR1::CG13::CLEAR)
    }

}

// TBD - chiar nu stiu ce si cum la asta
pub enum CPUClock {
}

pub enum PeripheralClock {
    CCGR1(HCLK1),
    CCGR4(HCLK4)
}

pub enum HCLK1 {
    GPIO1
    // si restul ...
}

pub enum HCLK4 {
    IOMUXC,
    // si restul ...
}

impl ClockInterface for PeripheralClock {
    fn is_enabled(&self) -> bool {
        match self {
            &PeripheralClock::CCGR1(ref v) => match v {
                HCLK1::GPIO1 => unsafe { CCM.is_enabled_gpio1_clock() },
            },
            &PeripheralClock::CCGR4(ref v) => match v {
                HCLK4::IOMUXC => unsafe { CCM.is_enabled_iomuxc_clock() },
            },
        }
    }

    fn enable(&self) {
        match self {
            &PeripheralClock::CCGR1(ref v) => match v {
                HCLK1::GPIO1 => unsafe {
                    CCM.enable_gpio1_clock();
                },
            },
            &PeripheralClock::CCGR4(ref v) => match v {
                HCLK4::IOMUXC => unsafe {
                    CCM.enable_iomuxc_clock();
                },
            },
        }
    }

    fn disable(&self) {
        match self {
            &PeripheralClock::CCGR1(ref v) => match v {
                HCLK1::GPIO1 => unsafe {
                    CCM.disable_gpio1_clock();
                },
            },
            &PeripheralClock::CCGR4(ref v) => match v {
                HCLK4::IOMUXC => unsafe {
                    CCM.disable_iomuxc_clock();
                },
            },
        }
    }
}

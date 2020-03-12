use kernel::common::registers::{register_bitfields, ReadWrite, ReadOnly};
use kernel::common::StaticRef;
use kernel::ClockInterface;

/// Clock Controller Module
// CCGR1
// CCGR4
#[repr(C)]
struct RccRegisters {
    /// CCM control register
    ccr: ReadWrite<u32, CR::Register>,
    _reserved1: [u8; 4],
    /// CCM status register
    csr: ReadOnly<u32, CR::Register>,
    /// unimplemented
    _reserved1: [u8; 96],
    // clock gating register 1
    ccgr1: [u8, CR::Register],
    _reserved2: [u8; 12],
    // clock gating register 4
    ccgr4: [u8, CR::Register],
    _reserved3: [u8; 12],
}

register_bitfields![u32,
    ccr [
    	/// Enable for REG_BYPASS_COUNTER
    	RBC_EN(27) NUMBITS(1) [],

    	/// Counter for analog_reg_bypass
    	REG_BYPASS_COUNT(21) NUMBITS(6) [],

    	/// On chip oscilator enable bit
    	COSC_EN OFFSET(12) NUMBITS(1) [],

        /// Oscilator ready counter value
        OSCNT OFFSET(0) NUMBITS(8) []
    ],

    csr [
    	// Status indication of on board oscillator
    	COSC_READY OFFSET(5) NUMBITS(1) [],

    	// Status indication of CAMP2
    	CAMP2_READY OFFSET(3) NUMBITS(1) [],

    	// Status of the value of CCM_REF_EN_B output of ccm
    	REF_EN_B OFFSET(0) NUMBITS(1) []
    ],

    ccgr1 [
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
    ]
]

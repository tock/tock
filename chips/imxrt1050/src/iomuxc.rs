use kernel::common::registers::{register_bitfields, ReadWrite, ReadOnly};
use kernel::common::StaticRef;
use kernel::ClockInterface;

/// IOMUX Controller Module
#[repr(C)]
struct IomuxcRegisters {
    _reserved0: [u8; 204],
    /// MUX Control register for gpio_ad_b0_09
    sw_mux_ctl_pad_gpio_ad_b0_09: ReadWrite<u32, SW_MUX_CTL_PAD_GPIO_AD_B0_09::Register>,
    _reserved1: [u8; 8],
    sw_mux_ctl_pad_gpio_ad_b0_12: ReadWrite<u32, SW_MUX_CTL_PAD_GPIO_AD_B0_12::Register>,
    sw_mux_ctl_pad_gpio_ad_b0_13: ReadWrite<u32, SW_MUX_CTL_PAD_GPIO_AD_B0_13::Register>,   
    _reserved2: [u8; 476],
    // PAD Control register for gpio_ad_b0_09
    sw_pad_ctl_pad_gpio_ad_b0_09: ReadWrite<u32, SW_PAD_CTL_PAD_GPIO_AD_B0_09::Register>,
    _reserved3: [u8; 8],
    sw_pad_ctl_pad_gpio_ad_b0_12: ReadWrite<u32, SW_PAD_CTL_PAD_GPIO_AD_B0_12::Register>,
    sw_pad_ctl_pad_gpio_ad_b0_13: ReadWrite<u32, SW_PAD_CTL_PAD_GPIO_AD_B0_13::Register>,
}

register_bitfields![u32,
    SW_MUX_CTL_PAD_GPIO_AD_B0_09 [
    	// Software Input On Field
    	SION OFFSET(4) NUMBITS(1) [],

        // MUX Mode Select Field
        MUX_MODE OFFSET(0) NUMBITS(3) []
    ],

    SW_MUX_CTL_PAD_GPIO_AD_B0_12 [
        // Software Input On Field
        SION OFFSET(4) NUMBITS(1) [],

        // MUX Mode Select Field
        MUX_MODE OFFSET(0) NUMBITS(3) []
    ],

    SW_MUX_CTL_PAD_GPIO_AD_B0_13 [
        // Software Input On Field
        SION OFFSET(4) NUMBITS(1) [],

        // MUX Mode Select Field
        MUX_MODE OFFSET(0) NUMBITS(3) []
    ],

    SW_PAD_CTL_PAD_GPIO_AD_B0_09 [
        // Hyst. Enable Field
        HYS OFFSET(16) NUMBITS(1) [],

        // Pull Up / Down Config Field
        PUS OFFSET(14) NUMBITS(2) [],

        // Pull / Keep Select Field
        PUE OFFSET(13) NUMBITS(1) [],

        // Pull / Keep enable field
        PKE OFFSET(12) NUMBITS(1) [],

        // Open drain enable field
        ODE OFFSET(11) NUMBITS(1) [],

    	// Speed
    	SPEED OFFSET(6) NUMBITS(2) [],

    	// Drive Strength Field
    	DSE OFFSET(3) NUMBITS(3) [],

    	// Slew Rate Field
    	SRE OFFSET(0) NUMBITS(1) []
    ],

    SW_PAD_CTL_PAD_GPIO_AD_B0_12 [
        // Hyst. Enable Field
        HYS OFFSET(16) NUMBITS(1) [],

        // Pull Up / Down Config Field
        PUS OFFSET(14) NUMBITS(2) [],

        // Pull / Keep Select Field
        PUE OFFSET(13) NUMBITS(1) [],

        // Pull / Keep enable field
        PKE OFFSET(12) NUMBITS(1) [],

        // Open drain enable field
        ODE OFFSET(11) NUMBITS(1) [],

        // Speed
        SPEED OFFSET(6) NUMBITS(2) [],

        // Drive Strength Field
        DSE OFFSET(3) NUMBITS(3) [],

        // Slew Rate Field
        SRE OFFSET(0) NUMBITS(1) []
    ],

     SW_PAD_CTL_PAD_GPIO_AD_B0_13 [
        // Hyst. Enable Field
        HYS OFFSET(16) NUMBITS(1) [],

        // Pull Up / Down Config Field
        PUS OFFSET(14) NUMBITS(2) [],

        // Pull / Keep Select Field
        PUE OFFSET(13) NUMBITS(1) [],

        // Pull / Keep enable field
        PKE OFFSET(12) NUMBITS(1) [],

        // Open drain enable field
        ODE OFFSET(11) NUMBITS(1) [],

        // Speed
        SPEED OFFSET(6) NUMBITS(2) [],

        // Drive Strength Field
        DSE OFFSET(3) NUMBITS(3) [],

        // Slew Rate Field
        SRE OFFSET(0) NUMBITS(1) []
    ]
];

const IOMUXC_BASE: StaticRef<IomuxcRegisters> =
    unsafe { StaticRef::new(0x401F8014 as *const IomuxcRegisters) };

pub struct Iomuxc {
    registers: StaticRef<IomuxcRegisters>,
}

pub static mut IOMUXC: Iomuxc = Iomuxc::new();

impl Iomuxc {
    const fn new() -> Iomuxc {
        Iomuxc {
            registers: IOMUXC_BASE,
        }
    }

    // SW_MUX_CTL_PAD_GPIO_AD_B0_09
    fn is_enabled_sw_mux_ctl_pad_gpio_ad_b0_09_mode(&self) -> bool {
        self.registers.sw_mux_ctl_pad_gpio_ad_b0_09.is_set(SW_MUX_CTL_PAD_GPIO_AD_B0_09::MUX_MODE) 
    }

    fn enable_sw_mux_ctl_pad_gpio_ad_b0_09_alt5_mode(&self) {
        self.registers.sw_mux_ctl_pad_gpio_ad_b0_09.modify(SW_MUX_CTL_PAD_GPIO_AD_B0_09::MUX_MODE.val(0b101 as u32));
    }

    fn disable_sw_mux_ctl_pad_gpio_ad_b0_09_mode(&self) {
        self.registers.sw_mux_ctl_pad_gpio_ad_b0_09.modify(SW_MUX_CTL_PAD_GPIO_AD_B0_09::MUX_MODE::CLEAR);
    }

    // SW_PAD_CTL_PAD_GPIO_AD_B0_09 
    // fn is_enabled_sw_pad_ctl_pad_gpio_ad_b0_09_clock(&self) -> bool {
    //     self.registers.sw_pad_ctl_pad_gpio_ad_b0_09.is_set(CCGR1::CG13)
    // }

    // Enable GPIO1 on pin 9
    pub fn enable_gpio1_09(&self) {
        self.registers.sw_pad_ctl_pad_gpio_ad_b0_09.modify(SW_PAD_CTL_PAD_GPIO_AD_B0_09::DSE.val(0b110 as u32));
        self.registers.sw_pad_ctl_pad_gpio_ad_b0_09.modify(SW_PAD_CTL_PAD_GPIO_AD_B0_09::SPEED.val(0b10 as u32));
        self.registers.sw_pad_ctl_pad_gpio_ad_b0_09.modify(SW_PAD_CTL_PAD_GPIO_AD_B0_09::PKE::SET);   
    }

    pub fn enable_lpuart1_tx(&self) {
        self.registers.sw_mux_ctl_pad_gpio_ad_b0_12.modify(SW_MUX_CTL_PAD_GPIO_AD_B0_12::SION::CLEAR);
        self.registers.sw_mux_ctl_pad_gpio_ad_b0_12.modify(SW_MUX_CTL_PAD_GPIO_AD_B0_12::MUX_MODE.val(0b010 as u32));
    }

    pub fn enable_lpuart1_rx(&self) {
        self.registers.sw_mux_ctl_pad_gpio_ad_b0_13.modify(SW_MUX_CTL_PAD_GPIO_AD_B0_13::SION::CLEAR);
        self.registers.sw_mux_ctl_pad_gpio_ad_b0_13.modify(SW_MUX_CTL_PAD_GPIO_AD_B0_13::MUX_MODE.val(0b010 as u32));
    }

    pub fn set_pin_config_lpuart1(&self) {
        self.registers.sw_pad_ctl_pad_gpio_ad_b0_12.modify(SW_PAD_CTL_PAD_GPIO_AD_B0_12::DSE.val(0b110 as u32));
        self.registers.sw_pad_ctl_pad_gpio_ad_b0_12.modify(SW_PAD_CTL_PAD_GPIO_AD_B0_12::SPEED.val(0b10 as u32));
        self.registers.sw_pad_ctl_pad_gpio_ad_b0_12.modify(SW_PAD_CTL_PAD_GPIO_AD_B0_12::PKE::SET); 

        self.registers.sw_pad_ctl_pad_gpio_ad_b0_13.modify(SW_PAD_CTL_PAD_GPIO_AD_B0_13::DSE.val(0b110 as u32));
        self.registers.sw_pad_ctl_pad_gpio_ad_b0_13.modify(SW_PAD_CTL_PAD_GPIO_AD_B0_13::SPEED.val(0b10 as u32));
        self.registers.sw_pad_ctl_pad_gpio_ad_b0_13.modify(SW_PAD_CTL_PAD_GPIO_AD_B0_13::PKE::SET); 
    }
    // fn disable_gpio1_clock(&self) {
    //     self.registers.ccgr1.modify(CCGR1::CG13::CLEAR)
    // }

}

// 
// pub enum CPUClock {
// }

// pub enum PeripheralClock {
//     CCGR1(HCLK1),
//     CCGR4(HCLK4)
// }

// pub enum HCLK1 {
//     GPIO1
//     // si restul ...
// }

// pub enum HCLK4 {
//     IOMUXC,
//     // si restul ...
// }

// impl ClockInterface for PeripheralClock {
//     fn is_enabled(&self) -> bool {
//         match self {
//             &PeripheralClock::CCGR1(ref v) => match v {
//                 HCLK1::GPIO1 => unsafe { CCM.is_enabled_gpio1_clock() },
//             },
//             &PeripheralClock::CCGR4(ref v) => match v {
//                 HCLK4::IOMUXC => unsafe { CCM.is_enabled_iomuxc_clock() },
//             },
//         }
//     }

//     fn enable(&self) {
//         match self {
//             &PeripheralClock::CCGR1(ref v) => match v {
//                 HCLK1::GPIO1 => unsafe {
//                     CCM.enable_gpio1_clock();
//                 },
//             },
//             &PeripheralClock::CCGR4(ref v) => match v {
//                 HCLK4::IOMUXC => unsafe {
//                     CCM.enable_iomuxc_clock();
//                 },
//             },
//         }
//     }

//     fn disable(&self) {
//         match self {
//             &PeripheralClock::CCGR1(ref v) => match v {
//                 HCLK1::GPIO1 => unsafe {
//                     CCM.disable_gpio1_clock();
//                 },
//             },
//             &PeripheralClock::CCGR4(ref v) => match v {
//                 HCLK4::IOMUXC => unsafe {
//                     CCM.disable_iomuxc_clock();
//                 },
//             },
//         }
//     }
// }

use kernel::common::registers::{register_bitfields, ReadWrite};
use kernel::common::StaticRef;
use kernel::ClockInterface;

/// Reset and clock control
#[repr(C)]
struct RccRegisters {
    /// clock control register
    cr: ReadWrite<u32, CR::Register>,
    /// clock configuration register
    cfgr: ReadWrite<u32, CFGR::Register>,
    /// clock interrupt register
    cir: ReadWrite<u32, CIR::Register>,
    /// APB2 peripheral reset register
    apb2rstr: ReadWrite<u32, APB2RSTR::Register>,
    /// APB1 peripheral reset register
    apb1rstr: ReadWrite<u32, APB1RSTR::Register>,
    /// AHB peripheral clock register
    ahbenr: ReadWrite<u32, AHBENR::Register>,
    /// APB2 peripheral clock enable register
    apb2enr: ReadWrite<u32, APB2ENR::Register>,
    /// APB1 peripheral clock enable register
    apb1enr: ReadWrite<u32, APB1ENR::Register>,
    /// Backup domain control register
    bdcr: ReadWrite<u32, BDCR::Register>,
    /// clock control & status register
    csr: ReadWrite<u32, CSR::Register>,
    /// AHB peripheral reset register
    ahbrstr: ReadWrite<u32, AHBRSTR::Register>,
    /// clocks configuration register 2
    cfgr2: ReadWrite<u32, CFGR2::Register>,
    /// clocks configuration register 3
    cfgr3: ReadWrite<u32, CFGR3::Register>,
}

register_bitfields![u32,
    CR [
        /// Main PLL (PLL) clock ready flag
        PLLRDY OFFSET(25) NUMBITS(1) [],
        /// Main PLL (PLL) enable
        PLLON OFFSET(24) NUMBITS(1) [],
        /// Clock security system enable
        CSSON OFFSET(19) NUMBITS(1) [],
        /// HSE clock bypass
        HSEBYP OFFSET(18) NUMBITS(1) [],
        /// HSE clock ready flag
        HSERDY OFFSET(17) NUMBITS(1) [],
        /// HSE clock enable
        HSEON OFFSET(16) NUMBITS(1) [],
        /// Internal high-speed clock calibration
        HSICAL OFFSET(8) NUMBITS(8) [],
        /// Internal high-speed clock trimming
        HSITRIM OFFSET(3) NUMBITS(5) [],
        /// Internal high-speed clock ready flag
        HSIRDY OFFSET(1) NUMBITS(1) [],
        /// Internal high-speed clock enable
        HSION OFFSET(0) NUMBITS(1) []
    ],
    CFGR [
        /// Do not divide PLL to MCO
        PLLNODIV OFFSET(31) NUMBITS(1) [],
        /// MCO prescaler
        MCOPRE OFFSET(28) NUMBITS(3) [],
        /// Microcontorller clock output flag
        MCOF OFFSET(28) NUMBITS(1) [],
        /// Microcontroller clock output
        MCO OFFSET(24) NUMBITS(3) [],
        /// I2S clock selection
        #[cfg(feature = "stm32f303vct6")]
        I2SSRC OFFSET(23) NUMBITS(1) [],
        /// USB prescaler
        USBPRE OFFSET(22) NUMBITS(1) [],
        /// PLL multiplication factor
        PLLMUL OFFSET(18) NUMBITS(4) [],
        /// HSE divider for PLL input clock
        PLLXTPRE OFFSET(17) NUMBITS(1) [],
        /// PLL entry clock source
        #[cfg(feature = "stm32f303vct6")]
        PLLSRC OFFSET(16) NUMBITS(1) [],
        /// APB high-speed prescaler (APB2)
        PPRE2 OFFSET(11) NUMBITS(3) [],
        /// APB Low speed prescaler (APB1)
        PPRE1 OFFSET(8) NUMBITS(3) [],
        /// AHB prescaler
        HPRE OFFSET(4) NUMBITS(4) [],
        /// System clock switch status
        SWS1 OFFSET(3) NUMBITS(1) [],
        /// System clock switch status
        SWS0 OFFSET(2) NUMBITS(1) [],
        /// System clock switch
        SW1 OFFSET(1) NUMBITS(1) [],
        /// System clock switch
        SW0 OFFSET(0) NUMBITS(1) []
    ],
    CIR [
        /// Clock security system interrupt clear
        CSSC OFFSET(23) NUMBITS(1) [],
        /// Main PLL(PLL) ready interrupt clear
        PLLRDYC OFFSET(20) NUMBITS(1) [],
        /// HSE ready interrupt clear
        HSERDYC OFFSET(19) NUMBITS(1) [],
        /// HSI ready interrupt clear
        HSIRDYC OFFSET(18) NUMBITS(1) [],
        /// LSE ready interrupt clear
        LSERDYC OFFSET(17) NUMBITS(1) [],
        /// LSI ready interrupt clear
        LSIRDYC OFFSET(16) NUMBITS(1) [],
        /// Main PLL (PLL) ready interrupt enable
        PLLRDYIE OFFSET(12) NUMBITS(1) [],
        /// HSE ready interrupt enable
        HSERDYIE OFFSET(11) NUMBITS(1) [],
        /// HSI ready interrupt enable
        HSIRDYIE OFFSET(10) NUMBITS(1) [],
        /// LSE ready interrupt enable
        LSERDYIE OFFSET(9) NUMBITS(1) [],
        /// LSI ready interrupt enable
        LSIRDYIE OFFSET(8) NUMBITS(1) [],
        /// Clock security system interrupt flag
        CSSF OFFSET(7) NUMBITS(1) [],
        /// PLLSAI ready interrupt flag
        PLLSAIRDYF OFFSET(6) NUMBITS(1) [],
        /// PLLI2S ready interrupt flag
        PLLI2SRDYF OFFSET(5) NUMBITS(1) [],
        /// Main PLL (PLL) ready interrupt flag
        PLLRDYF OFFSET(4) NUMBITS(1) [],
        /// HSE ready interrupt flag
        HSERDYF OFFSET(3) NUMBITS(1) [],
        /// HSI ready interrupt flag
        HSIRDYF OFFSET(2) NUMBITS(1) [],
        /// LSE ready interrupt flag
        LSERDYF OFFSET(1) NUMBITS(1) [],
        /// LSI ready interrupt flag
        LSIRDYF OFFSET(0) NUMBITS(1) []
    ],
    APB2RSTR [
        /// TIM20 reset
        TIM20RST OFFSET(20) NUMBITS(1) [],
        /// TIM17 reset
        TIM17RST OFFSET(18) NUMBITS(1) [],
        /// TIM16 reset
        TIM16RST OFFSET(17) NUMBITS(1) [],
        /// TIM15 reset
        TIM15RST OFFSET(16) NUMBITS(1) [],
        /// SPI4 reset
        SPI4RST OFFSET(15) NUMBITS(1) [],
        /// USART1 reset
        USART1RST OFFSET(14) NUMBITS(1) [],
        /// TIM8 reset
        TIM8RST OFFSET(13) NUMBITS(1) [],
        /// SPI 1 reset
        SPI1RST OFFSET(12) NUMBITS(1) [],
        /// TIM1 reset
        TIM1RST OFFSET(11) NUMBITS(1) [],
        /// SYSCFG, Comparators and operational amplifiers reset
        SYSCFGRST OFFSET(0) NUMBITS(1) []
    ],
    APB1RSTR [
        /// TIM2 reset
        TIM2RST OFFSET(0) NUMBITS(1) [],
        /// TIM3 reset
        TIM3RST OFFSET(1) NUMBITS(1) [],
        /// TIM4 reset
        TIM4RST OFFSET(2) NUMBITS(1) [],
        /// TIM6 reset
        TIM6RST OFFSET(4) NUMBITS(1) [],
        /// TIM7 reset
        TIM7RST OFFSET(5) NUMBITS(1) [],
        /// Window watchdog reset
        WWDGRST OFFSET(11) NUMBITS(1) [],
        /// SPI 2 reset
        SPI2RST OFFSET(14) NUMBITS(1) [],
        /// SPI 3 reset
        SPI3RST OFFSET(15) NUMBITS(1) [],
        /// USART 2 reset
        UART2RST OFFSET(17) NUMBITS(1) [],
        /// USART 3 reset
        UART3RST OFFSET(18) NUMBITS(1) [],
        /// USART 4 reset
        UART4RST OFFSET(19) NUMBITS(1) [],
        /// USART 5 reset
        UART5RST OFFSET(20) NUMBITS(1) [],
        /// I2C 1 reset
        I2C1RST OFFSET(21) NUMBITS(1) [],
        /// I2C 2 reset
        I2C2RST OFFSET(22) NUMBITS(1) [],
        /// USB reset
        USBRST OFFSET(23) NUMBITS(1) [],
        /// CAN reset
        CANRST OFFSET(25) NUMBITS(1) [],
        /// DAC2 reset
        DAC2RST OFFSET(26) NUMBITS(1) [],
        /// Power interface reset
        PWRRST OFFSET(28) NUMBITS(1) [],
        /// DAC1 reset
        DAC1RST OFFSET(29) NUMBITS(1) [],
        /// I2C 3 reset
        I2C3RST OFFSET(30) NUMBITS(1) []
    ],
    AHBENR [
        /// ADC3 and ADC4 enable
        #[cfg(feature = "stm32f303vct6")]
        ADC34EN OFFSET(29) NUMBITS(1) [],
        /// ADC1 and ADC2 enable
        ADC12EN OFFSET(28) NUMBITS(1) [],
        /// Touch sensing controller clock enable
        TSCEN OFFSET(24) NUMBITS(1) [],
        /// IO port F clock enable
        IOPFEN OFFSET(22) NUMBITS(1) [],
        /// IO port E clock enable
        IOPEEN OFFSET(21) NUMBITS(1) [],
        /// IO port D clock enable
        IOPDEN OFFSET(20) NUMBITS(1) [],
        /// IO port C clock enable
        IOPCEN OFFSET(19) NUMBITS(1) [],
        /// IO port B clock enable
        IOPBEN OFFSET(18) NUMBITS(1) [],
        /// IO port A clock enable
        IOPAEN OFFSET(17) NUMBITS(1) [],
        /// CRC clock enable
        CRCEN OFFSET(6) NUMBITS(1) [],
        /// FLITF clock enable
        FLITFEN OFFSET(4) NUMBITS(1) [],
        /// SRAM interface clock enable
        SRAMEN OFFSET(2) NUMBITS(1) [],
        /// DMA2 clock enable
        #[cfg(feature = "stm32f303vct6")]
        DMA2EN OFFSET(1) NUMBITS(1) [],
        /// DMA1 clock enable
        DMA1EN OFFSET(0) NUMBITS(1) []
    ],
    APB2ENR [
        /// TIM20 clock enable
        TIM20EN OFFSET(20) NUMBITS(1) [],
        /// TIM17 clock enable
        TIM17EN OFFSET(18) NUMBITS(1) [],
        /// TIM16 clock enable
        TIM16EN OFFSET(17) NUMBITS(1) [],
        /// TIM15 clock enable
        TIM15EN OFFSET(16) NUMBITS(1) [],
        /// SPI4 clock enable
        SPI4EN OFFSET(15) NUMBITS(1) [],
        /// USART1 clock enable
        USART1EN OFFSET(14) NUMBITS(1) [],
        /// TIM8 clock enable
        TIM8EN OFFSET(13) NUMBITS(1) [],
        /// SPI1 clock enable
        SPI1EN OFFSET(12) NUMBITS(1) [],
        /// TIM1 clock enable
        TIM1EN OFFSET(11) NUMBITS(1) [],
        /// SYSCFG clock enable
        SYSCFGEN OFFSET(0) NUMBITS(1) []
    ],
    APB1ENR [
        /// TIM2 clock enable
        TIM2EN OFFSET(0) NUMBITS(1) [],
        /// TIM3 clock enable
        TIM3EN OFFSET(1) NUMBITS(1) [],
        /// TIM4 clock enable
        TIM4EN OFFSET(2) NUMBITS(1) [],
        /// TIM6 clock enable
        TIM6EN OFFSET(4) NUMBITS(1) [],
        /// TIM7 clock enable
        TIM7EN OFFSET(5) NUMBITS(1) [],
        /// Window watchdog clock enable
        WWDGEN OFFSET(11) NUMBITS(1) [],
        /// SPI2 clock enable
        SPI2EN OFFSET(14) NUMBITS(1) [],
        /// SPI3 clock enable
        SPI3EN OFFSET(15) NUMBITS(1) [],
        /// USART 2 clock enable
        USART2EN OFFSET(17) NUMBITS(1) [],
        /// USART3 clock enable
        USART3EN OFFSET(18) NUMBITS(1) [],
        /// UART4 clock enable
        UART4EN OFFSET(19) NUMBITS(1) [],
        /// UART5 clock enable
        UART5EN OFFSET(20) NUMBITS(1) [],
        /// I2C1 clock enable
        I2C1EN OFFSET(21) NUMBITS(1) [],
        /// I2C2 clock enable
        I2C2EN OFFSET(22) NUMBITS(1) [],
        /// USB clock enable
        #[cfg(feature = "stm32f303vct6")]
        USBEN OFFSET(23) NUMBITS(1) [],
        /// I2CFMP1 clock enable
        I2CFMP1EN OFFSET(24) NUMBITS(1) [],
        /// CAN clock enable
        CANEN OFFSET(25) NUMBITS(1) [],
        /// DAC 2 clock enable
        DAC2EN OFFSET(26) NUMBITS(1) [],
        /// Power interface clock enable
        PWREN OFFSET(28) NUMBITS(1) [],
        /// DAC 1 clock enable
        DAC1EN OFFSET(29) NUMBITS(1) [],
        /// I2C 3 interface clock enable
        I2C3EN OFFSET(30) NUMBITS(1) []
    ],
    BDCR [
        /// Backup domain software reset
        BDRST OFFSET(16) NUMBITS(1) [],
        /// RTC clock enable
        RTCEN OFFSET(15) NUMBITS(1) [],
        /// RTC clock source selection
        RTCSEL OFFSET(8) NUMBITS(2) [],
        /// External low-speed oscillator mode
        LSEDRV OFFSET(3) NUMBITS(2) [],
        /// External low-speed oscillator bypass
        LSEBYP OFFSET(2) NUMBITS(1) [],
        /// External low-speed oscillator ready
        LSERDY OFFSET(1) NUMBITS(1) [],
        /// External low-speed oscillator enable
        LSEON OFFSET(0) NUMBITS(1) []
    ],
    CSR [
        /// Low-power reset flag
        LPWRRSTF OFFSET(31) NUMBITS(1) [],
        /// Window watchdog reset flag
        WWDGRSTF OFFSET(30) NUMBITS(1) [],
        /// Independent watchdog reset flag
        IWDGRSTF OFFSET(29) NUMBITS(1) [],
        /// Software reset flag
        SFTRSTF OFFSET(28) NUMBITS(1) [],
        /// POR/PDR reset flag
        PORRSTF OFFSET(27) NUMBITS(1) [],
        /// PIN reset flag
        PINRSTF OFFSET(26) NUMBITS(1) [],
        /// Option byte loader reset flag
        OBLRSTF OFFSET(25) NUMBITS(1) [],
        /// Remove reset flag
        RMVF OFFSET(24) NUMBITS(1) [],
        /// Reset flag of the 1.8 V domain
        V18PWRRSTF OFFSET(23) NUMBITS(1) [],
        /// Internal low-speed oscillator ready
        LSIRDY OFFSET(1) NUMBITS(1) [],
        /// Internal low-speed oscillator enable
        LSION OFFSET(0) NUMBITS(1) []
    ],
    AHBRSTR [
        /// ADC3 and ADC4
        #[cfg(feature = "stm32f303vct6")]
        ADC34RST OFFSET(29) NUMBITS(1) [],
        /// ADC1 and ADC2 reset
        ADC12RST OFFSET(28) NUMBITS(1) [],
        /// Touch sensing controller reset
        TSCRST OFFSET(24) NUMBITS(1) [],
        /// IO port F reset
        IOPFRST OFFSET(22) NUMBITS(1) [],
        /// IO port E reset
        IOPERST OFFSET(21) NUMBITS(1) [],
        /// IO port D reset
        IOPDRST OFFSET(20) NUMBITS(1) [],
        /// IO port C reset
        IOPCRST OFFSET(19) NUMBITS(1) [],
        /// IO port B reset
        IOPBRST OFFSET(18) NUMBITS(1) [],
        /// IO port A reset
        IOPARST OFFSET(17) NUMBITS(1) []
    ],
    CFGR2 [
        /// ADC34 prescaler
        #[cfg(feature = "stm32f303vct6")]
        ADC34PRES OFFSET(9) NUMBITS(5) [],
        /// ADC12 prescaler
        ADC12PRES OFFSET(4) NUMBITS(5) [],
        /// PREDIV division factor
        PREDIV OFFSET(0) NUMBITS(4) []
    ],
    CFGR3 [
        /// USART5 clock source selection
        #[cfg(feature = "stm32f303vct6")]
        USART5SW OFFSET(22) NUMBITS(2) [],
        /// USART4 clock source selection
        #[cfg(feature = "stm32f303vct6")]
        USART4SW OFFSET(20) NUMBITS(2) [],
        /// USART2 clock source selection
        USART2SW OFFSET(16) NUMBITS(2) [],
        /// Timer8 clock source selection
        #[cfg(feature = "stm32f303vct6")]
        TIM8SW OFFSET(9) NUMBITS(1) [],
        /// Timer1 clock source selection
        TIM1SW OFFSET(8) NUMBITS(1) [],
        /// I2C2 clock source selection
        #[cfg(feature = "stm32f303vct6")]
        I2C2SW OFFSET(5) NUMBITS(1) [],
        /// I2C1 clock source selection
        I2C1SW OFFSET(4) NUMBITS(1) [],
        /// USART1 clock source selection
        USART1SW OFFSET(0) NUMBITS(2) []
    ]
];

const RCC_BASE: StaticRef<RccRegisters> =
    unsafe { StaticRef::new(0x40021000 as *const RccRegisters) };

pub struct Rcc {
    registers: StaticRef<RccRegisters>,
}

pub static mut RCC: Rcc = Rcc::new();

impl Rcc {
    const fn new() -> Rcc {
        Rcc {
            registers: RCC_BASE,
        }
    }

    // TIM2 clock

    fn is_enabled_tim2_clock(&self) -> bool {
        self.registers.apb1enr.is_set(APB1ENR::TIM2EN)
    }

    fn enable_tim2_clock(&self) {
        self.registers.apb1enr.modify(APB1ENR::TIM2EN::SET)
    }

    fn disable_tim2_clock(&self) {
        self.registers.apb1enr.modify(APB1ENR::TIM2EN::CLEAR)
    }

    // SYSCFG clock

    fn is_enabled_syscfg_clock(&self) -> bool {
        self.registers.apb2enr.is_set(APB2ENR::SYSCFGEN)
    }

    fn enable_syscfg_clock(&self) {
        self.registers.apb2enr.modify(APB2ENR::SYSCFGEN::SET)
    }

    fn disable_syscfg_clock(&self) {
        self.registers.apb2enr.modify(APB2ENR::SYSCFGEN::CLEAR)
    }

    // DMA1 clock

    fn is_enabled_dma1_clock(&self) -> bool {
        self.registers.ahbenr.is_set(AHBENR::DMA1EN)
    }

    fn enable_dma1_clock(&self) {
        self.registers.ahbenr.modify(AHBENR::DMA1EN::SET)
    }

    fn disable_dma1_clock(&self) {
        self.registers.ahbenr.modify(AHBENR::DMA1EN::CLEAR)
    }

    // GPIOF clock

    fn is_enabled_gpiof_clock(&self) -> bool {
        self.registers.ahbenr.is_set(AHBENR::IOPFEN)
    }

    fn enable_gpiof_clock(&self) {
        self.registers.ahbenr.modify(AHBENR::IOPFEN::SET)
    }

    fn disable_gpiof_clock(&self) {
        self.registers.ahbenr.modify(AHBENR::IOPFEN::CLEAR)
    }

    // GPIOE clock

    fn is_enabled_gpioe_clock(&self) -> bool {
        self.registers.ahbenr.is_set(AHBENR::IOPEEN)
    }

    fn enable_gpioe_clock(&self) {
        self.registers.ahbenr.modify(AHBENR::IOPEEN::SET)
    }

    fn disable_gpioe_clock(&self) {
        self.registers.ahbenr.modify(AHBENR::IOPEEN::CLEAR)
    }

    // GPIOD clock

    fn is_enabled_gpiod_clock(&self) -> bool {
        self.registers.ahbenr.is_set(AHBENR::IOPDEN)
    }

    fn enable_gpiod_clock(&self) {
        self.registers.ahbenr.modify(AHBENR::IOPDEN::SET)
    }

    fn disable_gpiod_clock(&self) {
        self.registers.ahbenr.modify(AHBENR::IOPDEN::CLEAR)
    }

    // GPIOC clock

    fn is_enabled_gpioc_clock(&self) -> bool {
        self.registers.ahbenr.is_set(AHBENR::IOPCEN)
    }

    fn enable_gpioc_clock(&self) {
        self.registers.ahbenr.modify(AHBENR::IOPCEN::SET)
    }

    fn disable_gpioc_clock(&self) {
        self.registers.ahbenr.modify(AHBENR::IOPCEN::CLEAR)
    }

    // GPIOB clock

    fn is_enabled_gpiob_clock(&self) -> bool {
        self.registers.ahbenr.is_set(AHBENR::IOPBEN)
    }

    fn enable_gpiob_clock(&self) {
        self.registers.ahbenr.modify(AHBENR::IOPBEN::SET)
    }

    fn disable_gpiob_clock(&self) {
        self.registers.ahbenr.modify(AHBENR::IOPBEN::CLEAR)
    }

    // GPIOA clock

    fn is_enabled_gpioa_clock(&self) -> bool {
        self.registers.ahbenr.is_set(AHBENR::IOPAEN)
    }

    fn enable_gpioa_clock(&self) {
        self.registers.ahbenr.modify(AHBENR::IOPAEN::SET)
    }

    fn disable_gpioa_clock(&self) {
        self.registers.ahbenr.modify(AHBENR::IOPAEN::CLEAR)
    }

    // USART1 clock

    fn is_enabled_usart1_clock(&self) -> bool {
        self.registers.apb2enr.is_set(APB2ENR::USART1EN)
    }

    fn enable_usart1_clock(&self) {
        self.registers.apb2enr.modify(APB2ENR::USART1EN::SET)
    }

    fn disable_usart1_clock(&self) {
        self.registers.apb2enr.modify(APB2ENR::USART1EN::CLEAR)
    }

    // USART2 clock

    fn is_enabled_usart2_clock(&self) -> bool {
        self.registers.apb1enr.is_set(APB1ENR::USART2EN)
    }

    fn enable_usart2_clock(&self) {
        self.registers.apb1enr.modify(APB1ENR::USART2EN::SET)
    }

    fn disable_usart2_clock(&self) {
        self.registers.apb1enr.modify(APB1ENR::USART2EN::CLEAR)
    }

    // USART3 clock

    fn is_enabled_usart3_clock(&self) -> bool {
        self.registers.apb1enr.is_set(APB1ENR::USART3EN)
    }

    fn enable_usart3_clock(&self) {
        self.registers.apb1enr.modify(APB1ENR::USART3EN::SET)
    }

    fn disable_usart3_clock(&self) {
        self.registers.apb1enr.modify(APB1ENR::USART3EN::CLEAR)
    }

    // ADC12 clock
    
    fn is_enabled_adc12_clock(&self) -> bool {
        self.registers.ahbenr.is_set(AHBENR::ADC12EN)
    }

    fn enable_adc12_clock(&self) {
        self.registers.ahbenr.modify(AHBENR::ADC12EN::SET)
    }

    fn disable_adc12_clock(&self) {
        self.registers.ahbenr.modify(AHBENR::ADC12EN::CLEAR)
    }

    // ADC34 clock

    // fn is_enabled_adc34_clock(&self) -> bool {
    //     self.registers.ahbenr.is_set(AHBENR::ADC34EN)
    // }

    // fn enable_adc34_clock(&self) {
    //     self.registers.ahbenr.modify(AHBENR::ADC34EN::SET)
    // }

    // fn disable_adc34_clock(&self) {
    //     self.registers.ahbenr.modify(AHBENR::ADC34EN::CLEAR)
    // }
}

/// Clock sources for CPU
pub enum CPUClock {
    HSE,
    HSI,
    PLLCLK,
}

/// Bus + Clock name for the peripherals
///
/// Not yet implemented clocks:
///
/// AHB2(HCLK2)
/// AHB3(HCLK3)
/// APB1(PCLK1),
/// APB2(PCLK2),
pub enum PeripheralClock {
    AHB(HCLK),
    APB2(PCLK2),
    APB1(PCLK1),
}

/// Peripherals clocked by HCLK1
pub enum HCLK {
    ADC12,
    // ADC34,
    GPIOF,
    GPIOE,
    GPIOD,
    GPIOC,
    GPIOB,
    GPIOA,
}

/// Peripherals clocked by PCLK1
pub enum PCLK1 {
    TIM2,
    USART2,
    USART3,
    // SPI3,
}

/// Peripherals clocked by PCLK2
pub enum PCLK2 {
    SYSCFG,
    USART1,
}

impl ClockInterface for PeripheralClock {
    fn is_enabled(&self) -> bool {
        match self {
            &PeripheralClock::AHB(ref v) => match v {
                HCLK::DMA1 => unsafe { RCC.is_enabled_dma1_clock() },
                HCLK3::ADC12 => unsafe { RCC.is_enabled_adc12_clock() },
                // HCLK::ADC34 => unsafe { RCC.is_enabled_adc34_clock() },
                HCLK::GPIOF => unsafe { RCC.is_enabled_gpiof_clock() },
                HCLK::GPIOE => unsafe { RCC.is_enabled_gpioe_clock() },
                HCLK::GPIOD => unsafe { RCC.is_enabled_gpiod_clock() },
                HCLK::GPIOC => unsafe { RCC.is_enabled_gpioc_clock() },
                HCLK::GPIOB => unsafe { RCC.is_enabled_gpiob_clock() },
                HCLK::GPIOA => unsafe { RCC.is_enabled_gpioa_clock() },
            },
            &PeripheralClock::APB1(ref v) => match v {
                PCLK1::TIM2 => unsafe { RCC.is_enabled_tim2_clock() },
                PCLK1::USART2 => unsafe { RCC.is_enabled_usart2_clock() },
                PCLK1::USART3 => unsafe { RCC.is_enabled_usart3_clock() },
                //     PCLK1::SPI3 => unsafe { RCC.is_enabled_spi3_clock() },
            },
            &PeripheralClock::APB2(ref v) => match v {
                PCLK2::SYSCFG => unsafe { RCC.is_enabled_syscfg_clock() },
                PCLK2::USART1 => unsafe { RCC.is_enabled_usart1_clock() },
            },
        }
    }

    fn enable(&self) {
        match self {
            &PeripheralClock::AHB(ref v) => match v {
                HCLK::DMA1 => unsafe {
                    RCC.enable_dma1_clock();
                },
                HCLK3::ADC12 => unsafe {
                    RCC.enable_adc12_clock();
                },
                // HCLK3::ADC34 => unsafe {
                //     RCC.enable_adc34_clock();
                // },
                HCLK::GPIOF => unsafe {
                    RCC.enable_gpiof_clock();
                },
                HCLK::GPIOE => unsafe {
                    RCC.enable_gpioe_clock();
                },
                HCLK::GPIOD => unsafe {
                    RCC.enable_gpiod_clock();
                },
                HCLK::GPIOC => unsafe {
                    RCC.enable_gpioc_clock();
                },
                HCLK::GPIOB => unsafe {
                    RCC.enable_gpiob_clock();
                },
                HCLK::GPIOA => unsafe {
                    RCC.enable_gpioa_clock();
                },
            },
            &PeripheralClock::APB1(ref v) => match v {
                PCLK1::TIM2 => unsafe {
                    RCC.enable_tim2_clock();
                },
                PCLK1::USART2 => unsafe {
                    RCC.enable_usart2_clock();
                },
                PCLK1::USART3 => unsafe {
                    RCC.enable_usart3_clock();
                },
                //     PCLK1::SPI3 => unsafe {
                //         RCC.enable_spi3_clock();
                //     },
            },
            &PeripheralClock::APB2(ref v) => match v {
                PCLK2::SYSCFG => unsafe {
                    RCC.enable_syscfg_clock();
                },
                PCLK2::USART1 => unsafe {
                    RCC.enable_usart1_clock();
                },
            },
        }
    }

    fn disable(&self) {
        match self {
            &PeripheralClock::AHB(ref v) => match v {
                HCLK::DMA1 => unsafe {
                    RCC.disable_dma1_clock();
                },
                HCLK3::ADC12 => unsafe {
                    RCC.enable_adc12_clock();
                },
                // HCLK3::ADC34 => unsafe {
                //     RCC.enable_adc34_clock();
                // },
                HCLK::GPIOF => unsafe {
                    RCC.disable_gpiof_clock();
                },
                HCLK::GPIOE => unsafe {
                    RCC.disable_gpioe_clock();
                },
                HCLK::GPIOD => unsafe {
                    RCC.disable_gpiod_clock();
                },
                HCLK::GPIOC => unsafe {
                    RCC.disable_gpioc_clock();
                },
                HCLK::GPIOB => unsafe {
                    RCC.disable_gpiob_clock();
                },
                HCLK::GPIOA => unsafe {
                    RCC.disable_gpioa_clock();
                },
            },
            &PeripheralClock::APB1(ref v) => match v {
                PCLK1::TIM2 => unsafe {
                    RCC.disable_tim2_clock();
                },
                PCLK1::USART2 => unsafe {
                    RCC.disable_usart2_clock();
                },
                PCLK1::USART3 => unsafe {
                    RCC.disable_usart3_clock();
                },
                //     PCLK1::SPI3 => unsafe {
                //         RCC.disable_spi3_clock();
                //     },
            },
            &PeripheralClock::APB2(ref v) => match v {
                PCLK2::SYSCFG => unsafe {
                    RCC.disable_syscfg_clock();
                },
                PCLK2::USART1 => unsafe {
                    RCC.disable_usart1_clock();
                },
            },
        }
    }
}

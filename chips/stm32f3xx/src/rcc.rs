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

    // /// AHB2 peripheral reset register
    // ahb2rstr: ReadWrite<u32, AHB2RSTR::Register>,
    // /// AHB3 peripheral reset register
    // ahb3rstr: ReadWrite<u32, AHB3RSTR::Register>,
    // _reserved0: [u8; 4],
    // /// APB1 peripheral reset register
    // apb1rstr: ReadWrite<u32, APB1RSTR::Register>,
    // _reserved1: [u8; 8],
    // /// AHB2 peripheral clock enable register
    // ahb2enr: ReadWrite<u32, AHB2ENR::Register>,
    // /// AHB3 peripheral clock enable register
    // ahb3enr: ReadWrite<u32, AHB3ENR::Register>,
    // _reserved2: [u8; 4],
    // /// APB1 peripheral clock enable register
    // apb1enr: ReadWrite<u32, APB1ENR::Register>,
    // /// APB2 peripheral clock enable register
    // apb2enr: ReadWrite<u32, APB2ENR::Register>,
    // _reserved3: [u8; 8],
    // /// AHB1 peripheral clock enable in low power mode register
    // ahb1lpenr: ReadWrite<u32, AHB1LPENR::Register>,
    // /// AHB2 peripheral clock enable in low power mode register
    // ahb2lpenr: ReadWrite<u32, AHB2LPENR::Register>,
    // /// AHB3 peripheral clock enable in low power mode register
    // ahb3lpenr: ReadWrite<u32, AHB3LPENR::Register>,
    // _reserved4: [u8; 4],
    // /// APB1 peripheral clock enable in low power mode register
    // apb1lpenr: ReadWrite<u32, APB1LPENR::Register>,
    // /// APB2 peripheral clock enabled in low power mode register
    // apb2lpenr: ReadWrite<u32, APB2LPENR::Register>,
    // _reserved5: [u8; 8],
    // /// Backup domain control register
    // bdcr: ReadWrite<u32, BDCR::Register>,
    // /// clock control & status register
    // csr: ReadWrite<u32, CSR::Register>,
    // _reserved6: [u8; 8],
    // /// spread spectrum clock generation register
    // sscgr: ReadWrite<u32, SSCGR::Register>,
    // /// PLLI2S configuration register
    // plli2scfgr: ReadWrite<u32, PLLI2SCFGR::Register>,
    // /// PLL configuration register
    // pllsaicfgr: ReadWrite<u32, PLLSAICFGR::Register>,
    // /// Dedicated Clock Configuration Register
    // dckcfgr: ReadWrite<u32, DCKCFGR::Register>,
    // /// clocks gated enable register
    // ckgatenr: ReadWrite<u32, CKGATENR::Register>,
    // /// dedicated clocks configuration register 2
    // dckcfgr2: ReadWrite<u32, DCKCFGR2::Register>,
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
        #[cfg(feature = "stm32f303vct6")]
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
    ]


    // AHB1RSTR [
    //     /// USB OTG HS module reset
    //     OTGHSRST OFFSET(29) NUMBITS(1) [],
    //     /// DMA2 reset
    //     DMA2RST OFFSET(22) NUMBITS(1) [],
    //     /// DMA2 reset
    //     DMA1RST OFFSET(21) NUMBITS(1) [],
    //     /// CRC reset
    //     CRCRST OFFSET(12) NUMBITS(1) [],
    //     /// IO port H reset
    //     GPIOHRST OFFSET(7) NUMBITS(1) [],
    //     /// IO port G reset
    //     GPIOGRST OFFSET(6) NUMBITS(1) [],
    //     /// IO port F reset
    //     GPIOFRST OFFSET(5) NUMBITS(1) [],
    //     /// IO port E reset
    //     GPIOERST OFFSET(4) NUMBITS(1) [],
    //     /// IO port D reset
    //     GPIODRST OFFSET(3) NUMBITS(1) [],
    //     /// IO port C reset
    //     GPIOCRST OFFSET(2) NUMBITS(1) [],
    //     /// IO port B reset
    //     GPIOBRST OFFSET(1) NUMBITS(1) [],
    //     /// IO port A reset
    //     GPIOARST OFFSET(0) NUMBITS(1) []
    // ],
    // AHB2RSTR [
    //     /// USB OTG FS module reset
    //     OTGFSRST OFFSET(7) NUMBITS(1) [],
    //     /// Camera interface reset
    //     DCMIRST OFFSET(0) NUMBITS(1) []
    // ],
    // AHB3RSTR [
    //     /// Flexible memory controller module reset
    //     FMCRST OFFSET(0) NUMBITS(1) [],
    //     /// QUADSPI module reset
    //     QSPIRST OFFSET(1) NUMBITS(1) []
    // ],
    
    
    
    // AHB2ENR [
    //     /// USB OTG FS clock enable
    //     OTGFSEN OFFSET(7) NUMBITS(1) [],
    //     /// Camera interface enable
    //     DCMIEN OFFSET(0) NUMBITS(1) []
    // ],
    // AHB3ENR [
    //     /// Flexible memory controller module clock enable
    //     FMCEN OFFSET(0) NUMBITS(1) [],
    //     /// QUADSPI memory controller module clock enable
    //     QSPIEN OFFSET(1) NUMBITS(1) []
    // ],
    // APB1ENR [
    //     /// TIM2 clock enable
    //     TIM2EN OFFSET(0) NUMBITS(1) [],
    //     /// TIM3 clock enable
    //     TIM3EN OFFSET(1) NUMBITS(1) [],
    //     /// TIM4 clock enable
    //     TIM4EN OFFSET(2) NUMBITS(1) [],
    //     /// TIM5 clock enable
    //     TIM5EN OFFSET(3) NUMBITS(1) [],
    //     /// TIM6 clock enable
    //     TIM6EN OFFSET(4) NUMBITS(1) [],
    //     /// TIM7 clock enable
    //     TIM7EN OFFSET(5) NUMBITS(1) [],
    //     /// TIM12 clock enable
    //     TIM12EN OFFSET(6) NUMBITS(1) [],
    //     /// TIM13 clock enable
    //     TIM13EN OFFSET(7) NUMBITS(1) [],
    //     /// TIM14 clock enable
    //     TIM14EN OFFSET(8) NUMBITS(1) [],
    //     /// Window watchdog clock enable
    //     WWDGEN OFFSET(11) NUMBITS(1) [],
    //     /// SPI2 clock enable
    //     SPI2EN OFFSET(14) NUMBITS(1) [],
    //     /// SPI3 clock enable
    //     SPI3EN OFFSET(15) NUMBITS(1) [],
    //     /// SPDIF-IN clock enable
    //     SPDIFEN OFFSET(16) NUMBITS(1) [],
    //     /// USART 2 clock enable
    //     USART2EN OFFSET(17) NUMBITS(1) [],
    //     /// USART3 clock enable
    //     USART3EN OFFSET(18) NUMBITS(1) [],
    //     /// UART4 clock enable
    //     UART4EN OFFSET(19) NUMBITS(1) [],
    //     /// UART5 clock enable
    //     UART5EN OFFSET(20) NUMBITS(1) [],
    //     /// I2C1 clock enable
    //     I2C1EN OFFSET(21) NUMBITS(1) [],
    //     /// I2C2 clock enable
    //     I2C2EN OFFSET(22) NUMBITS(1) [],
    //     /// I2C3 clock enable
    //     I2C3EN OFFSET(23) NUMBITS(1) [],
    //     /// I2CFMP1 clock enable
    //     I2CFMP1EN OFFSET(24) NUMBITS(1) [],
    //     /// CAN 1 clock enable
    //     CAN1EN OFFSET(25) NUMBITS(1) [],
    //     /// CAN 2 clock enable
    //     CAN2EN OFFSET(26) NUMBITS(1) [],
    //     /// CEC interface clock enable
    //     CEC OFFSET(27) NUMBITS(1) [],
    //     /// Power interface clock enable
    //     PWREN OFFSET(28) NUMBITS(1) [],
    //     /// DAC interface clock enable
    //     DACEN OFFSET(29) NUMBITS(1) []
    // ],
    // APB2ENR [
    //     /// TIM1 clock enable
    //     TIM1EN OFFSET(0) NUMBITS(1) [],
    //     /// TIM8 clock enable
    //     TIM8EN OFFSET(1) NUMBITS(1) [],
    //     /// USART1 clock enable
    //     USART1EN OFFSET(4) NUMBITS(1) [],
    //     /// USART6 clock enable
    //     USART6EN OFFSET(5) NUMBITS(1) [],
    //     /// ADC1 clock enable
    //     ADC1EN OFFSET(8) NUMBITS(1) [],
    //     /// ADC2 clock enable
    //     ADC2EN OFFSET(9) NUMBITS(1) [],
    //     /// ADC3 clock enable
    //     ADC3EN OFFSET(10) NUMBITS(1) [],
    //     /// SDIO clock enable
    //     SDIOEN OFFSET(11) NUMBITS(1) [],
    //     /// SPI1 clock enable
    //     SPI1EN OFFSET(12) NUMBITS(1) [],
    //     /// SPI4 clock enable
    //     SPI4ENR OFFSET(13) NUMBITS(1) [],
    //     /// System configuration controller clock enable
    //     SYSCFGEN OFFSET(14) NUMBITS(1) [],
    //     /// TIM9 clock enable
    //     TIM9EN OFFSET(16) NUMBITS(1) [],
    //     /// TIM10 clock enable
    //     TIM10EN OFFSET(17) NUMBITS(1) [],
    //     /// TIM11 clock enable
    //     TIM11EN OFFSET(18) NUMBITS(1) [],
    //     /// SAI1 clock enable
    //     SAI1EN OFFSET(22) NUMBITS(1) [],
    //     /// SAI2 clock enable
    //     SAI2EN OFFSET(23) NUMBITS(1) []
    // ],
    // AHB1LPENR [
    //     /// IO port A clock enable during sleep mode
    //     GPIOALPEN OFFSET(0) NUMBITS(1) [],
    //     /// IO port B clock enable during Sleep mode
    //     GPIOBLPEN OFFSET(1) NUMBITS(1) [],
    //     /// IO port C clock enable during Sleep mode
    //     GPIOCLPEN OFFSET(2) NUMBITS(1) [],
    //     /// IO port D clock enable during Sleep mode
    //     GPIODLPEN OFFSET(3) NUMBITS(1) [],
    //     /// IO port E clock enable during Sleep mode
    //     GPIOELPEN OFFSET(4) NUMBITS(1) [],
    //     /// IO port F clock enable during Sleep mode
    //     GPIOFLPEN OFFSET(5) NUMBITS(1) [],
    //     /// IO port G clock enable during Sleep mode
    //     GPIOGLPEN OFFSET(6) NUMBITS(1) [],
    //     /// IO port H clock enable during Sleep mode
    //     GPIOHLPEN OFFSET(7) NUMBITS(1) [],
    //     /// CRC clock enable during Sleep mode
    //     CRCLPEN OFFSET(12) NUMBITS(1) [],
    //     /// Flash interface clock enable during Sleep mode
    //     FLITFLPEN OFFSET(15) NUMBITS(1) [],
    //     /// SRAM 1interface clock enable during Sleep mode
    //     SRAM1LPEN OFFSET(16) NUMBITS(1) [],
    //     /// SRAM 2 interface clock enable during Sleep mode
    //     SRAM2LPEN OFFSET(17) NUMBITS(1) [],
    //     /// Backup SRAM interface clock enable during Sleep mode
    //     BKPSRAMLPEN OFFSET(18) NUMBITS(1) [],
    //     /// DMA1 clock enable during Sleep mode
    //     DMA1LPEN OFFSET(21) NUMBITS(1) [],
    //     /// DMA2 clock enable during Sleep mode
    //     DMA2LPEN OFFSET(22) NUMBITS(1) [],
    //     /// USB OTG HS clock enable during Sleep mode
    //     OTGHSLPEN OFFSET(29) NUMBITS(1) [],
    //     /// USB OTG HS ULPI clock enable during Sleep mode
    //     OTGHSULPILPEN OFFSET(30) NUMBITS(1) []
    // ],
    // AHB2LPENR [
    //     /// USB OTG FS clock enable during Sleep mode
    //     OTGFSLPEN OFFSET(7) NUMBITS(1) [],
    //     /// Camera interface enable during Sleep mode
    //     DCMILPEN OFFSET(0) NUMBITS(1) []
    // ],
    // AHB3LPENR [
    //     /// Flexible memory controller module clock enable during Sleep mode
    //     FMCLPEN OFFSET(0) NUMBITS(1) [],
    //     /// QUADSPI memory controller module clock enable during Sleep mode
    //     QSPILPEN OFFSET(1) NUMBITS(1) []
    // ],
    // APB1LPENR [
    //     /// TIM2 clock enable during Sleep mode
    //     TIM2LPEN OFFSET(0) NUMBITS(1) [],
    //     /// TIM3 clock enable during Sleep mode
    //     TIM3LPEN OFFSET(1) NUMBITS(1) [],
    //     /// TIM4 clock enable during Sleep mode
    //     TIM4LPEN OFFSET(2) NUMBITS(1) [],
    //     /// TIM5 clock enable during Sleep mode
    //     TIM5LPEN OFFSET(3) NUMBITS(1) [],
    //     /// TIM6 clock enable during Sleep mode
    //     TIM6LPEN OFFSET(4) NUMBITS(1) [],
    //     /// TIM7 clock enable during Sleep mode
    //     TIM7LPEN OFFSET(5) NUMBITS(1) [],
    //     /// TIM12 clock enable during Sleep mode
    //     TIM12LPEN OFFSET(6) NUMBITS(1) [],
    //     /// TIM13 clock enable during Sleep mode
    //     TIM13LPEN OFFSET(7) NUMBITS(1) [],
    //     /// TIM14 clock enable during Sleep mode
    //     TIM14LPEN OFFSET(8) NUMBITS(1) [],
    //     /// Window watchdog clock enable during Sleep mode
    //     WWDGLPEN OFFSET(11) NUMBITS(1) [],
    //     /// SPI2 clock enable during Sleep mode
    //     SPI2LPEN OFFSET(14) NUMBITS(1) [],
    //     /// SPI3 clock enable during Sleep mode
    //     SPI3LPEN OFFSET(15) NUMBITS(1) [],
    //     /// SPDIF clock enable during Sleep mode
    //     SPDIFLPEN OFFSET(16) NUMBITS(1) [],
    //     /// USART2 clock enable during Sleep mode
    //     USART2LPEN OFFSET(17) NUMBITS(1) [],
    //     /// USART3 clock enable during Sleep mode
    //     USART3LPEN OFFSET(18) NUMBITS(1) [],
    //     /// UART4 clock enable during Sleep mode
    //     UART4LPEN OFFSET(19) NUMBITS(1) [],
    //     /// UART5 clock enable during Sleep mode
    //     UART5LPEN OFFSET(20) NUMBITS(1) [],
    //     /// I2C1 clock enable during Sleep mode
    //     I2C1LPEN OFFSET(21) NUMBITS(1) [],
    //     /// I2C2 clock enable during Sleep mode
    //     I2C2LPEN OFFSET(22) NUMBITS(1) [],
    //     /// I2C3 clock enable during Sleep mode
    //     I2C3LPEN OFFSET(23) NUMBITS(1) [],
    //     /// I2CFMP1 clock enable during Sleep mode
    //     I2CFMP1LPEN OFFSET(24) NUMBITS(1) [],
    //     /// CAN 1 clock enable during Sleep mode
    //     CAN1LPEN OFFSET(25) NUMBITS(1) [],
    //     /// CAN 2 clock enable during Sleep mode
    //     CAN2LPEN OFFSET(26) NUMBITS(1) [],
    //     /// CEC clock enable during Sleep mode
    //     CECLPEN OFFSET(27) NUMBITS(1) [],
    //     /// Power interface clock enable during Sleep mode
    //     PWRLPEN OFFSET(28) NUMBITS(1) [],
    //     /// DAC interface clock enable during Sleep mode
    //     DACLPEN OFFSET(29) NUMBITS(1) []
    // ],
    // APB2LPENR [
    //     /// TIM1 clock enable during Sleep mode
    //     TIM1LPEN OFFSET(0) NUMBITS(1) [],
    //     /// TIM8 clock enable during Sleep mode
    //     TIM8LPEN OFFSET(1) NUMBITS(1) [],
    //     /// USART1 clock enable during Sleep mode
    //     USART1LPEN OFFSET(4) NUMBITS(1) [],
    //     /// USART6 clock enable during Sleep mode
    //     USART6LPEN OFFSET(5) NUMBITS(1) [],
    //     /// ADC1 clock enable during Sleep mode
    //     ADC1LPEN OFFSET(8) NUMBITS(1) [],
    //     /// ADC2 clock enable during Sleep mode
    //     ADC2LPEN OFFSET(9) NUMBITS(1) [],
    //     /// ADC 3 clock enable during Sleep mode
    //     ADC3LPEN OFFSET(10) NUMBITS(1) [],
    //     /// SDIO clock enable during Sleep mode
    //     SDIOLPEN OFFSET(11) NUMBITS(1) [],
    //     /// SPI 1 clock enable during Sleep mode
    //     SPI1LPEN OFFSET(12) NUMBITS(1) [],
    //     /// SPI 4 clock enable during Sleep mode
    //     SPI4LPEN OFFSET(13) NUMBITS(1) [],
    //     /// System configuration controller clock enable during Sleep mode
    //     SYSCFGLPEN OFFSET(14) NUMBITS(1) [],
    //     /// TIM9 clock enable during sleep mode
    //     TIM9LPEN OFFSET(16) NUMBITS(1) [],
    //     /// TIM10 clock enable during Sleep mode
    //     TIM10LPEN OFFSET(17) NUMBITS(1) [],
    //     /// TIM11 clock enable during Sleep mode
    //     TIM11LPEN OFFSET(18) NUMBITS(1) [],
    //     /// SAI1 clock enable
    //     SAI1LPEN OFFSET(22) NUMBITS(1) [],
    //     /// SAI2 clock enable
    //     SAI2LPEN OFFSET(23) NUMBITS(1) []
    // ],
    // BDCR [
    //     /// Backup domain software reset
    //     BDRST OFFSET(16) NUMBITS(1) [],
    //     /// RTC clock enable
    //     RTCEN OFFSET(15) NUMBITS(1) [],
    //     /// RTC clock source selection
    //     RTCSEL OFFSET(8) NUMBITS(2) [],
    //     /// External low-speed oscillator mode
    //     LSEMOD OFFSET(3) NUMBITS(1) [],
    //     /// External low-speed oscillator bypass
    //     LSEBYP OFFSET(2) NUMBITS(1) [],
    //     /// External low-speed oscillator ready
    //     LSERDY OFFSET(1) NUMBITS(1) [],
    //     /// External low-speed oscillator enable
    //     LSEON OFFSET(0) NUMBITS(1) []
    // ],
    // CSR [
    //     /// Low-power reset flag
    //     LPWRRSTF OFFSET(31) NUMBITS(1) [],
    //     /// Window watchdog reset flag
    //     WWDGRSTF OFFSET(30) NUMBITS(1) [],
    //     /// Independent watchdog reset flag
    //     WDGRSTF OFFSET(29) NUMBITS(1) [],
    //     /// Software reset flag
    //     SFTRSTF OFFSET(28) NUMBITS(1) [],
    //     /// POR/PDR reset flag
    //     PORRSTF OFFSET(27) NUMBITS(1) [],
    //     /// PIN reset flag
    //     PADRSTF OFFSET(26) NUMBITS(1) [],
    //     /// BOR reset flag
    //     BORRSTF OFFSET(25) NUMBITS(1) [],
    //     /// Remove reset flag
    //     RMVF OFFSET(24) NUMBITS(1) [],
    //     /// Internal low-speed oscillator ready
    //     LSIRDY OFFSET(1) NUMBITS(1) [],
    //     /// Internal low-speed oscillator enable
    //     LSION OFFSET(0) NUMBITS(1) []
    // ],
    // SSCGR [
    //     /// Spread spectrum modulation enable
    //     SSCGEN OFFSET(31) NUMBITS(1) [],
    //     /// Spread Select
    //     SPREADSEL OFFSET(30) NUMBITS(1) [],
    //     /// Incrementation step
    //     INCSTEP OFFSET(13) NUMBITS(15) [],
    //     /// Modulation period
    //     MODPER OFFSET(0) NUMBITS(13) []
    // ],
    // PLLI2SCFGR [
    //     /// Division factor for audio PLL (PLLI2S) input clock
    //     PLLI2SM OFFSET(0) NUMBITS(6) [],
    //     /// PLLI2S multiplication factor for VCO
    //     PLLI2SN OFFSET(6) NUMBITS(9) [],
    //     /// PLLI2S division factor for SPDIF-IN clock
    //     PLLI2SP OFFSET(16) NUMBITS(2) [],
    //     /// PLLI2S division factor for SAI1 clock
    //     PLLI2SQ OFFSET(24) NUMBITS(4) [],
    //     /// PLLI2S division factor for I2S clocks
    //     PLLI2SR OFFSET(28) NUMBITS(3) []
    // ],
    // PLLSAICFGR [
    //     /// Division factor for audio PLLSAI input clock
    //     PLLSAIM OFFSET(0) NUMBITS(6) [],
    //     /// PLLSAI division factor for VCO
    //     PLLSAIN OFFSET(6) NUMBITS(9) [],
    //     /// PLLSAI division factor for 48 MHz clock
    //     PLLSAIP OFFSET(16) NUMBITS(2) [],
    //     /// PLLSAI division factor for SAIs clock
    //     PLLSAIQ OFFSET(24) NUMBITS(4) []
    // ],
    // DCKCFGR [
    //     /// PLLI2S division factor for SAIs clock
    //     PLLI2SDIVQ OFFSET(0) NUMBITS(5) [],
    //     /// PLLSAI division factor for SAIs clock
    //     PLLSAIDIVQ OFFSET(8) NUMBITS(5) [],
    //     /// SAI1 clock source selection
    //     SAI1SRC OFFSET(20) NUMBITS(2) [],
    //     /// SAI2 clock source selection
    //     SAI2SRC OFFSET(22) NUMBITS(2) [],
    //     /// Timers clocks prescalers selection
    //     TIMPRE OFFSET(24) NUMBITS(1) [],
    //     /// I2S APB1 clock source selection
    //     I2S1SRC OFFSET(25) NUMBITS(2) [],
    //     /// I2S APB2 clock source selection
    //     I2S2SRC OFFSET(27) NUMBITS(2) []
    // ],
    // CKGATENR [
    //     /// AHB to APB1 Bridge clock enable
    //     AHB2APB1_CKEN OFFSET(0) NUMBITS(1) [],
    //     /// AHB to APB2 Bridge clock enable
    //     AHB2APB2_CKEN OFFSET(1) NUMBITS(1) [],
    //     /// Cortex M4 ETM clock enable
    //     CM4DBG_CKEN OFFSET(2) NUMBITS(1) [],
    //     /// Spare clock enable
    //     SPARE_CKEN OFFSET(3) NUMBITS(1) [],
    //     /// SRQAM controller clock enable
    //     SRAM_CKEN OFFSET(4) NUMBITS(1) [],
    //     /// Flash Interface clock enable
    //     FLITF_CKEN OFFSET(5) NUMBITS(1) [],
    //     /// RCC clock enable
    //     RCC_CKEN OFFSET(6) NUMBITS(1) []
    // ],
    // DCKCFGR2 [
    //     /// I2C4 kernel clock source selection
    //     FMPI2C1SEL OFFSET(22) NUMBITS(2) [],
    //     /// HDMI CEC clock source selection
    //     CECSEL OFFSET(26) NUMBITS(1) [],
    //     /// SDIO/USBFS/HS clock selection
    //     CK48MSEL OFFSET(27) NUMBITS(1) [],
    //     /// SDIO clock selection
    //     SDIOSEL OFFSET(28) NUMBITS(1) [],
    //     /// SPDIF clock selection
    //     SPDIFSEL OFFSET(29) NUMBITS(1) []
    // ]
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

    // SPI3 clock

    // fn is_enabled_spi3_clock(&self) -> bool {
    //     self.registers.apb1enr.is_set(APB1ENR::SPI3EN)
    // }

    // fn enable_spi3_clock(&self) {
    //     self.registers.apb1enr.modify(APB1ENR::SPI3EN::SET)
    // }

    // fn disable_spi3_clock(&self) {
    //     self.registers.apb1enr.modify(APB1ENR::SPI3EN::CLEAR)
    // }

    // TIM2 clock

    // fn is_enabled_tim2_clock(&self) -> bool {
    //     self.registers.apb1enr.is_set(APB1ENR::TIM2EN)
    // }

    // fn enable_tim2_clock(&self) {
    //     self.registers.apb1enr.modify(APB1ENR::TIM2EN::SET)
    // }

    // fn disable_tim2_clock(&self) {
    //     self.registers.apb1enr.modify(APB1ENR::TIM2EN::CLEAR)
    // }

    // SYSCFG clock

    // fn is_enabled_syscfg_clock(&self) -> bool {
    //     self.registers.apb2enr.is_set(APB2ENR::SYSCFGEN)
    // }

    // fn enable_syscfg_clock(&self) {
    //     self.registers.apb2enr.modify(APB2ENR::SYSCFGEN::SET)
    // }

    // fn disable_syscfg_clock(&self) {
    //     self.registers.apb2enr.modify(APB2ENR::SYSCFGEN::CLEAR)
    // }

    // DMA1 clock

    // fn is_enabled_dma1_clock(&self) -> bool {
    //     self.registers.ahb1enr.is_set(AHB1ENR::DMA1EN)
    // }

    // fn enable_dma1_clock(&self) {
    //     self.registers.ahb1enr.modify(AHB1ENR::DMA1EN::SET)
    // }

    // fn disable_dma1_clock(&self) {
    //     self.registers.ahb1enr.modify(AHB1ENR::DMA1EN::CLEAR)
    // }

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

    // USART2 clock

    // fn is_enabled_usart2_clock(&self) -> bool {
    //     self.registers.apb1enr.is_set(APB1ENR::USART2EN)
    // }

    // fn enable_usart2_clock(&self) {
    //     self.registers.apb1enr.modify(APB1ENR::USART2EN::SET)
    // }

    // fn disable_usart2_clock(&self) {
    //     self.registers.apb1enr.modify(APB1ENR::USART2EN::CLEAR)
    // }

    // USART3 clock

    // fn is_enabled_usart3_clock(&self) -> bool {
    //     self.registers.apb1enr.is_set(APB1ENR::USART3EN)
    // }

    // fn enable_usart3_clock(&self) {
    //     self.registers.apb1enr.modify(APB1ENR::USART3EN::SET)
    // }

    // fn disable_usart3_clock(&self) {
    //     self.registers.apb1enr.modify(APB1ENR::USART3EN::CLEAR)
    // }
}

/// Clock sources for CPU
pub enum CPUClock {
    HSE,
    HSI,
    LSE,
    PLLCLK
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
}

/// Peripherals clocked by HCLK1
pub enum HCLK {
    GPIOF,
    GPIOE,
    GPIOD,
    GPIOC,
    GPIOB,
    GPIOA,
}

/// Peripherals clocked by PCLK1
// pub enum PCLK1 {
//     TIM2,
//     USART2,
//     USART3,
//     SPI3,
// }

/// Peripherals clocked by PCLK2
// pub enum PCLK2 {
//     SYSCFG,
// }

impl ClockInterface for PeripheralClock {
    fn is_enabled(&self) -> bool {
        match self {
            &PeripheralClock::AHB(ref v) => match v {
                HCLK::GPIOF => unsafe { RCC.is_enabled_gpiof_clock() },
                HCLK::GPIOE => unsafe { RCC.is_enabled_gpioe_clock() },
                HCLK::GPIOD => unsafe { RCC.is_enabled_gpiod_clock() },
                HCLK::GPIOC => unsafe { RCC.is_enabled_gpioc_clock() },
                HCLK::GPIOB => unsafe { RCC.is_enabled_gpiob_clock() },
                HCLK::GPIOA => unsafe { RCC.is_enabled_gpioa_clock() },
            },
            // &PeripheralClock::APB1(ref v) => match v {
            //     PCLK1::TIM2 => unsafe { RCC.is_enabled_tim2_clock() },
            //     PCLK1::USART2 => unsafe { RCC.is_enabled_usart2_clock() },
            //     PCLK1::USART3 => unsafe { RCC.is_enabled_usart3_clock() },
            //     PCLK1::SPI3 => unsafe { RCC.is_enabled_spi3_clock() },
            // },
            // &PeripheralClock::APB2(ref v) => match v {
            //     PCLK2::SYSCFG => unsafe { RCC.is_enabled_syscfg_clock() },
            // },
        }
    }

    fn enable(&self) {
        match self {
            &PeripheralClock::AHB(ref v) => match v {
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
            // &PeripheralClock::APB1(ref v) => match v {
            //     PCLK1::TIM2 => unsafe {
            //         RCC.enable_tim2_clock();
            //     },
            //     PCLK1::USART2 => unsafe {
            //         RCC.enable_usart2_clock();
            //     },
            //     PCLK1::USART3 => unsafe {
            //         RCC.enable_usart3_clock();
            //     },
            //     PCLK1::SPI3 => unsafe {
            //         RCC.enable_spi3_clock();
            //     },
            // },
            // &PeripheralClock::APB2(ref v) => match v {
            //     PCLK2::SYSCFG => unsafe {
            //         RCC.enable_syscfg_clock();
            //     },
            // },
        }
    }

    fn disable(&self) {
        match self {
            &PeripheralClock::AHB(ref v) => match v {
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
        }
    }
}

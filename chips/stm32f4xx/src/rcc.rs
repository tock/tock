use kernel::platform::chip::ClockInterface;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable};
use kernel::utilities::registers::{register_bitfields, ReadWrite};
use kernel::utilities::StaticRef;

/// Reset and clock control
#[repr(C)]
struct RccRegisters {
    /// clock control register
    cr: ReadWrite<u32, CR::Register>,
    /// PLL configuration register
    pllcfgr: ReadWrite<u32, PLLCFGR::Register>,
    /// clock configuration register
    cfgr: ReadWrite<u32, CFGR::Register>,
    /// clock interrupt register
    cir: ReadWrite<u32, CIR::Register>,
    /// AHB1 peripheral reset register
    ahb1rstr: ReadWrite<u32, AHB1RSTR::Register>,
    /// AHB2 peripheral reset register
    ahb2rstr: ReadWrite<u32, AHB2RSTR::Register>,
    /// AHB3 peripheral reset register
    ahb3rstr: ReadWrite<u32, AHB3RSTR::Register>,
    _reserved0: [u8; 4],
    /// APB1 peripheral reset register
    apb1rstr: ReadWrite<u32, APB1RSTR::Register>,
    /// APB2 peripheral reset register
    apb2rstr: ReadWrite<u32, APB2RSTR::Register>,
    _reserved1: [u8; 8],
    /// AHB1 peripheral clock register
    ahb1enr: ReadWrite<u32, AHB1ENR::Register>,
    /// AHB2 peripheral clock enable register
    ahb2enr: ReadWrite<u32, AHB2ENR::Register>,
    /// AHB3 peripheral clock enable register
    ahb3enr: ReadWrite<u32, AHB3ENR::Register>,
    _reserved2: [u8; 4],
    /// APB1 peripheral clock enable register
    apb1enr: ReadWrite<u32, APB1ENR::Register>,
    /// APB2 peripheral clock enable register
    apb2enr: ReadWrite<u32, APB2ENR::Register>,
    _reserved3: [u8; 8],
    /// AHB1 peripheral clock enable in low power mode register
    ahb1lpenr: ReadWrite<u32, AHB1LPENR::Register>,
    /// AHB2 peripheral clock enable in low power mode register
    ahb2lpenr: ReadWrite<u32, AHB2LPENR::Register>,
    /// AHB3 peripheral clock enable in low power mode register
    ahb3lpenr: ReadWrite<u32, AHB3LPENR::Register>,
    _reserved4: [u8; 4],
    /// APB1 peripheral clock enable in low power mode register
    apb1lpenr: ReadWrite<u32, APB1LPENR::Register>,
    /// APB2 peripheral clock enabled in low power mode register
    apb2lpenr: ReadWrite<u32, APB2LPENR::Register>,
    _reserved5: [u8; 8],
    /// Backup domain control register
    bdcr: ReadWrite<u32, BDCR::Register>,
    /// clock control & status register
    csr: ReadWrite<u32, CSR::Register>,
    _reserved6: [u8; 8],
    /// spread spectrum clock generation register
    sscgr: ReadWrite<u32, SSCGR::Register>,
    /// PLLI2S configuration register
    plli2scfgr: ReadWrite<u32, PLLI2SCFGR::Register>,
    /// PLL configuration register
    pllsaicfgr: ReadWrite<u32, PLLSAICFGR::Register>,
    /// Dedicated Clock Configuration Register
    dckcfgr: ReadWrite<u32, DCKCFGR::Register>,
    /// clocks gated enable register
    ckgatenr: ReadWrite<u32, CKGATENR::Register>,
    /// dedicated clocks configuration register 2
    dckcfgr2: ReadWrite<u32, DCKCFGR2::Register>,
}

register_bitfields![u32,
    CR [
        /// PLLI2S clock ready flag
        PLLI2SRDY OFFSET(27) NUMBITS(1) [],
        /// PLLI2S enable
        PLLI2SON OFFSET(26) NUMBITS(1) [],
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
    PLLCFGR [
        /// Main PLL (PLL) division factor for USB OTG FS, SDIO and random num
        PLLQ OFFSET(24) NUMBITS(4) [],
        /// Main PLL(PLL) and audio PLL (PLLI2S) entry clock source
        PLLSRC OFFSET(22) NUMBITS(1) [
            HSI = 0,
            HSE = 1,
        ],
        /// Main PLL (PLL) division factor for main system clock
        PLLP OFFSET(16) NUMBITS(2) [
            DivideBy2 = 0b00,
            DivideBy4 = 0b01,
            DivideBy6 = 0b10,
            DivideBy8 = 0b11,
        ],
        /// Main PLL (PLL) multiplication factor for VCO
        PLLN OFFSET(6) NUMBITS(9) [],
        /// Division factor for the main PLL (PLL) and audio PLL (PLLI2S) input
        PLLM OFFSET(0) NUMBITS(6) []
    ],
    CFGR [
        /// Microcontroller clock output 2
        MCO2 OFFSET(30) NUMBITS(2) [],
        /// MCO2 prescaler
        MCO2PRE OFFSET(27) NUMBITS(3) [],
        /// MCO1 prescaler
        MCO1PRE OFFSET(24) NUMBITS(3) [],
        /// I2S clock selection
        I2SSRC OFFSET(23) NUMBITS(1) [],
        /// Microcontroller clock output 1
        MCO1 OFFSET(21) NUMBITS(2) [],
        /// HSE division factor for RTC clock
        RTCPRE OFFSET(16) NUMBITS(5) [],
        /// APB high-speed prescaler (APB2)
        PPRE2 OFFSET(13) NUMBITS(3) [],
        /// APB Low speed prescaler (APB1)
        PPRE1 OFFSET(10) NUMBITS(3) [],
        /// AHB prescaler
        HPRE OFFSET(4) NUMBITS(4) [],
        /// System clock switch status
        SWS OFFSET(2) NUMBITS(2) [],
        /// System clock switch
        SW OFFSET(0) NUMBITS(2) [
            HSI = 0b00,
            HSE = 0b01,
            PLL = 0b10,
        ]
    ],
    CIR [
        /// Clock security system interrupt clear
        CSSC OFFSET(23) NUMBITS(1) [],
        /// PLLSAI Ready Interrupt Clear
        PLLSAIRDYC OFFSET(22) NUMBITS(1) [],
        /// PLLI2S ready interrupt clear
        PLLI2SRDYC OFFSET(21) NUMBITS(1) [],
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
        /// PLLSAI Ready Interrupt Enable
        PLLSAIRDYIE OFFSET(14) NUMBITS(1) [],
        /// PLLI2S ready interrupt enable
        PLLI2SRDYIE OFFSET(13) NUMBITS(1) [],
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
    AHB1RSTR [
        /// USB OTG HS module reset
        OTGHSRST OFFSET(29) NUMBITS(1) [],
        /// DMA2 reset
        DMA2RST OFFSET(22) NUMBITS(1) [],
        /// DMA2 reset
        DMA1RST OFFSET(21) NUMBITS(1) [],
        /// CRC reset
        CRCRST OFFSET(12) NUMBITS(1) [],
        /// IO port H reset
        GPIOHRST OFFSET(7) NUMBITS(1) [],
        /// IO port G reset
        GPIOGRST OFFSET(6) NUMBITS(1) [],
        /// IO port F reset
        GPIOFRST OFFSET(5) NUMBITS(1) [],
        /// IO port E reset
        GPIOERST OFFSET(4) NUMBITS(1) [],
        /// IO port D reset
        GPIODRST OFFSET(3) NUMBITS(1) [],
        /// IO port C reset
        GPIOCRST OFFSET(2) NUMBITS(1) [],
        /// IO port B reset
        GPIOBRST OFFSET(1) NUMBITS(1) [],
        /// IO port A reset
        GPIOARST OFFSET(0) NUMBITS(1) []
    ],
    AHB2RSTR [
        /// USB OTG FS module reset
        OTGFSRST OFFSET(7) NUMBITS(1) [],
        /// RNG module reset
        RNGSRST OFFSET(6) NUMBITS(1) [],
        /// Camera interface reset
        DCMIRST OFFSET(0) NUMBITS(1) []
    ],
    AHB3RSTR [
        /// Flexible memory controller module reset
        FMCRST OFFSET(0) NUMBITS(1) [],
        /// QUADSPI module reset
        QSPIRST OFFSET(1) NUMBITS(1) []
    ],
    APB1RSTR [
        /// TIM2 reset
        TIM2RST OFFSET(0) NUMBITS(1) [],
        /// TIM3 reset
        TIM3RST OFFSET(1) NUMBITS(1) [],
        /// TIM4 reset
        TIM4RST OFFSET(2) NUMBITS(1) [],
        /// TIM5 reset
        TIM5RST OFFSET(3) NUMBITS(1) [],
        /// TIM6 reset
        TIM6RST OFFSET(4) NUMBITS(1) [],
        /// TIM7 reset
        TIM7RST OFFSET(5) NUMBITS(1) [],
        /// TIM12 reset
        TIM12RST OFFSET(6) NUMBITS(1) [],
        /// TIM13 reset
        TIM13RST OFFSET(7) NUMBITS(1) [],
        /// TIM14 reset
        TIM14RST OFFSET(8) NUMBITS(1) [],
        /// Window watchdog reset
        WWDGRST OFFSET(11) NUMBITS(1) [],
        /// SPI 2 reset
        SPI2RST OFFSET(14) NUMBITS(1) [],
        /// SPI 3 reset
        SPI3RST OFFSET(15) NUMBITS(1) [],
        /// SPDIF-IN reset
        SPDIFRST OFFSET(16) NUMBITS(1) [],
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
        /// I2C3 reset
        I2C3RST OFFSET(23) NUMBITS(1) [],
        /// I2CFMP1 reset
        I2CFMP1RST OFFSET(24) NUMBITS(1) [],
        /// CAN1 reset
        CAN1RST OFFSET(25) NUMBITS(1) [],
        /// CAN2 reset
        CAN2RST OFFSET(26) NUMBITS(1) [],
        /// Power interface reset
        PWRRST OFFSET(28) NUMBITS(1) [],
        /// DAC reset
        DACRST OFFSET(29) NUMBITS(1) []
    ],
    APB2RSTR [
        /// TIM1 reset
        TIM1RST OFFSET(0) NUMBITS(1) [],
        /// TIM8 reset
        TIM8RST OFFSET(1) NUMBITS(1) [],
        /// USART1 reset
        USART1RST OFFSET(4) NUMBITS(1) [],
        /// USART6 reset
        USART6RST OFFSET(5) NUMBITS(1) [],
        /// ADC interface reset (common to all ADCs)
        ADCRST OFFSET(8) NUMBITS(1) [],
        /// SDIO reset
        SDIORST OFFSET(11) NUMBITS(1) [],
        /// SPI 1 reset
        SPI1RST OFFSET(12) NUMBITS(1) [],
        /// SPI4 reset
        SPI4RST OFFSET(13) NUMBITS(1) [],
        /// System configuration controller reset
        SYSCFGRST OFFSET(14) NUMBITS(1) [],
        /// TIM9 reset
        TIM9RST OFFSET(16) NUMBITS(1) [],
        /// TIM10 reset
        TIM10RST OFFSET(17) NUMBITS(1) [],
        /// TIM11 reset
        TIM11RST OFFSET(18) NUMBITS(1) [],
        /// SAI1 reset
        SAI1RST OFFSET(22) NUMBITS(1) [],
        /// SAI2 reset
        SAI2RST OFFSET(23) NUMBITS(1) []
    ],
    AHB1ENR [
        /// USB OTG HSULPI clock enable
        OTGHSULPIEN OFFSET(30) NUMBITS(1) [],
        /// USB OTG HS clock enable
        OTGHSEN OFFSET(29) NUMBITS(1) [],
        /// DMA2 clock enable
        DMA2EN OFFSET(22) NUMBITS(1) [],
        /// DMA1 clock enable
        DMA1EN OFFSET(21) NUMBITS(1) [],
        /// Backup SRAM interface clock enable
        BKPSRAMEN OFFSET(18) NUMBITS(1) [],
        /// CRC clock enable
        CRCEN OFFSET(12) NUMBITS(1) [],
        /// IO port H clock enable
        GPIOHEN OFFSET(7) NUMBITS(1) [],
        /// IO port G clock enable
        GPIOGEN OFFSET(6) NUMBITS(1) [],
        /// IO port F clock enable
        GPIOFEN OFFSET(5) NUMBITS(1) [],
        /// IO port E clock enable
        GPIOEEN OFFSET(4) NUMBITS(1) [],
        /// IO port D clock enable
        GPIODEN OFFSET(3) NUMBITS(1) [],
        /// IO port C clock enable
        GPIOCEN OFFSET(2) NUMBITS(1) [],
        /// IO port B clock enable
        GPIOBEN OFFSET(1) NUMBITS(1) [],
        /// IO port A clock enable
        GPIOAEN OFFSET(0) NUMBITS(1) []
    ],
    AHB2ENR [
        /// USB OTG FS clock enable
        OTGFSEN OFFSET(7) NUMBITS(1) [],
        /// RNG clock enable
        RNGEN OFFSET(6) NUMBITS(1) [],
        /// Camera interface enable
        DCMIEN OFFSET(0) NUMBITS(1) []
    ],
    AHB3ENR [
        /// Flexible memory controller module clock enable
        FMCEN OFFSET(0) NUMBITS(1) [],
        /// QUADSPI memory controller module clock enable
        QSPIEN OFFSET(1) NUMBITS(1) []
    ],
    APB1ENR [
        /// TIM2 clock enable
        TIM2EN OFFSET(0) NUMBITS(1) [],
        /// TIM3 clock enable
        TIM3EN OFFSET(1) NUMBITS(1) [],
        /// TIM4 clock enable
        TIM4EN OFFSET(2) NUMBITS(1) [],
        /// TIM5 clock enable
        TIM5EN OFFSET(3) NUMBITS(1) [],
        /// TIM6 clock enable
        TIM6EN OFFSET(4) NUMBITS(1) [],
        /// TIM7 clock enable
        TIM7EN OFFSET(5) NUMBITS(1) [],
        /// TIM12 clock enable
        TIM12EN OFFSET(6) NUMBITS(1) [],
        /// TIM13 clock enable
        TIM13EN OFFSET(7) NUMBITS(1) [],
        /// TIM14 clock enable
        TIM14EN OFFSET(8) NUMBITS(1) [],
        /// Window watchdog clock enable
        WWDGEN OFFSET(11) NUMBITS(1) [],
        /// SPI2 clock enable
        SPI2EN OFFSET(14) NUMBITS(1) [],
        /// SPI3 clock enable
        SPI3EN OFFSET(15) NUMBITS(1) [],
        /// SPDIF-IN clock enable
        SPDIFEN OFFSET(16) NUMBITS(1) [],
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
        /// I2C3 clock enable
        I2C3EN OFFSET(23) NUMBITS(1) [],
        /// I2CFMP1 clock enable
        I2CFMP1EN OFFSET(24) NUMBITS(1) [],
        /// CAN 1 clock enable
        CAN1EN OFFSET(25) NUMBITS(1) [],
        /// CAN 2 clock enable
        CAN2EN OFFSET(26) NUMBITS(1) [],
        /// CEC interface clock enable
        CEC OFFSET(27) NUMBITS(1) [],
        /// Power interface clock enable
        PWREN OFFSET(28) NUMBITS(1) [],
        /// DAC interface clock enable
        DACEN OFFSET(29) NUMBITS(1) []
    ],
    APB2ENR [
        /// TIM1 clock enable
        TIM1EN OFFSET(0) NUMBITS(1) [],
        /// TIM8 clock enable
        TIM8EN OFFSET(1) NUMBITS(1) [],
        /// USART1 clock enable
        USART1EN OFFSET(4) NUMBITS(1) [],
        /// USART6 clock enable
        USART6EN OFFSET(5) NUMBITS(1) [],
        /// ADC1 clock enable
        ADC1EN OFFSET(8) NUMBITS(1) [],
        /// ADC2 clock enable
        ADC2EN OFFSET(9) NUMBITS(1) [],
        /// ADC3 clock enable
        ADC3EN OFFSET(10) NUMBITS(1) [],
        /// SDIO clock enable
        SDIOEN OFFSET(11) NUMBITS(1) [],
        /// SPI1 clock enable
        SPI1EN OFFSET(12) NUMBITS(1) [],
        /// SPI4 clock enable
        SPI4ENR OFFSET(13) NUMBITS(1) [],
        /// System configuration controller clock enable
        SYSCFGEN OFFSET(14) NUMBITS(1) [],
        /// TIM9 clock enable
        TIM9EN OFFSET(16) NUMBITS(1) [],
        /// TIM10 clock enable
        TIM10EN OFFSET(17) NUMBITS(1) [],
        /// TIM11 clock enable
        TIM11EN OFFSET(18) NUMBITS(1) [],
        /// SAI1 clock enable
        SAI1EN OFFSET(22) NUMBITS(1) [],
        /// SAI2 clock enable
        SAI2EN OFFSET(23) NUMBITS(1) []
    ],
    AHB1LPENR [
        /// IO port A clock enable during sleep mode
        GPIOALPEN OFFSET(0) NUMBITS(1) [],
        /// IO port B clock enable during Sleep mode
        GPIOBLPEN OFFSET(1) NUMBITS(1) [],
        /// IO port C clock enable during Sleep mode
        GPIOCLPEN OFFSET(2) NUMBITS(1) [],
        /// IO port D clock enable during Sleep mode
        GPIODLPEN OFFSET(3) NUMBITS(1) [],
        /// IO port E clock enable during Sleep mode
        GPIOELPEN OFFSET(4) NUMBITS(1) [],
        /// IO port F clock enable during Sleep mode
        GPIOFLPEN OFFSET(5) NUMBITS(1) [],
        /// IO port G clock enable during Sleep mode
        GPIOGLPEN OFFSET(6) NUMBITS(1) [],
        /// IO port H clock enable during Sleep mode
        GPIOHLPEN OFFSET(7) NUMBITS(1) [],
        /// CRC clock enable during Sleep mode
        CRCLPEN OFFSET(12) NUMBITS(1) [],
        /// Flash interface clock enable during Sleep mode
        FLITFLPEN OFFSET(15) NUMBITS(1) [],
        /// SRAM 1interface clock enable during Sleep mode
        SRAM1LPEN OFFSET(16) NUMBITS(1) [],
        /// SRAM 2 interface clock enable during Sleep mode
        SRAM2LPEN OFFSET(17) NUMBITS(1) [],
        /// Backup SRAM interface clock enable during Sleep mode
        BKPSRAMLPEN OFFSET(18) NUMBITS(1) [],
        /// DMA1 clock enable during Sleep mode
        DMA1LPEN OFFSET(21) NUMBITS(1) [],
        /// DMA2 clock enable during Sleep mode
        DMA2LPEN OFFSET(22) NUMBITS(1) [],
        /// USB OTG HS clock enable during Sleep mode
        OTGHSLPEN OFFSET(29) NUMBITS(1) [],
        /// USB OTG HS ULPI clock enable during Sleep mode
        OTGHSULPILPEN OFFSET(30) NUMBITS(1) []
    ],
    AHB2LPENR [
        /// USB OTG FS clock enable during Sleep mode
        OTGFSLPEN OFFSET(7) NUMBITS(1) [],
        /// RNG clock enable during Sleep mode
        RNGLPEN OFFSET(6) NUMBITS(1) [],
        /// Camera interface enable during Sleep mode
        DCMILPEN OFFSET(0) NUMBITS(1) []
    ],
    AHB3LPENR [
        /// Flexible memory controller module clock enable during Sleep mode
        FMCLPEN OFFSET(0) NUMBITS(1) [],
        /// QUADSPI memory controller module clock enable during Sleep mode
        QSPILPEN OFFSET(1) NUMBITS(1) []
    ],
    APB1LPENR [
        /// TIM2 clock enable during Sleep mode
        TIM2LPEN OFFSET(0) NUMBITS(1) [],
        /// TIM3 clock enable during Sleep mode
        TIM3LPEN OFFSET(1) NUMBITS(1) [],
        /// TIM4 clock enable during Sleep mode
        TIM4LPEN OFFSET(2) NUMBITS(1) [],
        /// TIM5 clock enable during Sleep mode
        TIM5LPEN OFFSET(3) NUMBITS(1) [],
        /// TIM6 clock enable during Sleep mode
        TIM6LPEN OFFSET(4) NUMBITS(1) [],
        /// TIM7 clock enable during Sleep mode
        TIM7LPEN OFFSET(5) NUMBITS(1) [],
        /// TIM12 clock enable during Sleep mode
        TIM12LPEN OFFSET(6) NUMBITS(1) [],
        /// TIM13 clock enable during Sleep mode
        TIM13LPEN OFFSET(7) NUMBITS(1) [],
        /// TIM14 clock enable during Sleep mode
        TIM14LPEN OFFSET(8) NUMBITS(1) [],
        /// Window watchdog clock enable during Sleep mode
        WWDGLPEN OFFSET(11) NUMBITS(1) [],
        /// SPI2 clock enable during Sleep mode
        SPI2LPEN OFFSET(14) NUMBITS(1) [],
        /// SPI3 clock enable during Sleep mode
        SPI3LPEN OFFSET(15) NUMBITS(1) [],
        /// SPDIF clock enable during Sleep mode
        SPDIFLPEN OFFSET(16) NUMBITS(1) [],
        /// USART2 clock enable during Sleep mode
        USART2LPEN OFFSET(17) NUMBITS(1) [],
        /// USART3 clock enable during Sleep mode
        USART3LPEN OFFSET(18) NUMBITS(1) [],
        /// UART4 clock enable during Sleep mode
        UART4LPEN OFFSET(19) NUMBITS(1) [],
        /// UART5 clock enable during Sleep mode
        UART5LPEN OFFSET(20) NUMBITS(1) [],
        /// I2C1 clock enable during Sleep mode
        I2C1LPEN OFFSET(21) NUMBITS(1) [],
        /// I2C2 clock enable during Sleep mode
        I2C2LPEN OFFSET(22) NUMBITS(1) [],
        /// I2C3 clock enable during Sleep mode
        I2C3LPEN OFFSET(23) NUMBITS(1) [],
        /// I2CFMP1 clock enable during Sleep mode
        I2CFMP1LPEN OFFSET(24) NUMBITS(1) [],
        /// CAN 1 clock enable during Sleep mode
        CAN1LPEN OFFSET(25) NUMBITS(1) [],
        /// CAN 2 clock enable during Sleep mode
        CAN2LPEN OFFSET(26) NUMBITS(1) [],
        /// CEC clock enable during Sleep mode
        CECLPEN OFFSET(27) NUMBITS(1) [],
        /// Power interface clock enable during Sleep mode
        PWRLPEN OFFSET(28) NUMBITS(1) [],
        /// DAC interface clock enable during Sleep mode
        DACLPEN OFFSET(29) NUMBITS(1) []
    ],
    APB2LPENR [
        /// TIM1 clock enable during Sleep mode
        TIM1LPEN OFFSET(0) NUMBITS(1) [],
        /// TIM8 clock enable during Sleep mode
        TIM8LPEN OFFSET(1) NUMBITS(1) [],
        /// USART1 clock enable during Sleep mode
        USART1LPEN OFFSET(4) NUMBITS(1) [],
        /// USART6 clock enable during Sleep mode
        USART6LPEN OFFSET(5) NUMBITS(1) [],
        /// ADC1 clock enable during Sleep mode
        ADC1LPEN OFFSET(8) NUMBITS(1) [],
        /// ADC2 clock enable during Sleep mode
        ADC2LPEN OFFSET(9) NUMBITS(1) [],
        /// ADC 3 clock enable during Sleep mode
        ADC3LPEN OFFSET(10) NUMBITS(1) [],
        /// SDIO clock enable during Sleep mode
        SDIOLPEN OFFSET(11) NUMBITS(1) [],
        /// SPI 1 clock enable during Sleep mode
        SPI1LPEN OFFSET(12) NUMBITS(1) [],
        /// SPI 4 clock enable during Sleep mode
        SPI4LPEN OFFSET(13) NUMBITS(1) [],
        /// System configuration controller clock enable during Sleep mode
        SYSCFGLPEN OFFSET(14) NUMBITS(1) [],
        /// TIM9 clock enable during sleep mode
        TIM9LPEN OFFSET(16) NUMBITS(1) [],
        /// TIM10 clock enable during Sleep mode
        TIM10LPEN OFFSET(17) NUMBITS(1) [],
        /// TIM11 clock enable during Sleep mode
        TIM11LPEN OFFSET(18) NUMBITS(1) [],
        /// SAI1 clock enable
        SAI1LPEN OFFSET(22) NUMBITS(1) [],
        /// SAI2 clock enable
        SAI2LPEN OFFSET(23) NUMBITS(1) []
    ],
    BDCR [
        /// Backup domain software reset
        BDRST OFFSET(16) NUMBITS(1) [],
        /// RTC clock enable
        RTCEN OFFSET(15) NUMBITS(1) [],
        /// RTC clock source selection
        RTCSEL OFFSET(8) NUMBITS(2) [],
        /// External low-speed oscillator mode
        LSEMOD OFFSET(3) NUMBITS(1) [],
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
        WDGRSTF OFFSET(29) NUMBITS(1) [],
        /// Software reset flag
        SFTRSTF OFFSET(28) NUMBITS(1) [],
        /// POR/PDR reset flag
        PORRSTF OFFSET(27) NUMBITS(1) [],
        /// PIN reset flag
        PADRSTF OFFSET(26) NUMBITS(1) [],
        /// BOR reset flag
        BORRSTF OFFSET(25) NUMBITS(1) [],
        /// Remove reset flag
        RMVF OFFSET(24) NUMBITS(1) [],
        /// Internal low-speed oscillator ready
        LSIRDY OFFSET(1) NUMBITS(1) [],
        /// Internal low-speed oscillator enable
        LSION OFFSET(0) NUMBITS(1) []
    ],
    SSCGR [
        /// Spread spectrum modulation enable
        SSCGEN OFFSET(31) NUMBITS(1) [],
        /// Spread Select
        SPREADSEL OFFSET(30) NUMBITS(1) [],
        /// Incrementation step
        INCSTEP OFFSET(13) NUMBITS(15) [],
        /// Modulation period
        MODPER OFFSET(0) NUMBITS(13) []
    ],
    PLLI2SCFGR [
        /// Division factor for audio PLL (PLLI2S) input clock
        PLLI2SM OFFSET(0) NUMBITS(6) [],
        /// PLLI2S multiplication factor for VCO
        PLLI2SN OFFSET(6) NUMBITS(9) [],
        /// PLLI2S division factor for SPDIF-IN clock
        PLLI2SP OFFSET(16) NUMBITS(2) [],
        /// PLLI2S division factor for SAI1 clock
        PLLI2SQ OFFSET(24) NUMBITS(4) [],
        /// PLLI2S division factor for I2S clocks
        PLLI2SR OFFSET(28) NUMBITS(3) []
    ],
    PLLSAICFGR [
        /// Division factor for audio PLLSAI input clock
        PLLSAIM OFFSET(0) NUMBITS(6) [],
        /// PLLSAI division factor for VCO
        PLLSAIN OFFSET(6) NUMBITS(9) [],
        /// PLLSAI division factor for 48 MHz clock
        PLLSAIP OFFSET(16) NUMBITS(2) [],
        /// PLLSAI division factor for SAIs clock
        PLLSAIQ OFFSET(24) NUMBITS(4) []
    ],
    DCKCFGR [
        /// PLLI2S division factor for SAIs clock
        PLLI2SDIVQ OFFSET(0) NUMBITS(5) [],
        /// PLLSAI division factor for SAIs clock
        PLLSAIDIVQ OFFSET(8) NUMBITS(5) [],
        /// SAI1 clock source selection
        SAI1SRC OFFSET(20) NUMBITS(2) [],
        /// SAI2 clock source selection
        SAI2SRC OFFSET(22) NUMBITS(2) [],
        /// Timers clocks prescalers selection
        TIMPRE OFFSET(24) NUMBITS(1) [],
        /// I2S APB1 clock source selection
        I2S1SRC OFFSET(25) NUMBITS(2) [],
        /// I2S APB2 clock source selection
        I2S2SRC OFFSET(27) NUMBITS(2) []
    ],
    CKGATENR [
        /// AHB to APB1 Bridge clock enable
        AHB2APB1_CKEN OFFSET(0) NUMBITS(1) [],
        /// AHB to APB2 Bridge clock enable
        AHB2APB2_CKEN OFFSET(1) NUMBITS(1) [],
        /// Cortex M4 ETM clock enable
        CM4DBG_CKEN OFFSET(2) NUMBITS(1) [],
        /// Spare clock enable
        SPARE_CKEN OFFSET(3) NUMBITS(1) [],
        /// SRQAM controller clock enable
        SRAM_CKEN OFFSET(4) NUMBITS(1) [],
        /// Flash Interface clock enable
        FLITF_CKEN OFFSET(5) NUMBITS(1) [],
        /// RCC clock enable
        RCC_CKEN OFFSET(6) NUMBITS(1) []
    ],
    DCKCFGR2 [
        /// I2C4 kernel clock source selection
        FMPI2C1SEL OFFSET(22) NUMBITS(2) [],
        /// HDMI CEC clock source selection
        CECSEL OFFSET(26) NUMBITS(1) [],
        /// SDIO/USBFS/HS clock selection
        CK48MSEL OFFSET(27) NUMBITS(1) [],
        /// SDIO clock selection
        SDIOSEL OFFSET(28) NUMBITS(1) [],
        /// SPDIF clock selection
        SPDIFSEL OFFSET(29) NUMBITS(1) []
    ]
];

const RCC_BASE: StaticRef<RccRegisters> =
    unsafe { StaticRef::new(0x40023800 as *const RccRegisters) };

// Default values when the hardware is reset. Uncomment if you need them.
//pub(crate) const RESET_PLLM_VALUE: usize = PLLM::DivideBy16; // M = 16
//pub(crate) const RESET_PLLP_VALUE: PLLP = PLLP::DivideBy2; // P = 2
//pub(crate) const RESET_PLLQ_VALUE: PLLQ = PLLQ::DivideBy4; // Q = 4
pub(crate) const RESET_PLLN_VALUE: usize = 0b011_000_000; // N = 192

// Default PLL configuration. See Rcc::init_pll_clock() for more details.
//
// Choose PLLM::DivideBy8 for reduced PLL jitter or PLLM::DivideBy16 for default hardware
// configuration
pub(crate) const DEFAULT_PLLM_VALUE: PLLM = PLLM::DivideBy8;
// DON'T CHANGE THIS VALUE
pub(crate) const DEFAULT_PLLN_VALUE: usize = RESET_PLLN_VALUE;
// Dynamically computing the default PLLP value based on the PLLM value
pub(crate) const DEFAULT_PLLP_VALUE: PLLP = match DEFAULT_PLLM_VALUE {
    PLLM::DivideBy16 => PLLP::DivideBy2,
    PLLM::DivideBy8 => PLLP::DivideBy4,
};
// Dynamically computing the default PLLQ value based on the PLLM value
pub(crate) const DEFAULT_PLLQ_VALUE: PLLQ = match DEFAULT_PLLM_VALUE {
    PLLM::DivideBy16 => PLLQ::DivideBy4,
    PLLM::DivideBy8 => PLLQ::DivideBy8,
};

pub struct Rcc {
    registers: StaticRef<RccRegisters>,
}

impl Rcc {
    pub fn new() -> Self {
        let rcc = Self {
            registers: RCC_BASE,
        };
        rcc.init();
        rcc
    }

    // Some clocks need to be initialized before use
    fn init(&self) {
        self.init_pll_clock();
    }

    // Init the PLL clock. The default configuration:
    // + if DEFAULT_PLLM_VALUE == PLLM::DivideBy8:
    //   + 2MHz VCO input frequency for reduced PLL jitter: freq_VCO_input = freq_source / PLLM
    //   + 384MHz VCO output frequency: freq_VCO_output = freq_VCO_input * PLLN
    //   + 96MHz main output frequency: freq_PLL = freq_VCO_output / PLLP
    //   + 48MHz PLL48CLK output frequency: freq_PLL48CLK = freq_VCO_output / PLLQ
    // + if DEFAULT_PLLM_VALUE == PLLM::DivideBy16: (default hardware configuration)
    //   + 1MHz VCO input frequency for reduced PLL jitter: freq_VCO_input = freq_source / PLLM
    //   + 384MHz VCO output frequency: freq_VCO_output = freq_VCO_input * PLLN
    //   + 96MHz main output frequency: freq_PLL = freq_VCO_output / PLLP
    //   + 48MHz PLL48CLK output frequency: freq_PLL48CLK = freq_VCO_output / PLLQ
    fn init_pll_clock(&self) {
        self.set_pll_clocks_source(PllSource::HSI);
        self.set_pll_clocks_m_divider(DEFAULT_PLLM_VALUE);
        self.set_pll_clock_n_multiplier(DEFAULT_PLLN_VALUE);
        self.set_pll_clock_p_divider(DEFAULT_PLLP_VALUE);
        self.set_pll_clock_q_divider(DEFAULT_PLLQ_VALUE);
    }

    // Get the current system clock source
    pub(crate) fn get_sys_clock_source(&self) -> SysClockSource {
        // Can't panic because the SysClockSource is based on the hardware possible values
        SysClockSource::try_from(self.registers.cfgr.read(CFGR::SWS)).unwrap()
    }

    pub(crate) fn is_hsi_clock_system_clock(&self) -> bool {
        let system_clock_source = self.get_sys_clock_source();
        system_clock_source == SysClockSource::HSI ||
            system_clock_source == SysClockSource::PLLCLK &&
            self.registers.pllcfgr.read(PLLCFGR::PLLSRC) == PllSource::HSI as u32
    }

    /* HSI clock */
    // The HSI clock must not be configured as the sistem clock, either directly or indirectly.
    pub(crate) fn disable_hsi_clock(&self) {
        self.registers.cr.modify(CR::HSION::CLEAR);
    }

    pub(crate) fn enable_hsi_clock(&self) {
        self.registers.cr.modify(CR::HSION::SET);
    }

    pub(crate) fn is_enabled_hsi_clock(&self) -> bool {
        self.registers.cr.is_set(CR::HSION)
    }

    // Indicates whether the HSI oscillator is stable
    pub(crate) fn is_ready_hsi_clock(&self) -> bool {
        self.registers.cr.is_set(CR::HSIRDY)
    }

    /* Main PLL clock*/

    // The main PLL clock must not be configured as the sistem clock.
    pub(crate) fn disable_pll_clock(&self) {
        self.registers.cr.modify(CR::PLLON::CLEAR);
    }

    pub(crate) fn enable_pll_clock(&self) {
        self.registers.cr.modify(CR::PLLON::SET);
    }

    pub(crate) fn is_enabled_pll_clock(&self) -> bool {
        self.registers.cr.is_set(CR::PLLON)
    }

    // The PLL clock is locked when its signal is stable
    pub(crate) fn is_locked_pll_clock(&self) -> bool {
        self.registers.cr.is_set(CR::PLLRDY)
    }

    // This method must be called only when all PLL clocks are disabled
    pub(crate) fn set_pll_clocks_source(&self, source: PllSource) {
        self.registers
            .pllcfgr
            .modify(PLLCFGR::PLLSRC.val(source as u32));
    }

    // This method must be called only when all PLL clocks are disabled
    pub(crate) fn set_pll_clocks_m_divider(&self, m: PLLM) {
        self.registers.pllcfgr.modify(PLLCFGR::PLLM.val(m as u32));
    }

    // This method must be called only if the main PLL clock is disabled
    pub(crate) fn set_pll_clock_n_multiplier(&self, n: usize) {
        self.registers.pllcfgr.modify(PLLCFGR::PLLN.val(n as u32));
    }

    // This method must be called only if the main PLL clock is disabled
    pub(crate) fn set_pll_clock_p_divider(&self, p: PLLP) {
        self.registers.pllcfgr.modify(PLLCFGR::PLLP.val(p as u32));
    }

    // This method must be called only if the main PLL clock is disabled
    pub(crate) fn set_pll_clock_q_divider(&self, q: PLLQ) {
        self.registers.pllcfgr.modify(PLLCFGR::PLLQ.val(q as u32));
    }

    fn configure_rng_clock(&self) {
        self.registers.pllcfgr.modify(PLLCFGR::PLLQ.val(2));
        self.registers.cr.modify(CR::PLLON::SET);
    }

    // I2C1 clock

    fn is_enabled_i2c1_clock(&self) -> bool {
        self.registers.apb1enr.is_set(APB1ENR::I2C1EN)
    }

    fn enable_i2c1_clock(&self) {
        self.registers.apb1enr.modify(APB1ENR::I2C1EN::SET);
        self.registers.apb1rstr.modify(APB1RSTR::I2C1RST::SET);
        self.registers.apb1rstr.modify(APB1RSTR::I2C1RST::CLEAR);
    }

    fn disable_i2c1_clock(&self) {
        self.registers.apb1enr.modify(APB1ENR::I2C1EN::CLEAR)
    }

    // SPI3 clock

    fn is_enabled_spi3_clock(&self) -> bool {
        self.registers.apb1enr.is_set(APB1ENR::SPI3EN)
    }

    fn enable_spi3_clock(&self) {
        self.registers.apb1enr.modify(APB1ENR::SPI3EN::SET)
    }

    fn disable_spi3_clock(&self) {
        self.registers.apb1enr.modify(APB1ENR::SPI3EN::CLEAR)
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
        self.registers.ahb1enr.is_set(AHB1ENR::DMA1EN)
    }

    fn enable_dma1_clock(&self) {
        self.registers.ahb1enr.modify(AHB1ENR::DMA1EN::SET)
    }

    fn disable_dma1_clock(&self) {
        self.registers.ahb1enr.modify(AHB1ENR::DMA1EN::CLEAR)
    }

    // DMA2 clock
    fn is_enabled_dma2_clock(&self) -> bool {
        self.registers.ahb1enr.is_set(AHB1ENR::DMA2EN)
    }

    fn enable_dma2_clock(&self) {
        self.registers.ahb1enr.modify(AHB1ENR::DMA2EN::SET)
    }

    fn disable_dma2_clock(&self) {
        self.registers.ahb1enr.modify(AHB1ENR::DMA2EN::CLEAR)
    }

    // GPIOH clock

    fn is_enabled_gpioh_clock(&self) -> bool {
        self.registers.ahb1enr.is_set(AHB1ENR::GPIOHEN)
    }

    fn enable_gpioh_clock(&self) {
        self.registers.ahb1enr.modify(AHB1ENR::GPIOHEN::SET)
    }

    fn disable_gpioh_clock(&self) {
        self.registers.ahb1enr.modify(AHB1ENR::GPIOHEN::CLEAR)
    }

    // GPIOG clock

    fn is_enabled_gpiog_clock(&self) -> bool {
        self.registers.ahb1enr.is_set(AHB1ENR::GPIOGEN)
    }

    fn enable_gpiog_clock(&self) {
        self.registers.ahb1enr.modify(AHB1ENR::GPIOGEN::SET)
    }

    fn disable_gpiog_clock(&self) {
        self.registers.ahb1enr.modify(AHB1ENR::GPIOGEN::CLEAR)
    }

    // GPIOF clock

    fn is_enabled_gpiof_clock(&self) -> bool {
        self.registers.ahb1enr.is_set(AHB1ENR::GPIOFEN)
    }

    fn enable_gpiof_clock(&self) {
        self.registers.ahb1enr.modify(AHB1ENR::GPIOFEN::SET)
    }

    fn disable_gpiof_clock(&self) {
        self.registers.ahb1enr.modify(AHB1ENR::GPIOFEN::CLEAR)
    }

    // GPIOE clock

    fn is_enabled_gpioe_clock(&self) -> bool {
        self.registers.ahb1enr.is_set(AHB1ENR::GPIOEEN)
    }

    fn enable_gpioe_clock(&self) {
        self.registers.ahb1enr.modify(AHB1ENR::GPIOEEN::SET)
    }

    fn disable_gpioe_clock(&self) {
        self.registers.ahb1enr.modify(AHB1ENR::GPIOEEN::CLEAR)
    }

    // GPIOD clock

    fn is_enabled_gpiod_clock(&self) -> bool {
        self.registers.ahb1enr.is_set(AHB1ENR::GPIODEN)
    }

    fn enable_gpiod_clock(&self) {
        self.registers.ahb1enr.modify(AHB1ENR::GPIODEN::SET)
    }

    fn disable_gpiod_clock(&self) {
        self.registers.ahb1enr.modify(AHB1ENR::GPIODEN::CLEAR)
    }

    // GPIOC clock

    fn is_enabled_gpioc_clock(&self) -> bool {
        self.registers.ahb1enr.is_set(AHB1ENR::GPIOCEN)
    }

    fn enable_gpioc_clock(&self) {
        self.registers.ahb1enr.modify(AHB1ENR::GPIOCEN::SET)
    }

    fn disable_gpioc_clock(&self) {
        self.registers.ahb1enr.modify(AHB1ENR::GPIOCEN::CLEAR)
    }

    // GPIOB clock

    fn is_enabled_gpiob_clock(&self) -> bool {
        self.registers.ahb1enr.is_set(AHB1ENR::GPIOBEN)
    }

    fn enable_gpiob_clock(&self) {
        self.registers.ahb1enr.modify(AHB1ENR::GPIOBEN::SET)
    }

    fn disable_gpiob_clock(&self) {
        self.registers.ahb1enr.modify(AHB1ENR::GPIOBEN::CLEAR)
    }

    // GPIOA clock

    fn is_enabled_gpioa_clock(&self) -> bool {
        self.registers.ahb1enr.is_set(AHB1ENR::GPIOAEN)
    }

    fn enable_gpioa_clock(&self) {
        self.registers.ahb1enr.modify(AHB1ENR::GPIOAEN::SET)
    }

    fn disable_gpioa_clock(&self) {
        self.registers.ahb1enr.modify(AHB1ENR::GPIOAEN::CLEAR)
    }

    // FMC

    fn is_enabled_fmc_clock(&self) -> bool {
        self.registers.ahb3enr.is_set(AHB3ENR::FMCEN)
    }

    fn enable_fmc_clock(&self) {
        self.registers.ahb3enr.modify(AHB3ENR::FMCEN::SET)
    }

    fn disable_fmc_clock(&self) {
        self.registers.ahb3enr.modify(AHB3ENR::FMCEN::CLEAR)
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

    // ADC1 clock

    fn is_enabled_adc1_clock(&self) -> bool {
        self.registers.apb2enr.is_set(APB2ENR::ADC1EN)
    }

    fn enable_adc1_clock(&self) {
        self.registers.apb2enr.modify(APB2ENR::ADC1EN::SET)
    }

    fn disable_adc1_clock(&self) {
        self.registers.apb2enr.modify(APB2ENR::ADC1EN::CLEAR)
    }

    // RNG clock

    fn is_enabled_rng_clock(&self) -> bool {
        self.registers.ahb2enr.is_set(AHB2ENR::RNGEN)
    }

    fn enable_rng_clock(&self) {
        self.registers.ahb2enr.modify(AHB2ENR::RNGEN::SET);
    }

    fn disable_rng_clock(&self) {
        self.registers.ahb2enr.modify(AHB2ENR::RNGEN::CLEAR);
    }

    // OTGFS clock

    fn is_enabled_otgfs_clock(&self) -> bool {
        self.registers.ahb2enr.is_set(AHB2ENR::OTGFSEN)
    }

    fn enable_otgfs_clock(&self) {
        self.registers.ahb2enr.modify(AHB2ENR::OTGFSEN::SET);
    }

    fn disable_otgfs_clock(&self) {
        self.registers.ahb2enr.modify(AHB2ENR::OTGFSEN::CLEAR);
    }

    // CAN1 clock

    fn is_enabled_can1_clock(&self) -> bool {
        self.registers.apb1enr.is_set(APB1ENR::CAN1EN)
    }

    fn enable_can1_clock(&self) {
        self.registers.apb1rstr.modify(APB1RSTR::CAN1RST::SET);
        self.registers.apb1rstr.modify(APB1RSTR::CAN1RST::CLEAR);
        self.registers.apb1enr.modify(APB1ENR::CAN1EN::SET);
    }

    fn disable_can1_clock(&self) {
        self.registers.apb1enr.modify(APB1ENR::CAN1EN::CLEAR);
    }
}

// NOTE: HSE is not yet supported as source clock.
pub(crate) enum PllSource {
    HSI = 0b0,
    //HSE = 0b1,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum PLLP {
    DivideBy2 = 0b00,
    DivideBy4 = 0b01,
    DivideBy6 = 0b10,
    DivideBy8 = 0b11,
}

// Theoretically, the PLLM value can range from 2 to 63. However, the current implementation was
// designed to support 1MHz frequency precision. In a future update, PLLM will become a usize.
#[allow(dead_code)]
pub(crate) enum PLLM {
    DivideBy8 = 8,
    DivideBy16 = 16,
}

#[derive(Copy, Clone, Debug, PartialEq)]
// Due to the restricted values for PLLM, PLLQ 10-15 values are meaningless.
pub(crate) enum PLLQ {
    DivideBy2 = 2,
    DivideBy3,
    DivideBy4,
    DivideBy5,
    DivideBy6,
    DivideBy7,
    DivideBy8,
    DivideBy9,
}

impl TryFrom<usize> for PLLQ {
    type Error = &'static str;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            2 => Ok(PLLQ::DivideBy2),
            3 => Ok(PLLQ::DivideBy3),
            4 => Ok(PLLQ::DivideBy4),
            5 => Ok(PLLQ::DivideBy5),
            6 => Ok(PLLQ::DivideBy6),
            7 => Ok(PLLQ::DivideBy7),
            8 => Ok(PLLQ::DivideBy8),
            9 => Ok(PLLQ::DivideBy9),
            _ => Err("Invalid value for PLLQ::try_from"),
        }
    }
}

/// Clock sources for the CPU
#[derive(PartialEq)]
pub enum SysClockSource {
    HSI = 0b00,
    HSE = 0b01,
    PLLCLK = 0b10,
    // NOTE: not all STM32F4xx boards support this source.
    //PPLLR,
}

impl TryFrom<u32> for SysClockSource {
    type Error = &'static str;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0b00 => Ok(SysClockSource::HSI),
            0b01 => Ok(SysClockSource::HSE),
            0b10 => Ok(SysClockSource::PLLCLK),
            _ => Err("Invalid value for SysClockSource::try_from"),
        }
    }
}

pub struct PeripheralClock<'a> {
    pub clock: PeripheralClockType,
    rcc: &'a Rcc,
}

/// Bus + Clock name for the peripherals
pub enum PeripheralClockType {
    AHB1(HCLK1),
    AHB2(HCLK2),
    AHB3(HCLK3),
    APB1(PCLK1),
    APB2(PCLK2),
}

/// Peripherals clocked by HCLK1
pub enum HCLK1 {
    DMA1,
    DMA2,
    GPIOH,
    GPIOG,
    GPIOF,
    GPIOE,
    GPIOD,
    GPIOC,
    GPIOB,
    GPIOA,
}

/// Peripherals clocked by HCLK3
pub enum HCLK3 {
    FMC,
}

/// Peripherals clocked by HCLK2
pub enum HCLK2 {
    RNG,
    OTGFS,
}

/// Peripherals clocked by PCLK1
pub enum PCLK1 {
    TIM2,
    USART2,
    USART3,
    SPI3,
    I2C1,
    CAN1,
}

/// Peripherals clocked by PCLK2
pub enum PCLK2 {
    USART1,
    ADC1,
    SYSCFG,
}

impl<'a> PeripheralClock<'a> {
    pub const fn new(clock: PeripheralClockType, rcc: &'a Rcc) -> Self {
        Self { clock, rcc }
    }

    pub fn configure_rng_clock(&self) {
        self.rcc.configure_rng_clock();
    }
}

impl<'a> ClockInterface for PeripheralClock<'a> {
    fn is_enabled(&self) -> bool {
        match self.clock {
            PeripheralClockType::AHB1(ref v) => match v {
                HCLK1::DMA1 => self.rcc.is_enabled_dma1_clock(),
                HCLK1::DMA2 => self.rcc.is_enabled_dma2_clock(),
                HCLK1::GPIOH => self.rcc.is_enabled_gpioh_clock(),
                HCLK1::GPIOG => self.rcc.is_enabled_gpiog_clock(),
                HCLK1::GPIOF => self.rcc.is_enabled_gpiof_clock(),
                HCLK1::GPIOE => self.rcc.is_enabled_gpioe_clock(),
                HCLK1::GPIOD => self.rcc.is_enabled_gpiod_clock(),
                HCLK1::GPIOC => self.rcc.is_enabled_gpioc_clock(),
                HCLK1::GPIOB => self.rcc.is_enabled_gpiob_clock(),
                HCLK1::GPIOA => self.rcc.is_enabled_gpioa_clock(),
            },
            PeripheralClockType::AHB2(ref v) => match v {
                HCLK2::RNG => self.rcc.is_enabled_rng_clock(),
                HCLK2::OTGFS => self.rcc.is_enabled_otgfs_clock(),
            },
            PeripheralClockType::AHB3(ref v) => match v {
                HCLK3::FMC => self.rcc.is_enabled_fmc_clock(),
            },
            PeripheralClockType::APB1(ref v) => match v {
                PCLK1::TIM2 => self.rcc.is_enabled_tim2_clock(),
                PCLK1::USART2 => self.rcc.is_enabled_usart2_clock(),
                PCLK1::USART3 => self.rcc.is_enabled_usart3_clock(),
                PCLK1::I2C1 => self.rcc.is_enabled_i2c1_clock(),
                PCLK1::SPI3 => self.rcc.is_enabled_spi3_clock(),
                PCLK1::CAN1 => self.rcc.is_enabled_can1_clock(),
            },
            PeripheralClockType::APB2(ref v) => match v {
                PCLK2::USART1 => self.rcc.is_enabled_usart1_clock(),
                PCLK2::ADC1 => self.rcc.is_enabled_adc1_clock(),
                PCLK2::SYSCFG => self.rcc.is_enabled_syscfg_clock(),
            },
        }
    }

    fn enable(&self) {
        match self.clock {
            PeripheralClockType::AHB1(ref v) => match v {
                HCLK1::DMA1 => {
                    self.rcc.enable_dma1_clock();
                }
                HCLK1::DMA2 => {
                    self.rcc.enable_dma2_clock();
                }
                HCLK1::GPIOH => {
                    self.rcc.enable_gpioh_clock();
                }
                HCLK1::GPIOG => {
                    self.rcc.enable_gpiog_clock();
                }
                HCLK1::GPIOF => {
                    self.rcc.enable_gpiof_clock();
                }
                HCLK1::GPIOE => {
                    self.rcc.enable_gpioe_clock();
                }
                HCLK1::GPIOD => {
                    self.rcc.enable_gpiod_clock();
                }
                HCLK1::GPIOC => {
                    self.rcc.enable_gpioc_clock();
                }
                HCLK1::GPIOB => {
                    self.rcc.enable_gpiob_clock();
                }
                HCLK1::GPIOA => {
                    self.rcc.enable_gpioa_clock();
                }
            },
            PeripheralClockType::AHB2(ref v) => match v {
                HCLK2::RNG => {
                    self.rcc.enable_rng_clock();
                }
                HCLK2::OTGFS => {
                    self.rcc.enable_otgfs_clock();
                }
            },
            PeripheralClockType::AHB3(ref v) => match v {
                HCLK3::FMC => self.rcc.enable_fmc_clock(),
            },
            PeripheralClockType::APB1(ref v) => match v {
                PCLK1::TIM2 => {
                    self.rcc.enable_tim2_clock();
                }
                PCLK1::USART2 => {
                    self.rcc.enable_usart2_clock();
                }
                PCLK1::USART3 => {
                    self.rcc.enable_usart3_clock();
                }
                PCLK1::I2C1 => {
                    self.rcc.enable_i2c1_clock();
                }
                PCLK1::SPI3 => {
                    self.rcc.enable_spi3_clock();
                }
                PCLK1::CAN1 => {
                    self.rcc.enable_can1_clock();
                }
            },
            PeripheralClockType::APB2(ref v) => match v {
                PCLK2::USART1 => {
                    self.rcc.enable_usart1_clock();
                }
                PCLK2::ADC1 => {
                    self.rcc.enable_adc1_clock();
                }
                PCLK2::SYSCFG => {
                    self.rcc.enable_syscfg_clock();
                }
            },
        }
    }

    fn disable(&self) {
        match self.clock {
            PeripheralClockType::AHB1(ref v) => match v {
                HCLK1::DMA1 => {
                    self.rcc.disable_dma1_clock();
                }
                HCLK1::DMA2 => {
                    self.rcc.disable_dma2_clock();
                }
                HCLK1::GPIOH => {
                    self.rcc.disable_gpioh_clock();
                }
                HCLK1::GPIOG => {
                    self.rcc.disable_gpiog_clock();
                }
                HCLK1::GPIOF => {
                    self.rcc.disable_gpiof_clock();
                }
                HCLK1::GPIOE => {
                    self.rcc.disable_gpioe_clock();
                }
                HCLK1::GPIOD => {
                    self.rcc.disable_gpiod_clock();
                }
                HCLK1::GPIOC => {
                    self.rcc.disable_gpioc_clock();
                }
                HCLK1::GPIOB => {
                    self.rcc.disable_gpiob_clock();
                }
                HCLK1::GPIOA => {
                    self.rcc.disable_gpioa_clock();
                }
            },
            PeripheralClockType::AHB2(ref v) => match v {
                HCLK2::RNG => {
                    self.rcc.disable_rng_clock();
                }
                HCLK2::OTGFS => {
                    self.rcc.disable_otgfs_clock();
                }
            },
            PeripheralClockType::AHB3(ref v) => match v {
                HCLK3::FMC => self.rcc.disable_fmc_clock(),
            },
            PeripheralClockType::APB1(ref v) => match v {
                PCLK1::TIM2 => {
                    self.rcc.disable_tim2_clock();
                }
                PCLK1::USART2 => {
                    self.rcc.disable_usart2_clock();
                }
                PCLK1::USART3 => {
                    self.rcc.disable_usart3_clock();
                }
                PCLK1::I2C1 => {
                    self.rcc.disable_i2c1_clock();
                }
                PCLK1::SPI3 => {
                    self.rcc.disable_spi3_clock();
                }
                PCLK1::CAN1 => {
                    self.rcc.disable_can1_clock();
                }
            },
            PeripheralClockType::APB2(ref v) => match v {
                PCLK2::USART1 => {
                    self.rcc.disable_usart1_clock();
                }
                PCLK2::ADC1 => {
                    self.rcc.disable_adc1_clock();
                }
                PCLK2::SYSCFG => {
                    self.rcc.disable_syscfg_clock();
                }
            },
        }
    }
}

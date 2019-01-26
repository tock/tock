use kernel::common::registers::{register_bitfields, ReadWrite};
use kernel::common::StaticRef;
use kernel::ClockInterface;

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
        PLLQ3 OFFSET(27) NUMBITS(1) [],
        /// Main PLL (PLL) division factor for USB OTG FS, SDIO and random num
        PLLQ2 OFFSET(26) NUMBITS(1) [],
        /// Main PLL (PLL) division factor for USB OTG FS, SDIO and random num
        PLLQ1 OFFSET(25) NUMBITS(1) [],
        /// Main PLL (PLL) division factor for USB OTG FS, SDIO and random num
        PLLQ0 OFFSET(24) NUMBITS(1) [],
        /// Main PLL(PLL) and audio PLL (PLLI2S) entry clock source
        PLLSRC OFFSET(22) NUMBITS(1) [],
        /// Main PLL (PLL) division factor for main system clock
        PLLP1 OFFSET(17) NUMBITS(1) [],
        /// Main PLL (PLL) division factor for main system clock
        PLLP0 OFFSET(16) NUMBITS(1) [],
        /// Main PLL (PLL) multiplication factor for VCO
        PLLN8 OFFSET(14) NUMBITS(1) [],
        /// Main PLL (PLL) multiplication factor for VCO
        PLLN7 OFFSET(13) NUMBITS(1) [],
        /// Main PLL (PLL) multiplication factor for VCO
        PLLN6 OFFSET(12) NUMBITS(1) [],
        /// Main PLL (PLL) multiplication factor for VCO
        PLLN5 OFFSET(11) NUMBITS(1) [],
        /// Main PLL (PLL) multiplication factor for VCO
        PLLN4 OFFSET(10) NUMBITS(1) [],
        /// Main PLL (PLL) multiplication factor for VCO
        PLLN3 OFFSET(9) NUMBITS(1) [],
        /// Main PLL (PLL) multiplication factor for VCO
        PLLN2 OFFSET(8) NUMBITS(1) [],
        /// Main PLL (PLL) multiplication factor for VCO
        PLLN1 OFFSET(7) NUMBITS(1) [],
        /// Main PLL (PLL) multiplication factor for VCO
        PLLN0 OFFSET(6) NUMBITS(1) [],
        /// Division factor for the main PLL (PLL) and audio PLL (PLLI2S) inpu
        PLLM5 OFFSET(5) NUMBITS(1) [],
        /// Division factor for the main PLL (PLL) and audio PLL (PLLI2S) inpu
        PLLM4 OFFSET(4) NUMBITS(1) [],
        /// Division factor for the main PLL (PLL) and audio PLL (PLLI2S) inpu
        PLLM3 OFFSET(3) NUMBITS(1) [],
        /// Division factor for the main PLL (PLL) and audio PLL (PLLI2S) inpu
        PLLM2 OFFSET(2) NUMBITS(1) [],
        /// Division factor for the main PLL (PLL) and audio PLL (PLLI2S) inpu
        PLLM1 OFFSET(1) NUMBITS(1) [],
        /// Division factor for the main PLL (PLL) and audio PLL (PLLI2S) inpu
        PLLM0 OFFSET(0) NUMBITS(1) []
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
        self.registers.ahb1enr.is_set(AHB1ENR::DMA1EN)
    }

    fn enable_dma1_clock(&self) {
        self.registers.ahb1enr.modify(AHB1ENR::DMA1EN::SET)
    }

    fn disable_dma1_clock(&self) {
        self.registers.ahb1enr.modify(AHB1ENR::DMA1EN::CLEAR)
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
}

/// Clock sources for CPU
pub enum CPUClock {
    HSE,
    HSI,
    PLLCLK,
    PPLLR,
}

/// Bus + Clock name for the peripherals
///
/// Not yet implemented clocks:
///
/// AHB2(HCLK2)
/// AHB3(HCLK3)
pub enum PeripheralClock {
    AHB1(HCLK1),
    APB1(PCLK1),
    APB2(PCLK2),
}

/// Peripherals clocked by HCLK1
pub enum HCLK1 {
    DMA1,
    GPIOH,
    GPIOG,
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
}

/// Peripherals clocked by PCLK2
pub enum PCLK2 {
    SYSCFG,
}

impl ClockInterface for PeripheralClock {
    fn is_enabled(&self) -> bool {
        match self {
            &PeripheralClock::AHB1(ref v) => match v {
                HCLK1::DMA1 => unsafe { RCC.is_enabled_dma1_clock() },
                HCLK1::GPIOH => unsafe { RCC.is_enabled_gpioh_clock() },
                HCLK1::GPIOG => unsafe { RCC.is_enabled_gpiog_clock() },
                HCLK1::GPIOF => unsafe { RCC.is_enabled_gpiof_clock() },
                HCLK1::GPIOE => unsafe { RCC.is_enabled_gpioe_clock() },
                HCLK1::GPIOD => unsafe { RCC.is_enabled_gpiod_clock() },
                HCLK1::GPIOC => unsafe { RCC.is_enabled_gpioc_clock() },
                HCLK1::GPIOB => unsafe { RCC.is_enabled_gpiob_clock() },
                HCLK1::GPIOA => unsafe { RCC.is_enabled_gpioa_clock() },
            },
            &PeripheralClock::APB1(ref v) => match v {
                PCLK1::TIM2 => unsafe { RCC.is_enabled_tim2_clock() },
                PCLK1::USART2 => unsafe { RCC.is_enabled_usart2_clock() },
                PCLK1::USART3 => unsafe { RCC.is_enabled_usart3_clock() },
            },
            &PeripheralClock::APB2(ref v) => match v {
                PCLK2::SYSCFG => unsafe { RCC.is_enabled_syscfg_clock() },
            },
        }
    }

    fn enable(&self) {
        match self {
            &PeripheralClock::AHB1(ref v) => match v {
                HCLK1::DMA1 => unsafe {
                    RCC.enable_dma1_clock();
                },
                HCLK1::GPIOH => unsafe {
                    RCC.enable_gpioh_clock();
                },
                HCLK1::GPIOG => unsafe {
                    RCC.enable_gpiog_clock();
                },
                HCLK1::GPIOF => unsafe {
                    RCC.enable_gpiof_clock();
                },
                HCLK1::GPIOE => unsafe {
                    RCC.enable_gpioe_clock();
                },
                HCLK1::GPIOD => unsafe {
                    RCC.enable_gpiod_clock();
                },
                HCLK1::GPIOC => unsafe {
                    RCC.enable_gpioc_clock();
                },
                HCLK1::GPIOB => unsafe {
                    RCC.enable_gpiob_clock();
                },
                HCLK1::GPIOA => unsafe {
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
            },
            &PeripheralClock::APB2(ref v) => match v {
                PCLK2::SYSCFG => unsafe {
                    RCC.enable_syscfg_clock();
                },
            },
        }
    }

    fn disable(&self) {
        match self {
            &PeripheralClock::AHB1(ref v) => match v {
                HCLK1::DMA1 => unsafe {
                    RCC.disable_dma1_clock();
                },
                HCLK1::GPIOH => unsafe {
                    RCC.disable_gpioh_clock();
                },
                HCLK1::GPIOG => unsafe {
                    RCC.disable_gpiog_clock();
                },
                HCLK1::GPIOF => unsafe {
                    RCC.disable_gpiof_clock();
                },
                HCLK1::GPIOE => unsafe {
                    RCC.disable_gpioe_clock();
                },
                HCLK1::GPIOD => unsafe {
                    RCC.disable_gpiod_clock();
                },
                HCLK1::GPIOC => unsafe {
                    RCC.disable_gpioc_clock();
                },
                HCLK1::GPIOB => unsafe {
                    RCC.disable_gpiob_clock();
                },
                HCLK1::GPIOA => unsafe {
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
            },
            &PeripheralClock::APB2(ref v) => match v {
                PCLK2::SYSCFG => unsafe {
                    RCC.disable_syscfg_clock();
                },
            },
        }
    }
}

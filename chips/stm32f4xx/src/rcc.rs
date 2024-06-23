// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

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

pub enum RtcClockSource {
    LSI,
    LSE,
    HSERTC,
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
        match self.registers.cfgr.read(CFGR::SWS) {
            0b00 => SysClockSource::HSI,
            0b01 => SysClockSource::HSE,
            _ => SysClockSource::PLL,
            // Uncomment this when PPLLR support is added. Also change the above match arm to
            // 0b10 => SysClockSource::PLL,
            //_ => SysClockSource::PPLLR,
        }
    }

    // Set the system clock source
    // The source must be enabled
    // NOTE: The flash latency also needs to be configured when changing the system clock frequency
    pub(crate) fn set_sys_clock_source(&self, source: SysClockSource) {
        self.registers.cfgr.modify(CFGR::SW.val(source as u32));
    }

    pub(crate) fn is_hsi_clock_system_clock(&self) -> bool {
        let system_clock_source = self.get_sys_clock_source();
        system_clock_source == SysClockSource::HSI
            || system_clock_source == SysClockSource::PLL
                && self.registers.pllcfgr.read(PLLCFGR::PLLSRC) == PllSource::HSI as u32
    }

    pub(crate) fn is_hse_clock_system_clock(&self) -> bool {
        let system_clock_source = self.get_sys_clock_source();
        system_clock_source == SysClockSource::HSE
            || system_clock_source == SysClockSource::PLL
                && self.registers.pllcfgr.read(PLLCFGR::PLLSRC) == PllSource::HSE as u32
    }

    /* HSI clock */
    // The HSI clock must not be configured as the system clock, either directly or indirectly.
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

    /* HSE clock */
    pub(crate) fn disable_hse_clock(&self) {
        self.registers.cr.modify(CR::HSEON::CLEAR);
        self.registers.cr.modify(CR::HSEBYP::CLEAR);
    }

    pub(crate) fn enable_hse_clock_bypass(&self) {
        self.registers.cr.modify(CR::HSEBYP::SET);
    }

    pub(crate) fn enable_hse_clock(&self) {
        self.registers.cr.modify(CR::HSEON::SET);
    }

    pub(crate) fn is_enabled_hse_clock(&self) -> bool {
        self.registers.cr.is_set(CR::HSEON)
    }

    // Indicates whether the HSE oscillator is stable
    pub(crate) fn is_ready_hse_clock(&self) -> bool {
        self.registers.cr.is_set(CR::HSERDY)
    }

    /* Main PLL clock*/

    // The main PLL clock must not be configured as the system clock.
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

    pub(crate) fn get_pll_clocks_source(&self) -> PllSource {
        match self.registers.pllcfgr.read(PLLCFGR::PLLSRC) {
            0b0 => PllSource::HSI,
            _ => PllSource::HSE,
        }
    }

    // This method must be called only when all PLL clocks are disabled
    pub(crate) fn set_pll_clocks_source(&self, source: PllSource) {
        self.registers
            .pllcfgr
            .modify(PLLCFGR::PLLSRC.val(source as u32));
    }

    pub(crate) fn get_pll_clocks_m_divider(&self) -> PLLM {
        match self.registers.pllcfgr.read(PLLCFGR::PLLM) {
            8 => PLLM::DivideBy8,
            16 => PLLM::DivideBy16,
            _ => panic!("Unexpected PLLM divider"),
        }
    }

    // This method must be called only when all PLL clocks are disabled
    pub(crate) fn set_pll_clocks_m_divider(&self, m: PLLM) {
        self.registers.pllcfgr.modify(PLLCFGR::PLLM.val(m as u32));
    }

    pub(crate) fn get_pll_clock_n_multiplier(&self) -> usize {
        self.registers.pllcfgr.read(PLLCFGR::PLLN) as usize
    }

    // This method must be called only if the main PLL clock is disabled
    pub(crate) fn set_pll_clock_n_multiplier(&self, n: usize) {
        self.registers.pllcfgr.modify(PLLCFGR::PLLN.val(n as u32));
    }

    pub(crate) fn get_pll_clock_p_divider(&self) -> PLLP {
        match self.registers.pllcfgr.read(PLLCFGR::PLLP) {
            0b00 => PLLP::DivideBy2,
            0b01 => PLLP::DivideBy4,
            0b10 => PLLP::DivideBy6,
            _ => PLLP::DivideBy8,
        }
    }

    // This method must be called only if the main PLL clock is disabled
    pub(crate) fn set_pll_clock_p_divider(&self, p: PLLP) {
        self.registers.pllcfgr.modify(PLLCFGR::PLLP.val(p as u32));
    }

    pub(crate) fn _get_pll_clock_q_divider(&self) -> PLLQ {
        match self.registers.pllcfgr.read(PLLCFGR::PLLQ) {
            3 => PLLQ::DivideBy3,
            4 => PLLQ::DivideBy4,
            5 => PLLQ::DivideBy5,
            6 => PLLQ::DivideBy6,
            7 => PLLQ::DivideBy7,
            8 => PLLQ::DivideBy8,
            9 => PLLQ::DivideBy9,
            _ => panic!("Unexpected PLLQ divider"),
        }
    }

    // This method must be called only if the main PLL clock is disabled
    pub(crate) fn set_pll_clock_q_divider(&self, q: PLLQ) {
        self.registers.pllcfgr.modify(PLLCFGR::PLLQ.val(q as u32));
    }

    /* AHB prescaler */

    pub(crate) fn set_ahb_prescaler(&self, ahb_prescaler: AHBPrescaler) {
        self.registers
            .cfgr
            .modify(CFGR::HPRE.val(ahb_prescaler as u32));
    }

    pub(crate) fn get_ahb_prescaler(&self) -> AHBPrescaler {
        match self.registers.cfgr.read(CFGR::HPRE) {
            0b1000 => AHBPrescaler::DivideBy2,
            0b1001 => AHBPrescaler::DivideBy4,
            0b1010 => AHBPrescaler::DivideBy8,
            0b1011 => AHBPrescaler::DivideBy16,
            0b1100 => AHBPrescaler::DivideBy64,
            0b1101 => AHBPrescaler::DivideBy128,
            0b1110 => AHBPrescaler::DivideBy256,
            0b1111 => AHBPrescaler::DivideBy512,
            _ => AHBPrescaler::DivideBy1,
        }
    }

    /* APB1 prescaler */

    pub(crate) fn set_apb1_prescaler(&self, apb1_prescaler: APBPrescaler) {
        self.registers
            .cfgr
            .modify(CFGR::PPRE1.val(apb1_prescaler as u32));
    }

    pub(crate) fn get_apb1_prescaler(&self) -> APBPrescaler {
        match self.registers.cfgr.read(CFGR::PPRE1) {
            0b100 => APBPrescaler::DivideBy2,
            0b101 => APBPrescaler::DivideBy4,
            0b110 => APBPrescaler::DivideBy8,
            0b111 => APBPrescaler::DivideBy16,
            _ => APBPrescaler::DivideBy1, // 0b0xx means no division
        }
    }

    /* APB2 prescaler */

    pub(crate) fn set_apb2_prescaler(&self, apb2_prescaler: APBPrescaler) {
        self.registers
            .cfgr
            .modify(CFGR::PPRE2.val(apb2_prescaler as u32));
    }

    pub(crate) fn get_apb2_prescaler(&self) -> APBPrescaler {
        match self.registers.cfgr.read(CFGR::PPRE2) {
            0b100 => APBPrescaler::DivideBy2,
            0b101 => APBPrescaler::DivideBy4,
            0b110 => APBPrescaler::DivideBy8,
            0b111 => APBPrescaler::DivideBy16,
            _ => APBPrescaler::DivideBy1, // 0b0xx means no division
        }
    }

    pub(crate) fn set_mco1_clock_source(&self, source: MCO1Source) {
        self.registers.cfgr.modify(CFGR::MCO1.val(source as u32));
    }

    pub(crate) fn get_mco1_clock_source(&self) -> MCO1Source {
        match self.registers.cfgr.read(CFGR::MCO1) {
            0b00 => MCO1Source::HSI,
            // When LSE or HSE are added, uncomment the following lines
            //0b01 => MCO1Source::LSE,
            0b10 => MCO1Source::HSE,
            // 0b11 corresponds to MCO1Source::PLL
            _ => MCO1Source::PLL,
        }
    }

    pub(crate) fn set_mco1_clock_divider(&self, divider: MCO1Divider) {
        self.registers
            .cfgr
            .modify(CFGR::MCO1PRE.val(divider as u32));
    }

    pub(crate) fn get_mco1_clock_divider(&self) -> MCO1Divider {
        match self.registers.cfgr.read(CFGR::MCO1PRE) {
            0b100 => MCO1Divider::DivideBy2,
            0b101 => MCO1Divider::DivideBy3,
            0b110 => MCO1Divider::DivideBy4,
            0b111 => MCO1Divider::DivideBy5,
            _ => MCO1Divider::DivideBy1,
        }
    }

    pub(crate) fn configure_rng_clock(&self) {
        self.registers.pllcfgr.modify(PLLCFGR::PLLQ.val(2));
        self.registers.cr.modify(CR::PLLON::SET);
    }

    // I2C1 clock

    pub(crate) fn is_enabled_i2c1_clock(&self) -> bool {
        self.registers.apb1enr.is_set(APB1ENR::I2C1EN)
    }

    pub(crate) fn enable_i2c1_clock(&self) {
        self.registers.apb1enr.modify(APB1ENR::I2C1EN::SET);
        self.registers.apb1rstr.modify(APB1RSTR::I2C1RST::SET);
        self.registers.apb1rstr.modify(APB1RSTR::I2C1RST::CLEAR);
    }

    pub(crate) fn disable_i2c1_clock(&self) {
        self.registers.apb1enr.modify(APB1ENR::I2C1EN::CLEAR)
    }

    // SPI3 clock

    pub(crate) fn is_enabled_spi3_clock(&self) -> bool {
        self.registers.apb1enr.is_set(APB1ENR::SPI3EN)
    }

    pub(crate) fn enable_spi3_clock(&self) {
        self.registers.apb1enr.modify(APB1ENR::SPI3EN::SET)
    }

    pub(crate) fn disable_spi3_clock(&self) {
        self.registers.apb1enr.modify(APB1ENR::SPI3EN::CLEAR)
    }

    // TIM2 clock
    pub(crate) fn is_enabled_tim_pre(&self) -> bool {
        self.registers.dckcfgr.is_set(DCKCFGR::TIMPRE)
    }

    pub(crate) fn is_enabled_tim2_clock(&self) -> bool {
        self.registers.apb1enr.is_set(APB1ENR::TIM2EN)
    }

    pub(crate) fn enable_tim2_clock(&self) {
        self.registers.apb1enr.modify(APB1ENR::TIM2EN::SET)
    }

    pub(crate) fn disable_tim2_clock(&self) {
        self.registers.apb1enr.modify(APB1ENR::TIM2EN::CLEAR)
    }

    // SYSCFG clock

    pub(crate) fn is_enabled_syscfg_clock(&self) -> bool {
        self.registers.apb2enr.is_set(APB2ENR::SYSCFGEN)
    }

    pub(crate) fn enable_syscfg_clock(&self) {
        self.registers.apb2enr.modify(APB2ENR::SYSCFGEN::SET)
    }

    pub(crate) fn disable_syscfg_clock(&self) {
        self.registers.apb2enr.modify(APB2ENR::SYSCFGEN::CLEAR)
    }

    // DMA1 clock

    pub(crate) fn is_enabled_dma1_clock(&self) -> bool {
        self.registers.ahb1enr.is_set(AHB1ENR::DMA1EN)
    }

    pub(crate) fn enable_dma1_clock(&self) {
        self.registers.ahb1enr.modify(AHB1ENR::DMA1EN::SET)
    }

    pub(crate) fn disable_dma1_clock(&self) {
        self.registers.ahb1enr.modify(AHB1ENR::DMA1EN::CLEAR)
    }

    // DMA2 clock
    pub(crate) fn is_enabled_dma2_clock(&self) -> bool {
        self.registers.ahb1enr.is_set(AHB1ENR::DMA2EN)
    }

    pub(crate) fn enable_dma2_clock(&self) {
        self.registers.ahb1enr.modify(AHB1ENR::DMA2EN::SET)
    }

    pub(crate) fn disable_dma2_clock(&self) {
        self.registers.ahb1enr.modify(AHB1ENR::DMA2EN::CLEAR)
    }

    // GPIOH clock

    pub(crate) fn is_enabled_gpioh_clock(&self) -> bool {
        self.registers.ahb1enr.is_set(AHB1ENR::GPIOHEN)
    }

    pub(crate) fn enable_gpioh_clock(&self) {
        self.registers.ahb1enr.modify(AHB1ENR::GPIOHEN::SET)
    }

    pub(crate) fn disable_gpioh_clock(&self) {
        self.registers.ahb1enr.modify(AHB1ENR::GPIOHEN::CLEAR)
    }

    // GPIOG clock

    pub(crate) fn is_enabled_gpiog_clock(&self) -> bool {
        self.registers.ahb1enr.is_set(AHB1ENR::GPIOGEN)
    }

    pub(crate) fn enable_gpiog_clock(&self) {
        self.registers.ahb1enr.modify(AHB1ENR::GPIOGEN::SET)
    }

    pub(crate) fn disable_gpiog_clock(&self) {
        self.registers.ahb1enr.modify(AHB1ENR::GPIOGEN::CLEAR)
    }

    // GPIOF clock

    pub(crate) fn is_enabled_gpiof_clock(&self) -> bool {
        self.registers.ahb1enr.is_set(AHB1ENR::GPIOFEN)
    }

    pub(crate) fn enable_gpiof_clock(&self) {
        self.registers.ahb1enr.modify(AHB1ENR::GPIOFEN::SET)
    }

    pub(crate) fn disable_gpiof_clock(&self) {
        self.registers.ahb1enr.modify(AHB1ENR::GPIOFEN::CLEAR)
    }

    // GPIOE clock

    pub(crate) fn is_enabled_gpioe_clock(&self) -> bool {
        self.registers.ahb1enr.is_set(AHB1ENR::GPIOEEN)
    }

    pub(crate) fn enable_gpioe_clock(&self) {
        self.registers.ahb1enr.modify(AHB1ENR::GPIOEEN::SET)
    }

    pub(crate) fn disable_gpioe_clock(&self) {
        self.registers.ahb1enr.modify(AHB1ENR::GPIOEEN::CLEAR)
    }

    // GPIOD clock

    pub(crate) fn is_enabled_gpiod_clock(&self) -> bool {
        self.registers.ahb1enr.is_set(AHB1ENR::GPIODEN)
    }

    pub(crate) fn enable_gpiod_clock(&self) {
        self.registers.ahb1enr.modify(AHB1ENR::GPIODEN::SET)
    }

    pub(crate) fn disable_gpiod_clock(&self) {
        self.registers.ahb1enr.modify(AHB1ENR::GPIODEN::CLEAR)
    }

    // GPIOC clock

    pub(crate) fn is_enabled_gpioc_clock(&self) -> bool {
        self.registers.ahb1enr.is_set(AHB1ENR::GPIOCEN)
    }

    pub(crate) fn enable_gpioc_clock(&self) {
        self.registers.ahb1enr.modify(AHB1ENR::GPIOCEN::SET)
    }

    pub(crate) fn disable_gpioc_clock(&self) {
        self.registers.ahb1enr.modify(AHB1ENR::GPIOCEN::CLEAR)
    }

    // GPIOB clock

    pub(crate) fn is_enabled_gpiob_clock(&self) -> bool {
        self.registers.ahb1enr.is_set(AHB1ENR::GPIOBEN)
    }

    pub(crate) fn enable_gpiob_clock(&self) {
        self.registers.ahb1enr.modify(AHB1ENR::GPIOBEN::SET)
    }

    pub(crate) fn disable_gpiob_clock(&self) {
        self.registers.ahb1enr.modify(AHB1ENR::GPIOBEN::CLEAR)
    }

    // GPIOA clock

    pub(crate) fn is_enabled_gpioa_clock(&self) -> bool {
        self.registers.ahb1enr.is_set(AHB1ENR::GPIOAEN)
    }

    pub(crate) fn enable_gpioa_clock(&self) {
        self.registers.ahb1enr.modify(AHB1ENR::GPIOAEN::SET)
    }

    pub(crate) fn disable_gpioa_clock(&self) {
        self.registers.ahb1enr.modify(AHB1ENR::GPIOAEN::CLEAR)
    }

    // FMC

    pub(crate) fn is_enabled_fmc_clock(&self) -> bool {
        self.registers.ahb3enr.is_set(AHB3ENR::FMCEN)
    }

    pub(crate) fn enable_fmc_clock(&self) {
        self.registers.ahb3enr.modify(AHB3ENR::FMCEN::SET)
    }

    pub(crate) fn disable_fmc_clock(&self) {
        self.registers.ahb3enr.modify(AHB3ENR::FMCEN::CLEAR)
    }

    // USART1 clock
    pub(crate) fn is_enabled_usart1_clock(&self) -> bool {
        self.registers.apb2enr.is_set(APB2ENR::USART1EN)
    }

    pub(crate) fn enable_usart1_clock(&self) {
        self.registers.apb2enr.modify(APB2ENR::USART1EN::SET)
    }

    pub(crate) fn disable_usart1_clock(&self) {
        self.registers.apb2enr.modify(APB2ENR::USART1EN::CLEAR)
    }

    // USART2 clock

    pub(crate) fn is_enabled_usart2_clock(&self) -> bool {
        self.registers.apb1enr.is_set(APB1ENR::USART2EN)
    }

    pub(crate) fn enable_usart2_clock(&self) {
        self.registers.apb1enr.modify(APB1ENR::USART2EN::SET)
    }

    pub(crate) fn disable_usart2_clock(&self) {
        self.registers.apb1enr.modify(APB1ENR::USART2EN::CLEAR)
    }

    // USART3 clock

    pub(crate) fn is_enabled_usart3_clock(&self) -> bool {
        self.registers.apb1enr.is_set(APB1ENR::USART3EN)
    }

    pub(crate) fn enable_usart3_clock(&self) {
        self.registers.apb1enr.modify(APB1ENR::USART3EN::SET)
    }

    pub(crate) fn disable_usart3_clock(&self) {
        self.registers.apb1enr.modify(APB1ENR::USART3EN::CLEAR)
    }

    // ADC1 clock

    pub(crate) fn is_enabled_adc1_clock(&self) -> bool {
        self.registers.apb2enr.is_set(APB2ENR::ADC1EN)
    }

    pub(crate) fn enable_adc1_clock(&self) {
        self.registers.apb2enr.modify(APB2ENR::ADC1EN::SET)
    }

    pub(crate) fn disable_adc1_clock(&self) {
        self.registers.apb2enr.modify(APB2ENR::ADC1EN::CLEAR)
    }

    // DAC clock

    pub(crate) fn is_enabled_dac_clock(&self) -> bool {
        self.registers.apb1enr.is_set(APB1ENR::DACEN)
    }

    pub(crate) fn enable_dac_clock(&self) {
        self.registers.apb1enr.modify(APB1ENR::DACEN::SET)
    }

    pub(crate) fn disable_dac_clock(&self) {
        self.registers.apb1enr.modify(APB1ENR::DACEN::CLEAR)
    }

    // RNG clock

    pub(crate) fn is_enabled_rng_clock(&self) -> bool {
        self.registers.ahb2enr.is_set(AHB2ENR::RNGEN)
    }

    pub(crate) fn enable_rng_clock(&self) {
        self.registers.ahb2enr.modify(AHB2ENR::RNGEN::SET);
    }

    pub(crate) fn disable_rng_clock(&self) {
        self.registers.ahb2enr.modify(AHB2ENR::RNGEN::CLEAR);
    }

    // OTGFS clock

    pub(crate) fn is_enabled_otgfs_clock(&self) -> bool {
        self.registers.ahb2enr.is_set(AHB2ENR::OTGFSEN)
    }

    pub(crate) fn enable_otgfs_clock(&self) {
        self.registers.ahb2enr.modify(AHB2ENR::OTGFSEN::SET);
    }

    pub(crate) fn disable_otgfs_clock(&self) {
        self.registers.ahb2enr.modify(AHB2ENR::OTGFSEN::CLEAR);
    }

    // CAN1 clock

    pub(crate) fn is_enabled_can1_clock(&self) -> bool {
        self.registers.apb1enr.is_set(APB1ENR::CAN1EN)
    }

    pub(crate) fn enable_can1_clock(&self) {
        self.registers.apb1rstr.modify(APB1RSTR::CAN1RST::SET);
        self.registers.apb1rstr.modify(APB1RSTR::CAN1RST::CLEAR);
        self.registers.apb1enr.modify(APB1ENR::CAN1EN::SET);
    }

    pub(crate) fn disable_can1_clock(&self) {
        self.registers.apb1enr.modify(APB1ENR::CAN1EN::CLEAR);
    }

    // RTC clock
    pub(crate) fn source_into_u32(source: RtcClockSource) -> u32 {
        match source {
            RtcClockSource::LSE => 1,
            RtcClockSource::LSI => 2,
            RtcClockSource::HSERTC => 3,
        }
    }

    pub(crate) fn enable_lsi_clock(&self) {
        self.registers.csr.modify(CSR::LSION::SET);
    }

    pub(crate) fn is_enabled_pwr_clock(&self) -> bool {
        self.registers.apb1enr.is_set(APB1ENR::PWREN)
    }

    pub(crate) fn enable_pwr_clock(&self) {
        // Enable the power interface clock
        self.registers.apb1enr.modify(APB1ENR::PWREN::SET);
    }

    pub(crate) fn disable_pwr_clock(&self) {
        self.registers.apb1enr.modify(APB1ENR::PWREN::CLEAR);
    }

    pub(crate) fn is_enabled_rtc_clock(&self) -> bool {
        self.registers.bdcr.is_set(BDCR::RTCEN)
    }

    pub(crate) fn enable_rtc_clock(&self, source: RtcClockSource) {
        // Enable LSI
        self.enable_lsi_clock();
        let mut counter = 1_000;
        while counter > 0 && !self.registers.csr.is_set(CSR::LSION) {
            counter -= 1;
        }
        if counter == 0 {
            panic!("Unable to activate lsi clock");
        }

        // Select RTC clock source
        let source_num = Rcc::source_into_u32(source);
        self.registers.bdcr.modify(BDCR::RTCSEL.val(source_num));

        // Enable RTC clock
        self.registers.bdcr.modify(BDCR::RTCEN::SET);
    }

    pub(crate) fn disable_rtc_clock(&self) {
        self.registers.bdcr.modify(BDCR::RTCEN.val(1));
        self.registers.bdcr.modify(BDCR::RTCSEL.val(0));
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum PLLP {
    DivideBy2 = 0b00,
    DivideBy4 = 0b01,
    DivideBy6 = 0b10,
    DivideBy8 = 0b11,
}

impl From<PLLP> for usize {
    // (variant_value + 1) * 2 = X for X in DivideByX
    fn from(item: PLLP) -> Self {
        (item as usize + 1) << 1
    }
}

// Theoretically, the PLLM value can range from 2 to 63. However, the current implementation was
// designed to support 1MHz frequency precision. In a future update, PLLM will become a usize.
#[allow(dead_code)]
pub(crate) enum PLLM {
    DivideBy8 = 8,
    DivideBy16 = 16,
}

#[derive(Copy, Clone, Debug, PartialEq)]
// Due to the restricted values for PLLM, PLLQ 2/10-15 values are meaningless.
pub(crate) enum PLLQ {
    DivideBy3 = 3,
    DivideBy4,
    DivideBy5,
    DivideBy6,
    DivideBy7,
    DivideBy8,
    DivideBy9,
}

/// Clock sources for the CPU
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SysClockSource {
    HSI = 0b00,
    HSE = 0b01,
    PLL = 0b10,
    // NOTE: not all STM32F4xx boards support this source.
    //PPLLR = 0b11, Uncomment this when support for PPLLR is added
}

pub enum PllSource {
    HSI = 0b0,
    HSE = 0b1,
}

pub enum MCO1Source {
    HSI = 0b00,
    //LSE = 0b01, // When support for LSE is added, uncomment this
    HSE = 0b10,
    PLL = 0b11,
}

pub enum MCO1Divider {
    DivideBy1 = 0b000,
    DivideBy2 = 0b100,
    DivideBy3 = 0b101,
    DivideBy4 = 0b110,
    DivideBy5 = 0b111,
}

/// HSE Mode
#[derive(PartialEq)]
pub enum HseMode {
    BYPASS,
    CRYSTAL,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum AHBPrescaler {
    DivideBy1 = 0b0000,
    DivideBy2 = 0b1000,
    DivideBy4 = 0b1001,
    DivideBy8 = 0b1010,
    DivideBy16 = 0b1011,
    DivideBy64 = 0b1100,
    DivideBy128 = 0b1101,
    DivideBy256 = 0b1110,
    DivideBy512 = 0b1111,
}

impl From<AHBPrescaler> for usize {
    fn from(item: AHBPrescaler) -> usize {
        match item {
            AHBPrescaler::DivideBy1 => 1,
            AHBPrescaler::DivideBy2 => 2,
            AHBPrescaler::DivideBy4 => 4,
            AHBPrescaler::DivideBy8 => 8,
            AHBPrescaler::DivideBy16 => 16,
            AHBPrescaler::DivideBy64 => 64,
            AHBPrescaler::DivideBy128 => 128,
            AHBPrescaler::DivideBy256 => 256,
            AHBPrescaler::DivideBy512 => 512,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum APBPrescaler {
    DivideBy1 = 0b000, // No division
    DivideBy2 = 0b100,
    DivideBy4 = 0b101,
    DivideBy8 = 0b110,
    DivideBy16 = 0b111,
}

impl From<APBPrescaler> for usize {
    fn from(item: APBPrescaler) -> Self {
        match item {
            APBPrescaler::DivideBy1 => 1,
            APBPrescaler::DivideBy2 => 2,
            APBPrescaler::DivideBy4 => 4,
            APBPrescaler::DivideBy8 => 8,
            APBPrescaler::DivideBy16 => 16,
        }
    }
}

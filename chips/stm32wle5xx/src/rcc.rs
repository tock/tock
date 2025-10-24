// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

use kernel::utilities::registers::interfaces::{ReadWriteable, Readable};
use kernel::utilities::registers::{register_bitfields, ReadWrite};
use kernel::utilities::StaticRef;

/// Reset and clock control
#[repr(C)]
struct RccRegisters {
    /// clock control register
    cr: ReadWrite<u32, CR::Register>,
    icsr: ReadWrite<u32, ICSCR::Register>,
    /// clock configuration register
    cfgr: ReadWrite<u32, CFGR::Register>,
    /// PLL configuration register
    pllcfgr: ReadWrite<u32, PLLCFGR::Register>,
    _reserved0: [u8; 8],
    cier: ReadWrite<u32, CIER::Register>,
    cifgr: ReadWrite<u32, CIFR::Register>,
    cicr: ReadWrite<u32, CICR::Register>,
    _reserved1: [u8; 4],
    /// AHB1 peripheral reset register
    ahb1rstr: ReadWrite<u32, AHB1RSTR::Register>,
    /// AHB2 peripheral reset register
    ahb2rstr: ReadWrite<u32, AHB2RSTR::Register>,
    /// AHB3 peripheral reset register
    ahb3rstr: ReadWrite<u32, AHB3RSTR::Register>,
    _reserved2: [u8; 4],
    /// APB1 peripheral reset register 1
    apb1rstr1: ReadWrite<u32, APB1RSTR1::Register>,
    apb1rstr2: ReadWrite<u32, APB1RSTR2::Register>,
    /// APB2 peripheral reset register
    apb2rstr: ReadWrite<u32, APB2RSTR::Register>,
    apb3rstr: ReadWrite<u32, APB3RSTR::Register>,
    /// AHB1 peripheral clock register
    ahb1enr: ReadWrite<u32, AHB1ENR::Register>,
    /// AHB2 peripheral clock enable register
    ahb2enr: ReadWrite<u32, AHB2ENR::Register>,
    /// AHB3 peripheral clock enable register
    ahb3enr: ReadWrite<u32, AHB3ENR::Register>,
    _reserved3: [u8; 4],
    /// APB1 peripheral clock enable register 1
    apb1enr1: ReadWrite<u32, APB1ENR1::Register>,
    apb1enr2: ReadWrite<u32, APB1ENR2::Register>,
    /// APB2 peripheral clock enable register
    apb2enr: ReadWrite<u32, APB2ENR::Register>,
    apb3enr: ReadWrite<u32, APB3ENR::Register>,
    ahb1smenr: ReadWrite<u32, AHB1SMENR::Register>,
    ahb2smenr: ReadWrite<u32, AHB2SMENR::Register>,
    ahb3smenr: ReadWrite<u32, AHB3SMENR::Register>,
    _reserved4: [u8; 4],
    apb1smenr1: ReadWrite<u32, APB1SMENR1::Register>,
    apb1smenr2: ReadWrite<u32, APB1SMENR2::Register>,
    apb2smenr: ReadWrite<u32, APB2SMENR::Register>,
    apb3smenr: ReadWrite<u32, APB3SMENR::Register>,
    ccipr: ReadWrite<u32, CCIPR::Register>,
    _reserved5: [u8; 4],
    bdcr: ReadWrite<u32, BDCR::Register>,
    csr: ReadWrite<u32, CSR::Register>,
    _reserved6: [u8; 14], // TODO: confirm that this offset is correct
    extcfgr: ReadWrite<u32, EXTCFGR::Register>,
}

register_bitfields![u32,
    CR [
        /// Main PLL clock ready flag
        PLLRDY OFFSET(25) NUMBITS(1) [],
        /// Main PLL enable
        PLLON OFFSET(24) NUMBITS(1) [],
        /// HSE32 VDDTCXO output on package
        /// pin PB0-VDDTCXO enable
        HSEBYPPWR OFFSET(21) NUMBITS(1) [
            PBOSelected = 0,
            VDDTCXOSelected = 1
        ],
        /// HSE32 SYSCLK prescaler
        HSEPRE OFFSET(20) NUMBITS(1) [
            DivideBy1 = 0,
            DivideBy2 = 1
        ],
        /// HSE32 clock security system enable
        CSSON OFFSET(19) NUMBITS(1) [],
        /// HSE32 clock ready flag (set and cleared by hardware)
        HSERDY OFFSET(17) NUMBITS(1) [],
        /// HSE32 clock enable for CPU
        HSEON OFFSET(16) NUMBITS(1) [],
        /// HSI16 kernel clock ready flag for peripheral requests
        HSIKERDY OFFSET(12) NUMBITS(1) [],
        /// HSI16 automatic start from stop mode
        HSIASFS OFFSET(11) NUMBITS(1) [],
        /// HSI16 clock ready flag (set and cleared by hardware)
        HSIRDY OFFSET(10) NUMBITS(1) [],
        /// HSI16 enable for peripheral kernel clocks
        HSIKERON OFFSET(9) NUMBITS(1) [],
        /// HSI16 clock enable
        HSION OFFSET(8) NUMBITS(1) [],
        /// MSI clock ranges
        MSIRANGE OFFSET(4) NUMBITS(4) [
            Range0 = 0b0000,
            Range1 = 0b0001,
            Range2 = 0b0010,
            Range3 = 0b0011,
            Range4 = 0b0100,
            Range5 = 0b0101,
            Range6 = 0b0110,
            Range7 = 0b0111,
            Range8 = 0b1000,
            Range9 = 0b1001,
            Range10 = 0b1010,
            Range11 = 0b1011,
        ],
        /// MSI range control selection
        MSIRGSEL OFFSET(3) NUMBITS(1) [],
        /// MSI clock PLL enable
        MSIPLLEN OFFSET(2) NUMBITS(1) [],
        /// MSI clock ready flag
        MSIRDY OFFSET(1) NUMBITS(1) [],
        /// MSI clock enable
        MSION OFFSET(0) NUMBITS(1) [],
    ],
    ICSCR [
        /// HSI16 clock trimming
        HSITRIM OFFSET(24) NUMBITS(7) [],
        /// HSI16 clock calibration
        HSICAL OFFSET(16) NUMBITS(8) [],
        /// MSI clock trimming
        MSITRIM OFFSET(8) NUMBITS(8) [],
        /// MSI clock calibration
        MSICAL OFFSET(0) NUMBITS (8) [],
    ],
    CFGR [
        /// Microcontroller clock output prescaler
        MCOPRE OFFSET(28) NUMBITS(3) [
            DivideBy1 = 0b000,
            DivideBy2 = 0b001,
            DivideBy4 = 0b010,
            DivideBy8 = 0b011,
            DivideBy16 = 0b100,
        ],
        /// Microcontroller clock output selection
        MCOSEL OFFSET(24) NUMBITS(4) [
            NoClock = 0b0000,
            SYSCLK = 0b0001,
            MSI = 0b0010,
            HSI16 = 0b0011,
            HSE32 = 0b0100,
            PLLR = 0b0101,
            LSI = 0b0110,
            LSE = 0b1000,
            PLLP = 0b1101,
            PLLQ = 0b1110
        ],
        /// PCLK2 prescaler flag (APB2)
        PPRE2F OFFSET(18) NUMBITS(1) [],
        /// PCLK1 prescaler flag (APB1)
        PPRE1F OFFSET(17) NUMBITS(1) [],
        /// HCLK1 prescaler flag (CPU, AHB1, and AHB2)
        HPREF OFFSET(16) NUMBITS(1) [],
        /// Wake-up from Stop and CSS backup clock selection
        STOPWUICK OFFSET(15) NUMBITS(1) [],
        /// PCLK2 high-speed prescaler (APB2)
        PPRE2 OFFSET(13) NUMBITS(3) [],
        /// PCLK1 low-speed prescaler (APB1)
        PPRE1 OFFSET(10) NUMBITS(3) [],
        /// HCLK1 prescaler (CPU, AHB1, and AHB2)
        HPRE OFFSET(4) NUMBITS(4) [],
        /// System clock switch status
        SWS OFFSET(2) NUMBITS(2) [
            MSI = 0b00,
            HSI16 = 0b01,
            HSE32 = 0b10,
            PLL = 0b11,
        ],
        /// System clock switch
        SW OFFSET(0) NUMBITS(2) [
            MSI = 0b00,
            HSI16 = 0b01,
            HSE32 = 0b10,
            PLL = 0b11,
        ]
    ],
    PLLCFGR [
        /// Main PLL division factor for PLLRCLK
        PLLR OFFSET(29) NUMBITS(3) [
            DivideBy2 = 0b001,
            DivideBy3 = 0b010,
            DivideBy4 = 0b011,
            DivideBy5 = 0b100,
            DivideBy6 = 0b101,
            DivideBy7 = 0b110,
            DivideBy8 = 0b111,
        ],
        /// Main PLL PLLRCLK output enable
        PLLREN OFFSET(28) NUMBITS(1) [],
        /// Main PLL division factor for PLLQCLK
        PLLQ OFFSET(25) NUMBITS(3) [
            DivideBy2 = 0b001,
            DivideBy3 = 0b010,
            DivideBy4 = 0b011,
            DivideBy5 = 0b100,
            DivideBy6 = 0b101,
            DivideBy7 = 0b110,
            DivideBy8 = 0b111,
        ],
        /// Main PLL PLLQCLK output enable
        PLLQEN OFFSET(24) NUMBITS(1) [],
        /// Main PLL division factor for PLLPCLK
        PLLP OFFSET(17) NUMBITS(5) [
            DivideBy2 = 0b00001,
            DivideBy3 = 0b00010,
            DivideBy4 = 0b00011,
            DivideBy5 = 0b00100,
            DivideBy6 = 0b00101,
            DivideBy7 = 0b00110,
            DivideBy8 = 0b00111,
            DivideBy9 = 0b01000,
            DivideBy10 = 0b01001,
            DivideBy11 = 0b01010,
            DivideBy12 = 0b01011,
            DivideBy13 = 0b01100,
            DivideBy14 = 0b01101,
            DivideBy15 = 0b01110,
            DivideBy16 = 0b01111,
            DivideBy17 = 0b10000,
            DivideBy18 = 0b10001,
            DivideBy19 = 0b10010,
            DivideBy20 = 0b10011,
            DivideBy21 = 0b10100,
            DivideBy22 = 0b10101,
            DivideBy23 = 0b10110,
            DivideBy24 = 0b10111,
            DivideBy25 = 0b11000,
            DivideBy26 = 0b11001,
            DivideBy27 = 0b11010,
            DivideBy28 = 0b11011,
            DivideBy29 = 0b11100,
            DivideBy30 = 0b11101,
            DivideBy31 = 0b11110,
            DivideBy32 = 0b11111,
        ],
        /// Main PLL PLLPCLK output enable
        PLLPEN OFFSET(16) NUMBITS(1) [],
        /// Main PLL multiplication factor for VCO.
        /// Valid values range from 6-127.
        PLLN OFFSET(8) NUMBITS(7) [],
        /// Main PLL division factor for main PLL
        /// input clock.
        PLLM OFFSET(4) NUMBITS(3) [
            DivideBy1 = 0b000,
            DivideBy2 = 0b001,
            DivideBy3 = 0b010,
            DivideBy4 = 0b011,
            DivideBy5 = 0b100,
            DivideBy6 = 0b101,
            DivideBy7 = 0b110,
            DivideBy8 = 0b111,
        ],
        /// Main PLL entry clock source
        PLLSRC OFFSET(0) NUMBITS(2) [
            None = 0b00,
            MSI = 0b01,
            HSI16 = 0b10,
            HSE32 = 0b11,
        ]
    ],
    CIER [
        /// LSE clock security system interrupt enable
        LSECSSIE OFFSET(9) NUMBITS(1) [],
        /// PLL ready interrupt enable
        PLLRDYIE OFFSET(5) NUMBITS(1) [],
        /// HSE ready interrupt enable
        HSERDYIE OFFSET(4) NUMBITS(1) [],
        /// HSI ready interrupt enable
        HSIRDYIE OFFSET(3) NUMBITS(1) [],
        /// MSI ready interrupt enable
        MSIRDYIE OFFSET(2) NUMBITS(1) [],
        /// LSE ready interrupt enable
        LSERDYIE OFFSET(1) NUMBITS(1) [],
        /// LSI ready interrupt enable
        LSIRDYIE OFFSET(0) NUMBITS(1) []
    ],
    CIFR [
        /// LSE CSS flag
        LSECSSF OFFSET(9) NUMBITS(1) [],
        /// HSE32 CSS flag
        CSSF OFFSET(8) NUMBITS(1) [],
        /// PLL ready interrupt flag
        PLLRDYF OFFSET(5) NUMBITS(1) [],
        /// HSE ready interrupt flag
        HSERDYF OFFSET(4) NUMBITS(1) [],
        /// HSI ready interrupt flag
        HSIRDYF OFFSET(3) NUMBITS(1) [],
        /// MSI ready interrupt flag
        MSIRDYF OFFSET(2) NUMBITS(1) [],
        /// LSE ready interrupt flag
        LSERDYF OFFSET(1) NUMBITS(1) [],
        /// LSI ready interrupt flag
        LSIRDYF OFFSET(0) NUMBITS(1) []
    ],
    CICR [
        /// LSE CSS flag clear
        LSECSSC OFFSET(9) NUMBITS(1) [],
        /// HSE32 CSS flag clear
        CSSC OFFSET(8) NUMBITS(1) [],
        /// PLL ready interrupt clear
        PLLRDYC OFFSET(5) NUMBITS(1) [],
        /// HSE ready interrupt clear
        HSERDYC OFFSET(4) NUMBITS(1) [],
        /// HSI ready interrupt clear
        HSIRDYC OFFSET(3) NUMBITS(1) [],
        /// MSI ready interrupt clear
        MSIRDYC OFFSET(2) NUMBITS(1) [],
        /// LSE ready interrupt clear
        LSERDYC OFFSET(1) NUMBITS(1) [],
        /// LSI ready interrupt clear
        LSIRDYC OFFSET(0) NUMBITS(1) []
    ],
    AHB1RSTR [
        /// CRC reset
        CRCRST OFFSET(12) NUMBITS(1) [],
        /// DMAMUX1 reset
        DMAMUX1RST OFFSET(2) NUMBITS(1) [],
        /// DMA2 reset
        DMA2RST OFFSET(1) NUMBITS(1) [],
        /// DMA1 reset
        DMA1RST OFFSET(0) NUMBITS(1) [],
    ],
    AHB2RSTR [
        /// IO port H reset
        GPIOHRST OFFSET(7) NUMBITS(1) [],
        /// IO port C reset
        GPIOCRST OFFSET(2) NUMBITS(1) [],
        /// IO port B reset
        GPIOBRST OFFSET(1) NUMBITS(1) [],
        /// IO port A reset
        GPIOARST OFFSET(0) NUMBITS(1) [],
    ],
    AHB3RSTR [
        /// Flash interface reset
        FLASHRST OFFSET(25) NUMBITS(1) [],
        /// HSEM reset
        HSEMRST OFFSET(19) NUMBITS(1) [],
        /// True RNG reset
        RNGRST OFFSET(18) NUMBITS(1) [],
        /// AES hardware accelerator reset
        AESRST OFFSET(17) NUMBITS(1) [],
        /// PKA hardware accelerator reset
        PKARST OFFSET(16) NUMBITS(1) [],
    ],
    APB1RSTR1 [
        /// Low-power timer 1 reset
        LPTIM1RST OFFSET(31) NUMBITS(1) [],
        /// DAC reset
        DACRST OFFSET(29) NUMBITS(1) [],
        /// I2C3 reset
        I2C3RST OFFSET(23) NUMBITS(1) [],
        /// I2C2 reset
        I2C2RST OFFSET(22) NUMBITS(1) [],
        /// I2C1 reset
        I2C1RST OFFSET(21) NUMBITS(1) [],
        /// USART2 reset
        USART2RST OFFSET(17) NUMBITS(1) [],
        /// SPI2S2 reset
        SPI2S2RST OFFSET(14) NUMBITS(1) [],
        /// TIM2 timer reset
        TIM2RST OFFSET(0) NUMBITS(1) [],
    ],
    APB1RSTR2 [
        /// Low-power timer 3 reset
        LPTIM3RST OFFSET(6) NUMBITS(1) [],
        /// Low-power timer 2 reset
        LPTIM2RST OFFSET(5) NUMBITS(1) [],
        /// Low-power UART 1 reset
        LPUART1RST OFFSET(0) NUMBITS(1) [],
    ],
    APB2RSTR [
        /// Timer 17 reset
        TIM17RST OFFSET(18) NUMBITS(1) [],
        /// Timer 16 reset
        TIM16RST OFFSET(17) NUMBITS(1) [],
        /// USART1 reset
        USART1RST OFFSET(14) NUMBITS(1) [],
        /// SPI1 reset
        SPI1RST OFFSET(12) NUMBITS(1) [],
        /// Timer 1 reset
        TIM1RST OFFSET(11) NUMBITS(1) [],
        /// ADC reset
        ADCRST OFFSET(9) NUMBITS(1) [],
    ],
    APB3RSTR [
        /// Sub-GHz radio SPI reset
        SUBGHZSPIRST OFFSET(0) NUMBITS(1) [],
    ],
    AHB1ENR [
        /// CRC clock enable
        CRCEN OFFSET(12) NUMBITS(1) [],
        /// DMAMUX1 clock enable
        DMAMUX1EN OFFSET(2) NUMBITS(1) [],
        /// DMA2 clock enable
        DMA2EN OFFSET(1) NUMBITS(1) [],
        /// DMA1 clock enable
        DMA1EN OFFSET(0) NUMBITS(1) [],
    ],
    AHB2ENR [
        /// IO port H clock enable
        GPIOHEN OFFSET(7) NUMBITS(1) [],
        /// IO port C clock enable
        GPIOCEN OFFSET(2) NUMBITS(1) [],
        /// IO port B clock enable
        GPIOBEN OFFSET(1) NUMBITS(1) [],
        /// IO port A clock enable
        GPIOAEN OFFSET(0) NUMBITS(1) [],
    ],
    AHB3ENR [
        /// Flash memory interface clock enable
        FLASHEN OFFSET(25) NUMBITS(1) [],
        /// HSEM clock enable
        HSEMEN OFFSET(19) NUMBITS(1) [],
        /// True RNG clock enable
        RNGEN OFFSET(18) NUMBITS(1) [],
        /// AES hardware accelerator clock enable
        AESEN OFFSET(17) NUMBITS(1) [],
        /// PKA hardware accelerator clock enable
        PKAEN OFFSET(16) NUMBITS(1) [],
    ],
    APB1ENR1 [
        /// Low power timer 1 clocks enable
        LPTIM1EN OFFSET(31) NUMBITS(1) [],
        /// DAC clock enable
        DACEN OFFSET(29) NUMBITS(1) [],
        /// I2C3 clock enable
        I2C3EN OFFSET(23) NUMBITS(1) [],
        /// I2C2 clock enable
        I2C2EN OFFSET(22) NUMBITS(1) [],
        /// I2C1 clock enable
        I2C1EN OFFSET(21) NUMBITS(1) [],
        /// USART2 clock enable
        USART2EN OFFSET(17) NUMBITS(1) [],
        /// SPI2S2 clock enable
        SPI2S2EN OFFSET(14) NUMBITS(1) [],
        /// Window watchdog clock enable
        WWDGEN OFFSET(11) NUMBITS(1) [],
        /// RTC APB bus clock enable
        RTCAPBEN OFFSET(10) NUMBITS(1) [],
        /// Timer 2 clock enable
        TIM2EN OFFSET(0) NUMBITS(1) [],

    ],
    APB1ENR2[
        /// Low power timer 3 clock enable
        LPTIM3EN OFFSET(6) NUMBITS(1) [],
        /// Low power timer 2 clock enable
        LPTIM2EN OFFSET(5) NUMBITS(1) [],
        /// Low power UART 1 clock enable
        LPUART1EN OFFSET(0) NUMBITS(1) [],
    ],
    APB2ENR [
        /// Timer 17 clock enable
        TIM17EN OFFSET(18) NUMBITS(1) [],
        /// Timer 16 clock enable
        TIM16EN OFFSET(17) NUMBITS(1) [],
        /// USART1 clock enable
        USART1EN OFFSET(14) NUMBITS(1) [],
        /// SPI1 clock enable
        SPI1EN OFFSET(12) NUMBITS(1) [],
        /// Timer 1 clock enable
        TIM1EN OFFSET(11) NUMBITS(1) [],
        /// ADC clock enable
        ADCEN OFFSET(9) NUMBITS(1) [],
    ],
    APB3ENR [
        /// Sub-GHz radio SPI clock enable
        SUBGHZSPIEN OFFSET(0) NUMBITS(1) [],
    ],
    AHB1SMENR [
        /// CRC clock enable during Sleep mode
        CRCEN OFFSET(12) NUMBITS(1) [],
        /// DMAMUX1 clock enable during Sleep mode
        DMAMUX1EN OFFSET(2) NUMBITS(1) [],
        /// DMA2 clock enable during Sleep mode
        DMA2EN OFFSET(1) NUMBITS(1) [],
        /// DMA1 clock enable during Sleep mode
        DMA1EN OFFSET(0) NUMBITS(1) [],
    ],
    AHB2SMENR [
        /// IO port H clock enable during Sleep mode
        GPIOHEN OFFSET(7) NUMBITS(1) [],
        /// IO port C clock enable during Sleep mode
        GPIOCEN OFFSET(2) NUMBITS(1) [],
        /// IO port B clock enable during Sleep mode
        GPIOBEN OFFSET(1) NUMBITS(1) [],
        /// IO port A clock enable during Sleep mode
        GPIOAEN OFFSET(0) NUMBITS(1) [],
    ],
    AHB3SMENR [
        /// Flash memory interface clock enable during Sleep mode
        FLASHEN OFFSET(25) NUMBITS(1) [],
        /// SRAM2 memory interface clock enable during Sleep mode
        SRAM2SMEN OFFSET(24) NUMBITS(1) [],
        /// SRAM1 interface clock enble during Sleep mode
        SRAM1SMEN OFFSET(23) NUMBITS(1) [],
        /// True RNG clock enable during Sleep mode
        RNGEN OFFSET(18) NUMBITS(1) [],
        /// AES hardware accelerator clock enable during Sleep mode
        AESEN OFFSET(17) NUMBITS(1) [],
        /// PKA hardware accelerator clock enable during Sleep mode
        PKAEN OFFSET(16) NUMBITS(1) [],
    ],
    APB1SMENR1 [
        /// Low power timer 1 clock enable during Sleep and Stop modes
        LPTIM1SMEN OFFSET(31) NUMBITS(1) [],
        /// DAC clock enable during Sleep and Stop modes
        DACSMEN OFFSET(29) NUMBITS(1) [],
        /// I2C3 clock enable during Sleep and Stop modes
        I2C3SMEN OFFSET(23) NUMBITS(1) [],
        /// I2C2 clock enable during Sleep and Stop modes
        I2C2SMEN OFFSET(22) NUMBITS(1) [],
        /// I2C1 clock enable during Sleep and Stop modes
        I2C1SMEN OFFSET(21) NUMBITS(1) [],
        /// USART2 clock enable during Sleep and Stop modes
        USART2SMEN OFFSET(17) NUMBITS(1) [],
        /// SPI2S2 clock enable during Sleep and Stop modes
        SPI2S2SMEN OFFSET(14) NUMBITS(1) [],
        /// Window watchdog clock enable during Sleep and Stop modes
        WWDGSMEN OFFSET(11) NUMBITS(1) [],
        /// RTC APB bus clock enable during Sleep and Stop modes
        RTCAPBSMEN OFFSET(10) NUMBITS(1) [],
        /// Timer 2 clock enable during Sleep and Stop modes
        TIM2SMEN OFFSET(0) NUMBITS(1) [],
    ],
    APB1SMENR2 [
        /// Low power timer 3 clock enable during Sleep and Stop modes
        LPTIM3SMEN OFFSET(6) NUMBITS(1) [],
        /// Low power timer 2 clock enable during Sleep and Stop modes
        LPTIM2SMEN OFFSET(5) NUMBITS(1) [],
        /// Low power UART 1 clock enable during Sleep and Stop modes
        LPUART1SMEN OFFSET(0) NUMBITS(1) [],
    ],
    APB2SMENR [
        /// Timer 17 clock enable during Sleep and Stop modes
        TIM17SMEN OFFSET(18) NUMBITS(1) [],
        /// Timer 16 clock enable during Sleep and Stop modes
        TIM16SMEN OFFSET(17) NUMBITS(1) [],
        /// USART1 clock enable during Sleep and Stop modes
        USART1SMEN OFFSET(14) NUMBITS(1) [],
        /// SPI1 clock enable during Sleep and Stop modes
        SPI1SMEN OFFSET(12) NUMBITS(1) [],
        /// Timer 1 clock enable during Sleep and Stop modes
        TIM1SMEN OFFSET(11) NUMBITS(1) [],
        /// ADC clock enable during Sleep and Stop modes
        ADCSMEN OFFSET(9) NUMBITS(1) [],
    ],
    APB3SMENR [
        /// Sub-GHz radio SPI clock enable during Sleep and Stop modes
        SUBGHZSPISMEN OFFSET(0) NUMBITS(1) [],
    ],
    CCIPR [
        /// RNG clock source selection
        RNGSEL OFFSET(30) NUMBITS(2) [
            PLLQ = 0b00,
            LSI = 0b01,
            LSE = 0b10,
            MSI = 0b11,
        ],
        /// ADC clock source selection
        ADCSEL OFFSET(28) NUMBITS(2) [
            None = 0b00,
            HSI = 0b01,
            PLLP = 0b10,
            SYSCLK = 0b11,
        ],
        /// Low-power timer 3 clock source selection
        LPTIM3SEL OFFSET(22) NUMBITS(2) [
            PCLK = 0b00,
            LSI = 0b01,
            HSI16 = 0b10,
            LSE = 0b11,
        ],
        /// Low-power timer 2 clock source selection
        LPTIM2SEL OFFSET(20) NUMBITS(2) [
            PCLK = 0b00,
            LSI = 0b01,
            HSI16 = 0b10,
            LSE = 0b11,
        ],
        /// Low-power timer 1 clock source selection
        LPTIM1SEL OFFSET(18) NUMBITS(2) [
            PCLK = 0b00,
            LSI = 0b01,
            HSI16 = 0b10,
            LSE = 0b11,
        ],
        /// I2C3 clock source selection
        I2C3SEL OFFSET(16) NUMBITS(2) [
            PCLK = 0b00,
            SYSCLK = 0b01,
            HSI16 = 0b10,
        ],
        /// I2C2 clock source selection
        I2C2SEL OFFSET(14) NUMBITS(2) [
            PCLK = 0b00,
            SYSCLK = 0b01,
            HSI16 = 0b10,
        ],
        /// I2C1 clock source selection
        I2C1SEL OFFSET(12) NUMBITS(2) [
            PCLK = 0b00,
            SYSCLK = 0b01,
            HSI16 = 0b10,
        ],
        /// LPUART1 clock source selection
        LPUART1SEL OFFSET(10) NUMBITS(2) [
            PCLK = 0b00,
            SYSCLK = 0b01,
            HSI16 = 0b10,
            LSE = 0b11,
        ],
        /// SPI2S2 I@S clock source selection
        SPI2S2SEL OFFSET(8) NUMBITS(2) [
            PLLQ = 0b01,
            HSI16 = 0b10,
            I2S_CKIN = 0b11,
        ],
        /// USART2 clock source selection
        USART2SEL OFFSET(2) NUMBITS(2) [
            PCLK = 0b00,
            SYSCLK = 0b01,
            HSI16 = 0b10,
            LSE = 0b11,
        ],
        /// USART1 clock source selection
        USART1SEL OFFSET(0) NUMBITS(2) [
            PCLK = 0b00,
            SYSCLK = 0b01,
            HSI16 = 0b10,
            LSE = 0b11,
        ],
    ],
    BDCR [
        /// Low-speed clock output selection
        LSCOSEL OFFSET(25) NUMBITS(1) [
            LSI = 0,
            LSE = 1,
        ],
        /// Low-speed clock output enable
        LSCOEN OFFSET(24) NUMBITS(1) [],
        /// Backup domain software reset
        BDRST OFFSET(16) NUMBITS(1) [],
        /// RTC kernel clock enable
        RTCEN OFFSET(15) NUMBITS(1) [],
        /// LSE system clock ready
        LSESYSRDY OFFSET(11) NUMBITS(1) [],
        /// RTC clock source selection
        RTCSEL OFFSET(8) NUMBITS(2) [
            None = 0b00,
            LSE = 0b01,
            LSI = 0b10,
            HSE32 = 0b11,
        ],
        /// LSE system clock enable
        LSESYSEN OFFSET(7) NUMBITS(1) [],
        /// CSS on LSE failure detection
        LSECSSD OFFSET(6) NUMBITS(1) [],
        /// CSS on LSE enable
        LSECSSON OFFSET(5) NUMBITS(1) [],
        /// LSE oscillator drive capability
        LSEDRV OFFSET(3) NUMBITS(2) [
            Low = 0b00,
            MediumLow = 0b01,
            MediumHigh = 0b10,
            High = 0b11,
        ],
        /// LSE oscillator bypass
        LSEBYP OFFSET(2) NUMBITS(1) [],
        /// LSE oscillator ready
        LSERDY OFFSET(1) NUMBITS(1) [],
        /// LSE oscillator enable
        LSEON OFFSET(0) NUMBITS(1) [],
    ],
    CSR [
        /// Low-power reset flag
        LPWRSTF OFFSET(31) NUMBITS(1) [],
        /// Window watchdog reset flag
        WWDGRSTF OFFSET(30) NUMBITS(1) [],
        /// Independent window watchdog reset flag
        IWWDGRSTF OFFSET(29) NUMBITS(1) [],
        /// Software reset flag
        SFTRSTF OFFSET(28) NUMBITS(1) [],
        /// BOR reset flag
        BORRSTF OFFSET(27) NUMBITS(1) [],
        /// Pin reset flag
        PINRSTF OFFSET(26) NUMBITS(1) [],
        /// Option byte loader reset flag
        OBLRSTF OFFSET(25) NUMBITS(1) [],
        /// Sub-GHz radio illegal command flag
        SUBGHZSPICF OFFSET(24) NUMBITS(1) [],
        /// Remove reset flag
        RMVF OFFSET(23) NUMBITS(1) [],
        /// Sub-GHz radio reset
        RFRST OFFSET(15) NUMBITS(1) [],
        /// Sub-GHz radio in reset status flag
        RFRSTF OFFSET(14) NUMBITS(1) [],
        /// MSI clock ranges
        MSISRANGE OFFSET(8) NUMBITS(4) [
            Range4 = 0b100,
            Range5 = 0b101,
            Range6 = 0b110,
            Range7 = 0b111,
        ],
        /// LSI frequency prescaler
        LSIPRE OFFSET(4) NUMBITS(1) [],
        /// LSI oscillator ready
        LSIRDY OFFSET(1) NUMBITS(1) [],
        /// LSI oscillator enable
        LSION OFFSET(0) NUMBITS(1) [],

    ],
    EXTCFGR [
        /// HCLK3 shared prescaler flag (AHB3, Flash, and SRAM1/2)
        SHDHPREF OFFSET(16) NUMBITS(1) [],
        /// HCLK3 shared prescaler (AHB3, Flash, and SRAM1/2)
        SHDHPRE OFFSET(0) NUMBITS(4) [
        ],
    ],
];

const RCC_BASE: StaticRef<RccRegisters> =
    unsafe { StaticRef::new(0x58000000 as *const RccRegisters) };

pub(crate) const PLLN_MULTIPLY_28: usize = 28;

pub struct Rcc {
    registers: StaticRef<RccRegisters>,
}

pub enum RtcClockSource {
    LSI,
    LSE,
    HSERTC,
}

pub(crate) const DEFAULT_PLLM_VALUE: PLLMDivider = PLLMDivider::DivideBy2;
pub(crate) const DEFAULT_PLLN_VALUE: usize = PLLN_MULTIPLY_28;
pub(crate) const DEFAULT_PLLP_VALUE: PLLPDivider = PLLPDivider::DivideBy8;
pub(crate) const DEFAULT_PLLQ_VALUE: PLLQDivider = PLLQDivider::DivideBy8;

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

    // Init the PLL clock.
    // CONSTRAINTS:
    //  - f(VCO) must be between 96 and 344 MHz
    //  - PLL input frequency must be between 2.66 and 16 MHz
    //  - PLLQ & PLLR frequency must not exceed 48 MHz
    //  - PLLP frequency must not exceed 62 MHz
    //  - PLLN multipler must be between 6 and 127
    // FORMULAS:
    //  - f(PLL)       = f(PLL clock input) / PLLM
    //  - f(VCO clock) = f(PLL clock input) x (PLLN / PLLM)
    //  - f(PLL_P)     = f(VCO clock) / PLLP
    //  - f(PLL_Q)     = f(VCO clock) / PLLQ
    //  - f(PLL_R)     = f(VCO clock) / PLLR
    // CONFIGURATION:
    //   + CLOCK SRC: HSI16
    //   + PLLM: DivideBy2
    //   + 8MHz PLL input frequency: 16 MHz / PLLM (PLLM=2)
    //   + 224MHz VCO output frequency: (8MHz) X PLLN (PLLN=28)
    //   + 28MHz freq_PLLQ & freq_PLLR & freq_PLLP: (224 MHz) / PLLQ (PLLQ=PLLR=8)
    fn init_pll_clock(&self) {
        self.set_pll_clocks_source(PllSource::HSI);
        self.set_pll_clocks_m_divider(PLLMDivider::DivideBy2);
        self.set_pll_clock_n_multiplier(PLLN_MULTIPLY_28);
        self.set_pll_clock_p_divider(PLLPDivider::DivideBy8);
        self.set_pll_clock_q_divider(PLLQDivider::DivideBy8);
        self.set_pll_clock_r_divider(PLLRDivider::DivideBy8);
    }

    // Get the current system clock source
    pub(crate) fn get_sys_clock_source(&self) -> SysClockSource {
        match self.registers.cfgr.read(CFGR::SWS) {
            0b00 => SysClockSource::MSI,
            0b01 => SysClockSource::HSI,
            0b10 => SysClockSource::HSE,
            _ => SysClockSource::PLLR,
        }
    }

    // Set the system clock source
    // The source must be enabled
    // NOTE: The flash latency also needs to be configured when changing the system clock frequency
    pub(crate) fn set_sys_clock_source(&self, source: SysClockSource) {
        self.registers.cfgr.modify(CFGR::SW.val(source as u32));
    }

    pub(crate) fn is_msi_clock_system_clock(&self) -> bool {
        let system_clock_source = self.get_sys_clock_source();
        system_clock_source == SysClockSource::MSI
            || system_clock_source == SysClockSource::PLLR
                && self.registers.pllcfgr.read(PLLCFGR::PLLSRC) == PllSource::MSI as u32
    }

    pub(crate) fn is_hsi_clock_system_clock(&self) -> bool {
        let system_clock_source = self.get_sys_clock_source();
        system_clock_source == SysClockSource::HSI
            || system_clock_source == SysClockSource::PLLR
                && self.registers.pllcfgr.read(PLLCFGR::PLLSRC) == PllSource::HSI as u32
    }

    pub(crate) fn is_hse_clock_system_clock(&self) -> bool {
        let system_clock_source = self.get_sys_clock_source();
        system_clock_source == SysClockSource::HSE
            || system_clock_source == SysClockSource::PLLR
                && self.registers.pllcfgr.read(PLLCFGR::PLLSRC) == PllSource::HSE as u32
    }

    /* MSI close */
    pub(crate) fn disable_msi_clock(&self) {
        self.registers.cr.modify(CR::MSION::CLEAR);
    }

    pub(crate) fn enable_msi_clock(&self) {
        self.registers.cr.modify(CR::MSION::SET);
    }

    pub(crate) fn is_enabled_msi_clock(&self) -> bool {
        self.registers.cr.is_set(CR::MSION)
    }

    pub(crate) fn is_ready_msi_clock(&self) -> bool {
        self.registers.cr.is_set(CR::MSIRDY)
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
        self.registers.cr.modify(CR::HSEBYPPWR::CLEAR);
    }

    pub(crate) fn enable_hse_clock_bypass(&self) {
        self.registers.cr.modify(CR::HSEBYPPWR::SET);
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
            0b0 => PllSource::None,
            0b01 => PllSource::MSI,
            0b10 => PllSource::HSI,
            0b11 => PllSource::HSE,
            _ => unreachable!(),
        }
    }

    // This method must be called only when all PLL clocks are disabled
    pub(crate) fn set_pll_clocks_source(&self, source: PllSource) {
        self.registers
            .pllcfgr
            .modify(PLLCFGR::PLLSRC.val(source as u32));
    }

    pub(crate) fn get_pll_clocks_m_divider(&self) -> PLLMDivider {
        match self.registers.pllcfgr.read(PLLCFGR::PLLM) {
            0b000 => PLLMDivider::DivideBy1,
            0b001 => PLLMDivider::DivideBy2,
            0b010 => PLLMDivider::DivideBy3,
            0b011 => PLLMDivider::DivideBy4,
            0b100 => PLLMDivider::DivideBy5,
            0b101 => PLLMDivider::DivideBy6,
            0b110 => PLLMDivider::DivideBy7,
            0b111 => PLLMDivider::DivideBy8,
            _ => unreachable!(),
        }
    }

    // This method must be called only when all PLL clocks are disabled
    pub(crate) fn set_pll_clocks_m_divider(&self, m: PLLMDivider) {
        self.registers.pllcfgr.modify(PLLCFGR::PLLM.val(m as u32));
    }

    pub(crate) fn get_pll_clock_n_multiplier(&self) -> usize {
        self.registers.pllcfgr.read(PLLCFGR::PLLN) as usize
    }

    // This method must be called only if the main PLL clock is disabled
    pub(crate) fn set_pll_clock_n_multiplier(&self, n: usize) {
        self.registers.pllcfgr.modify(PLLCFGR::PLLN.val(n as u32));
    }

    pub(crate) fn get_pll_clock_p_divider(&self) -> PLLPDivider {
        let val = self.registers.pllcfgr.read(PLLCFGR::PLLP);

        // PLLP divider is val + 1
        match val {
            1 => PLLPDivider::DivideBy2,
            2 => PLLPDivider::DivideBy3,
            3 => PLLPDivider::DivideBy4,
            4 => PLLPDivider::DivideBy5,
            5 => PLLPDivider::DivideBy6,
            6 => PLLPDivider::DivideBy7,
            7 => PLLPDivider::DivideBy8,
            8 => PLLPDivider::DivideBy9,
            9 => PLLPDivider::DivideBy10,
            10 => PLLPDivider::DivideBy11,
            11 => PLLPDivider::DivideBy12,
            12 => PLLPDivider::DivideBy13,
            13 => PLLPDivider::DivideBy14,
            14 => PLLPDivider::DivideBy15,
            15 => PLLPDivider::DivideBy16,
            16 => PLLPDivider::DivideBy17,
            17 => PLLPDivider::DivideBy18,
            18 => PLLPDivider::DivideBy19,
            19 => PLLPDivider::DivideBy20,
            20 => PLLPDivider::DivideBy21,
            21 => PLLPDivider::DivideBy22,
            22 => PLLPDivider::DivideBy23,
            23 => PLLPDivider::DivideBy24,
            24 => PLLPDivider::DivideBy25,
            25 => PLLPDivider::DivideBy26,
            26 => PLLPDivider::DivideBy27,
            27 => PLLPDivider::DivideBy28,
            28 => PLLPDivider::DivideBy29,
            29 => PLLPDivider::DivideBy30,
            30 => PLLPDivider::DivideBy31,
            31 => PLLPDivider::DivideBy32,
            _ => unreachable!(),
        }
    }

    // This method must be called only if the main PLL clock is disabled
    pub(crate) fn set_pll_clock_p_divider(&self, p: PLLPDivider) {
        self.registers.pllcfgr.modify(PLLCFGR::PLLP.val(p as u32));
    }

    pub(crate) fn _get_pll_clock_q_divider(&self) -> PLLQDivider {
        match self.registers.pllcfgr.read(PLLCFGR::PLLQ) {
            1 => PLLQDivider::DivideBy2,
            2 => PLLQDivider::DivideBy3,
            3 => PLLQDivider::DivideBy4,
            4 => PLLQDivider::DivideBy5,
            5 => PLLQDivider::DivideBy6,
            6 => PLLQDivider::DivideBy7,
            7 => PLLQDivider::DivideBy8,
            _ => panic!("Unexpected PLLQ divider"),
        }
    }

    // This method must be called only if the main PLL clock is disabled
    pub(crate) fn set_pll_clock_q_divider(&self, q: PLLQDivider) {
        self.registers.pllcfgr.modify(PLLCFGR::PLLQ.val(q as u32));
    }

    // This method must be called only if the main PLL clock is disabled
    pub(crate) fn set_pll_clock_r_divider(&self, r: PLLRDivider) {
        self.registers.pllcfgr.modify(PLLCFGR::PLLR.val(r as u32));
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

    pub(crate) fn get_apb3_prescaler(&self) -> APBPrescaler {
        // From RM0461 6.2.13:
        // The SUBGHZSPI_SCK frequency is obtained by PCLK3 divided
        // by two. The SUBGHZSPI_SCK clock maximum speed must not exceed 16 MHz.
        APBPrescaler::DivideBy2
    }

    pub(crate) fn set_mco1_clock_source(&self, source: MCOSource) {
        self.registers.cfgr.modify(CFGR::MCOSEL.val(source as u32));
    }

    pub(crate) fn get_mco1_clock_source(&self) -> MCOSource {
        match self.registers.cfgr.read(CFGR::MCOSEL) {
            0b0000 => MCOSource::DISABLED,
            0b0001 => MCOSource::SYSCLK,
            0b0010 => MCOSource::MSI,
            0b0011 => MCOSource::HSI,
            0b0100 => MCOSource::HSE,
            0b0101 => MCOSource::PLLR,
            0b0110 => MCOSource::LSI,
            0b1000 => MCOSource::LSE,
            0b1101 => MCOSource::PLLPCLK,
            0b1110 => MCOSource::PLLQCLK,
            _ => unreachable!(),
        }
    }

    pub(crate) fn set_mco_clock_divider(&self, divider: MCODivider) {
        self.registers.cfgr.modify(CFGR::MCOPRE.val(divider as u32));
    }

    pub(crate) fn get_mco_clock_divider(&self) -> MCODivider {
        match self.registers.cfgr.read(CFGR::MCOPRE) {
            0b000 => MCODivider::DivideBy1,
            0b001 => MCODivider::DivideBy2,
            0b010 => MCODivider::DivideBy4,
            0b011 => MCODivider::DivideBy8,
            0b100 => MCODivider::DivideBy16,
            _ => unreachable!(),
        }
    }

    // I2C1 clock
    pub(crate) fn is_enabled_i2c1_clock(&self) -> bool {
        self.registers.apb1enr1.is_set(APB1ENR1::I2C1EN)
    }

    pub(crate) fn enable_i2c1_clock(&self) {
        self.registers.apb1enr1.modify(APB1ENR1::I2C1EN::SET);
        self.registers.apb1rstr1.modify(APB1RSTR1::I2C1RST::SET);
        self.registers.apb1rstr1.modify(APB1RSTR1::I2C1RST::CLEAR);
    }

    pub(crate) fn disable_i2c1_clock(&self) {
        self.registers.apb1enr1.modify(APB1ENR1::I2C1EN::CLEAR)
    }

    // I2C2 clock
    pub(crate) fn is_enabled_i2c2_clock(&self) -> bool {
        self.registers.apb1enr1.is_set(APB1ENR1::I2C2EN)
    }

    pub(crate) fn enable_i2c2_clock(&self) {
        self.registers.apb1enr1.modify(APB1ENR1::I2C2EN::SET);
        self.registers.apb1rstr1.modify(APB1RSTR1::I2C2RST::SET);
        self.registers.apb1rstr1.modify(APB1RSTR1::I2C2RST::CLEAR);
    }

    pub(crate) fn disable_i2c2_clock(&self) {
        self.registers.apb1enr1.modify(APB1ENR1::I2C2EN::CLEAR)
    }

    // SPI1 clock
    pub(crate) fn is_enabled_spi1_clock(&self) -> bool {
        self.registers.apb2enr.is_set(APB2ENR::SPI1EN)
    }

    pub(crate) fn enable_spi1_clock(&self) {
        self.registers.apb2enr.modify(APB2ENR::SPI1EN::SET)
    }

    pub(crate) fn disable_spi1_clock(&self) {
        self.registers.apb2enr.modify(APB2ENR::SPI1EN::CLEAR)
    }

    // TIM2 clock
    pub(crate) fn is_enabled_tim2_clock(&self) -> bool {
        self.registers.apb1enr1.is_set(APB1ENR1::TIM2EN)
    }

    pub(crate) fn enable_tim2_clock(&self) {
        self.registers.apb1enr1.modify(APB1ENR1::TIM2EN::SET)
    }

    pub(crate) fn disable_tim2_clock(&self) {
        self.registers.apb1enr1.modify(APB1ENR1::TIM2EN::CLEAR)
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
        self.registers.ahb2enr.is_set(AHB2ENR::GPIOHEN)
    }

    pub(crate) fn enable_gpioh_clock(&self) {
        self.registers.ahb2enr.modify(AHB2ENR::GPIOHEN::SET)
    }

    pub(crate) fn disable_gpioh_clock(&self) {
        self.registers.ahb2enr.modify(AHB2ENR::GPIOHEN::CLEAR)
    }

    // GPIOC clock
    pub(crate) fn is_enabled_gpioc_clock(&self) -> bool {
        self.registers.ahb2enr.is_set(AHB2ENR::GPIOCEN)
    }

    pub(crate) fn enable_gpioc_clock(&self) {
        self.registers.ahb2enr.modify(AHB2ENR::GPIOCEN::SET)
    }

    pub(crate) fn disable_gpioc_clock(&self) {
        self.registers.ahb2enr.modify(AHB2ENR::GPIOCEN::CLEAR)
    }

    // GPIOB clock
    pub(crate) fn is_enabled_gpiob_clock(&self) -> bool {
        self.registers.ahb2enr.is_set(AHB2ENR::GPIOBEN)
    }

    pub(crate) fn enable_gpiob_clock(&self) {
        self.registers.ahb2enr.modify(AHB2ENR::GPIOBEN::SET)
    }

    pub(crate) fn disable_gpiob_clock(&self) {
        self.registers.ahb2enr.modify(AHB2ENR::GPIOBEN::CLEAR)
    }

    // GPIOA clock
    pub(crate) fn is_enabled_gpioa_clock(&self) -> bool {
        self.registers.ahb2enr.is_set(AHB2ENR::GPIOAEN)
    }

    pub(crate) fn enable_gpioa_clock(&self) {
        self.registers.ahb2enr.modify(AHB2ENR::GPIOAEN::SET)
    }

    pub(crate) fn disable_gpioa_clock(&self) {
        self.registers.ahb2enr.modify(AHB2ENR::GPIOAEN::CLEAR)
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
        self.registers.apb1enr1.is_set(APB1ENR1::USART2EN)
    }

    pub(crate) fn enable_usart2_clock(&self) {
        self.registers.apb1enr1.modify(APB1ENR1::USART2EN::SET)
    }

    pub(crate) fn disable_usart2_clock(&self) {
        self.registers.apb1enr1.modify(APB1ENR1::USART2EN::CLEAR)
    }

    // ADC1 clock
    pub(crate) fn is_enabled_adc1_clock(&self) -> bool {
        self.registers.apb2enr.is_set(APB2ENR::ADCEN)
    }

    pub(crate) fn enable_adc1_clock(&self) {
        self.registers.apb2enr.modify(APB2ENR::ADCEN::SET)
    }

    pub(crate) fn disable_adc1_clock(&self) {
        self.registers.apb2enr.modify(APB2ENR::ADCEN::CLEAR)
    }

    // SUBGHZSPI clock
    pub(crate) fn is_enabled_subghzspi_clock(&self) -> bool {
        self.registers.apb3enr.is_set(APB3ENR::SUBGHZSPIEN)
    }

    pub(crate) fn enable_subghzspi_clock(&self) {
        self.registers.apb3enr.modify(APB3ENR::SUBGHZSPIEN::SET)
    }

    pub(crate) fn disable_subghzspi_clock(&self) {
        self.registers.apb3enr.modify(APB3ENR::SUBGHZSPIEN::CLEAR)
    }

    // DAC clock
    pub(crate) fn is_enabled_dac_clock(&self) -> bool {
        self.registers.apb1enr1.is_set(APB1ENR1::DACEN)
    }

    pub(crate) fn enable_dac_clock(&self) {
        self.registers.apb1enr1.modify(APB1ENR1::DACEN::SET)
    }

    pub(crate) fn disable_dac_clock(&self) {
        self.registers.apb1enr1.modify(APB1ENR1::DACEN::CLEAR)
    }

    // RNG clock

    pub(crate) fn is_enabled_rng_clock(&self) -> bool {
        self.registers.ahb3enr.is_set(AHB3ENR::RNGEN)
    }

    pub(crate) fn enable_rng_clock(&self) {
        self.registers.ahb3enr.modify(AHB3ENR::RNGEN::SET);
    }

    pub(crate) fn disable_rng_clock(&self) {
        self.registers.ahb3enr.modify(AHB3ENR::RNGEN::CLEAR);
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

    pub(crate) fn is_enabled_rtc_clock(&self) -> bool {
        self.registers.apb3enr.is_set(APB3ENR::SUBGHZSPIEN)
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
pub(crate) enum PLLPDivider {
    DivideBy2 = 0b00001,
    DivideBy3 = 0b00010,
    DivideBy4 = 0b00011,
    DivideBy5 = 0b00100,
    DivideBy6 = 0b00101,
    DivideBy7 = 0b00110,
    DivideBy8 = 0b00111,
    DivideBy9 = 0b01000,
    DivideBy10 = 0b01001,
    DivideBy11 = 0b01010,
    DivideBy12 = 0b01011,
    DivideBy13 = 0b01100,
    DivideBy14 = 0b01101,
    DivideBy15 = 0b01110,
    DivideBy16 = 0b01111,
    DivideBy17 = 0b10000,
    DivideBy18 = 0b10001,
    DivideBy19 = 0b10010,
    DivideBy20 = 0b10011,
    DivideBy21 = 0b10100,
    DivideBy22 = 0b10101,
    DivideBy23 = 0b10110,
    DivideBy24 = 0b10111,
    DivideBy25 = 0b11000,
    DivideBy26 = 0b11001,
    DivideBy27 = 0b11010,
    DivideBy28 = 0b11011,
    DivideBy29 = 0b11100,
    DivideBy30 = 0b11101,
    DivideBy31 = 0b11110,
    DivideBy32 = 0b11111,
}

impl From<PLLPDivider> for usize {
    fn from(item: PLLPDivider) -> Self {
        match item {
            PLLPDivider::DivideBy2 => 2,
            PLLPDivider::DivideBy3 => 3,
            PLLPDivider::DivideBy4 => 4,
            PLLPDivider::DivideBy5 => 5,
            PLLPDivider::DivideBy6 => 6,
            PLLPDivider::DivideBy7 => 7,
            PLLPDivider::DivideBy8 => 8,
            PLLPDivider::DivideBy9 => 9,
            PLLPDivider::DivideBy10 => 10,
            PLLPDivider::DivideBy11 => 11,
            PLLPDivider::DivideBy12 => 12,
            PLLPDivider::DivideBy13 => 13,
            PLLPDivider::DivideBy14 => 14,
            PLLPDivider::DivideBy15 => 15,
            PLLPDivider::DivideBy16 => 16,
            PLLPDivider::DivideBy17 => 17,
            PLLPDivider::DivideBy18 => 18,
            PLLPDivider::DivideBy19 => 19,
            PLLPDivider::DivideBy20 => 20,
            PLLPDivider::DivideBy21 => 21,
            PLLPDivider::DivideBy22 => 22,
            PLLPDivider::DivideBy23 => 23,
            PLLPDivider::DivideBy24 => 24,
            PLLPDivider::DivideBy25 => 25,
            PLLPDivider::DivideBy26 => 26,
            PLLPDivider::DivideBy27 => 27,
            PLLPDivider::DivideBy28 => 28,
            PLLPDivider::DivideBy29 => 29,
            PLLPDivider::DivideBy30 => 30,
            PLLPDivider::DivideBy31 => 31,
            PLLPDivider::DivideBy32 => 32,
        }
    }
}

#[allow(dead_code)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum PLLRDivider {
    DivideBy2 = 0b001,
    DivideBy3 = 0b010,
    DivideBy4 = 0b011,
    DivideBy5 = 0b100,
    DivideBy6 = 0b101,
    DivideBy7 = 0b110,
    DivideBy8 = 0b111,
}

impl From<PLLRDivider> for usize {
    fn from(item: PLLRDivider) -> Self {
        match item {
            PLLRDivider::DivideBy2 => 2,
            PLLRDivider::DivideBy3 => 3,
            PLLRDivider::DivideBy4 => 4,
            PLLRDivider::DivideBy5 => 5,
            PLLRDivider::DivideBy6 => 6,
            PLLRDivider::DivideBy7 => 7,
            PLLRDivider::DivideBy8 => 8,
        }
    }
}

#[allow(dead_code)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum PLLQDivider {
    DivideBy2 = 0b001,
    DivideBy3 = 0b010,
    DivideBy4 = 0b011,
    DivideBy5 = 0b100,
    DivideBy6 = 0b101,
    DivideBy7 = 0b110,
    DivideBy8 = 0b111,
}

impl From<PLLQDivider> for usize {
    fn from(item: PLLQDivider) -> Self {
        match item {
            PLLQDivider::DivideBy2 => 2,
            PLLQDivider::DivideBy3 => 3,
            PLLQDivider::DivideBy4 => 4,
            PLLQDivider::DivideBy5 => 5,
            PLLQDivider::DivideBy6 => 6,
            PLLQDivider::DivideBy7 => 7,
            PLLQDivider::DivideBy8 => 8,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum PLLMDivider {
    DivideBy1 = 0b000,
    DivideBy2 = 0b001,
    DivideBy3 = 0b010,
    DivideBy4 = 0b011,
    DivideBy5 = 0b100,
    DivideBy6 = 0b101,
    DivideBy7 = 0b110,
    DivideBy8 = 0b111,
}

impl From<PLLMDivider> for usize {
    fn from(item: PLLMDivider) -> Self {
        match item {
            PLLMDivider::DivideBy1 => 1,
            PLLMDivider::DivideBy2 => 2,
            PLLMDivider::DivideBy3 => 3,
            PLLMDivider::DivideBy4 => 4,
            PLLMDivider::DivideBy5 => 5,
            PLLMDivider::DivideBy6 => 6,
            PLLMDivider::DivideBy7 => 7,
            PLLMDivider::DivideBy8 => 8,
        }
    }
}

/// Clock sources for the CPU
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SysClockSource {
    MSI = 0b00,
    HSI = 0b01,
    HSE = 0b10,
    PLLR = 0b11,
}

pub enum PllSource {
    None = 0b00,
    MSI = 0b01,
    HSI = 0b10,
    HSE = 0b11,
}

pub enum MCOSource {
    DISABLED = 0b0000,
    SYSCLK = 0b0001,
    MSI = 0b0010,
    HSI = 0b0011,
    HSE = 0b0100,
    PLLR = 0b0101,
    LSI = 0b0110,
    LSE = 0b1000,
    PLLPCLK = 0b1101,
    PLLQCLK = 0b1110,
}

pub enum MCODivider {
    DivideBy1 = 0b000,
    DivideBy2 = 0b001,
    DivideBy4 = 0b010,
    DivideBy8 = 0b011,
    DivideBy16 = 0b100,
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

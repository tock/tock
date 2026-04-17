// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Author: Kamil Duljas <kamil.duljas@gmail.com>

use kernel::utilities::registers::interfaces::{ReadWriteable, Readable};
use kernel::utilities::registers::{register_bitfields, ReadWrite};
use kernel::utilities::StaticRef;

/// Reset and clock control
#[repr(C)]
struct RccRegisters {
    /// clock control register
    cr: ReadWrite<u32, CR::Register>,
    /// internal clock sources calibration register
    icscr: ReadWrite<u32, ICSCR::Register>,
    /// clock configuration register
    cfgr: ReadWrite<u32, CFGR::Register>,
    /// system PLL configuration register
    pllcfgr: ReadWrite<u32, PLLCFGR::Register>,
    /// PLL SAI1 configuration register
    pllsai1cfgr: ReadWrite<u32, PLLSAI1CFGR::Register>,
    /// PLL SAI2 configuration register
    pllsai2cfgr: ReadWrite<u32, PLLSAI2CFGR::Register>,
    /// clock interrupt enable register
    cier: ReadWrite<u32, CIER::Register>,
    /// clock interrupt flag register
    cifr: ReadWrite<u32, CIFR::Register>,
    /// clock interrupt clear register
    cicr: ReadWrite<u32, CICR::Register>,
    _reserved0: [u8; 4],
    /// AHB1 peripheral reset register
    ahb1rstr: ReadWrite<u32, AHB1RSTR::Register>,
    /// AHB2 peripheral reset register
    ahb2rstr: ReadWrite<u32, AHB2RSTR::Register>,
    /// AHB3 peripheral reset register
    ahb3rstr: ReadWrite<u32, AHB3RSTR::Register>,
    _reserved1: [u8; 4],
    /// APB1 peripheral reset register 1
    apb1rstr1: ReadWrite<u32, APB1RSTR1::Register>,
    /// APB1 peripheral reset register 2
    apb1rstr2: ReadWrite<u32, APB1RSTR2::Register>,
    /// APB2 peripheral reset register
    apb2rstr: ReadWrite<u32, APB2RSTR::Register>,
    _reserved2: [u8; 4],
    /// AHB1 peripheral clock enable register
    ahb1enr: ReadWrite<u32, AHB1ENR::Register>,
    /// AHB2 peripheral clock enable register
    ahb2enr: ReadWrite<u32, AHB2ENR::Register>,
    /// AHB3 peripheral clock enable register
    ahb3enr: ReadWrite<u32, AHB3ENR::Register>,
    _reserved3: [u8; 4],
    /// APB1 peripheral clock enable register 1
    apb1enr1: ReadWrite<u32, APB1ENR1::Register>,
    /// APB1 peripheral clock enable register 2
    apb1enr2: ReadWrite<u32, APB1ENR2::Register>,
    /// APB2 peripheral clock enable register
    apb2enr: ReadWrite<u32, APB2ENR::Register>,
    _reserved4: [u8; 4],
    /// AHB1 peripheral clocks enable in sleep and stop modes register
    ahb1smenr: ReadWrite<u32, AHB1SMENR::Register>,
    /// AHB2 peripheral clocks enable in sleep and stop modes register
    ahb2smenr: ReadWrite<u32, AHB2SMENR::Register>,
    /// AHB3 peripheral clocks enable in sleep and stop modes register
    ahb3smenr: ReadWrite<u32, AHB3SMENR::Register>,
    _reserved5: [u8; 4],
    /// APB1 peripheral clocks enable in sleep and stop modes register 1
    apb1smenr1: ReadWrite<u32, APB1SMENR1::Register>,
    /// APB1 peripheral clocks enable in sleep and stop modes register 2
    apb1smenr2: ReadWrite<u32, APB1SMENR2::Register>,
    /// APB2 peripheral clocks enable in sleep and stop modes register
    apb2smenr: ReadWrite<u32, APB2SMENR::Register>,
    _reserved6: [u8; 4],
    /// peripherals independent clock configuration register
    ccipr: ReadWrite<u32, CCIPR::Register>,
    _reserved7: [u8; 4],
    /// backup domain control register
    bdcr: ReadWrite<u32, BDCR::Register>,
    /// clock control & status register
    csr: ReadWrite<u32, CSR::Register>,
}

register_bitfields![u32,
    CR [
        /// SAI2 PLL clock ready flag
        PLLSAI2RDY OFFSET(29) NUMBITS(1) [],
        /// SAI2 PLL enable
        PLLSAI2ON OFFSET(28) NUMBITS(1) [],
        /// SAI1 PLL clock ready flag
        PLLSAI1RDY OFFSET(27) NUMBITS(1) [],
        /// SAI1 PLL enable
        PLLSAI1ON OFFSET(26) NUMBITS(1) [],
        /// Main PLL clock ready flag
        PLLRDY OFFSET(25) NUMBITS(1) [],
        /// Main PLL enable
        PLLON OFFSET(24) NUMBITS(1) [],
        /// Clock security system enable
        CSSON OFFSET(19) NUMBITS(1) [],
        /// HSE crystal oscillator bypass
        HSEBYP OFFSET(18) NUMBITS(1) [],
        /// HSE clock ready flag
        HSERDY OFFSET(17) NUMBITS(1) [],
        /// HSE clock enable
        HSEON OFFSET(16) NUMBITS(1) [],
        /// HSI automatic start from Stop
        HSIASFS OFFSET(11) NUMBITS(1) [],
        /// HSI clock ready flag
        HSIRDY OFFSET(10) NUMBITS(1) [],
        /// HSI always enable for peripheral kernels
        HSIKERON OFFSET(9) NUMBITS(1) [],
        /// HSI clock enable
        HSION OFFSET(8) NUMBITS(1) [],
        /// MSI clock ranges
        MSIRANGE OFFSET(4) NUMBITS(4) [
            Range100kHz = 0,
            Range200kHz = 1,
            Range400kHz = 2,
            Range800kHz = 3,
            Range1MHz = 4,
            Range2MHz = 5,
            Range4MHz = 6,
            Range8MHz = 7,
            Range16MHz = 8,
            Range24MHz = 9,
            Range32MHz = 10,
            Range48MHz = 11
        ],
        /// MSI clock range selection
        MSIRGSEL OFFSET(3) NUMBITS(1) [],
        /// MSI clock PLL enable
        MSIPLLEN OFFSET(2) NUMBITS(1) [],
        /// MSI clock ready flag
        MSIRDY OFFSET(1) NUMBITS(1) [],
        /// MSI clock enable
        MSION OFFSET(0) NUMBITS(1) []
    ],
    ICSCR [
        /// HSI clock trimming
        HSITRIM OFFSET(24) NUMBITS(7) [],
        /// HSI clock calibration
        HSICAL OFFSET(16) NUMBITS(8) [],
        /// MSI clock trimming
        MSITRIM OFFSET(8) NUMBITS(8) [],
        /// MSI clock calibration
        MSICAL OFFSET(0) NUMBITS(8) []
    ],
    CFGR [
        /// Microcontroller clock output prescaler
        MCOPRE OFFSET(28) NUMBITS(3) [
            Div1 = 0,
            Div2 = 1,
            Div4 = 2,
            Div8 = 3,
            Div16 = 4
        ],
        /// Microcontroller clock output
        MCOSEL OFFSET(24) NUMBITS(4) [
            NoClock = 0,
            SYSCLK = 1,
            MSI = 2,
            HSI16 = 3,
            HSE = 4,
            PLL = 5,
            LSI = 6,
            LSE = 7,
            HSI48 = 8
        ],
        /// Wakeup from Stop and CSS backup clock selection
        STOPWUCK OFFSET(15) NUMBITS(1) [
            MSI = 0,
            HSI16 = 1
        ],
        /// APB high-speed prescaler (APB2)
        PPRE2 OFFSET(11) NUMBITS(3) [
            Div1 = 0,
            Div2 = 4,
            Div4 = 5,
            Div8 = 6,
            Div16 = 7
        ],
        /// APB low-speed prescaler (APB1)
        PPRE1 OFFSET(8) NUMBITS(3) [
            Div1 = 0,
            Div2 = 4,
            Div4 = 5,
            Div8 = 6,
            Div16 = 7
        ],
        /// AHB prescaler
        HPRE OFFSET(4) NUMBITS(4) [
            Div1 = 0,
            Div2 = 8,
            Div4 = 9,
            Div8 = 10,
            Div16 = 11,
            Div64 = 12,
            Div128 = 13,
            Div256 = 14,
            Div512 = 15
        ],
        /// System clock switch status
        SWS OFFSET(2) NUMBITS(2) [
            MSI = 0,
            HSI16 = 1,
            HSE = 2,
            PLL = 3
        ],
        /// System clock switch
        SW OFFSET(0) NUMBITS(2) [
            MSI = 0,
            HSI16 = 1,
            HSE = 2,
            PLL = 3
        ]
    ],
    PLLCFGR [
        /// Main PLL division factor for PLLSAI2CLK
        PLLPDIV OFFSET(27) NUMBITS(5) [],
        /// Main PLL PLLCLK output enable
        PLLR OFFSET(25) NUMBITS(2) [
            Div2 = 0,
            Div4 = 1,
            Div6 = 2,
            Div8 = 3
        ],
        /// Main PLL PLLCLK output enable
        PLLREN OFFSET(24) NUMBITS(1) [],
        /// Main PLL division factor for PLLCLK (system clock)
        PLLQ OFFSET(21) NUMBITS(2) [
            Div2 = 0,
            Div4 = 1,
            Div6 = 2,
            Div8 = 3
        ],
        /// Main PLL PLLQCLK output enable
        PLLQEN OFFSET(20) NUMBITS(1) [],
        /// Main PLL division factor for PLLSAI3CLK (SAI1 and SAI2 clock)
        PLLP OFFSET(17) NUMBITS(1) [
            Div7 = 0,
            Div17 = 1
        ],
        /// Main PLL PLLSAI3CLK output enable
        PLLPEN OFFSET(16) NUMBITS(1) [],
        /// Main PLL multiplication factor for VCO
        PLLN OFFSET(8) NUMBITS(7) [],
        /// Division factor for the main PLL and audio PLL input clock
        PLLM OFFSET(4) NUMBITS(3) [
            Div1 = 0,
            Div2 = 1,
            Div3 = 2,
            Div4 = 3,
            Div5 = 4,
            Div6 = 5,
            Div7 = 6,
            Div8 = 7
        ],
        /// Main PLL, PLLSAI1 and PLLSAI2 entry clock source
        PLLSRC OFFSET(0) NUMBITS(2) [
            NoClock = 0,
            MSI = 1,
            HSI = 2,
            HSE = 3
        ]
    ],
    PLLSAI1CFGR [
        /// PLLSAI1 division factor for PLLADC1CLK (ADC clock)
        PLLSAI1R OFFSET(25) NUMBITS(2) [
            Div2 = 0,
            Div4 = 1,
            Div6 = 2,
            Div8 = 3
        ],
        /// PLLSAI1 PLLADC1CLK output enable
        PLLSAI1REN OFFSET(24) NUMBITS(1) [],
        /// SAI1PLL division factor for PLLUSB2CLK (48 MHz clock)
        PLLSAI1Q OFFSET(21) NUMBITS(2) [
            Div2 = 0,
            Div4 = 1,
            Div6 = 2,
            Div8 = 3
        ],
        /// SAI1PLL PLLUSB2CLK output enable
        PLLSAI1QEN OFFSET(20) NUMBITS(1) [],
        /// SAI1PLL division factor for PLLSAI1CLK (SAI1 or SAI2 clock)
        PLLSAI1P OFFSET(17) NUMBITS(1) [
            Div7 = 0,
            Div17 = 1
        ],
        /// SAI1PLL PLLSAI1CLK output enable
        PLLSAI1PEN OFFSET(16) NUMBITS(1) [],
        /// SAI1PLL multiplication factor for VCO
        PLLSAI1N OFFSET(8) NUMBITS(7) []
    ],
    PLLSAI2CFGR [
        /// PLLSAI2 division factor for PLLADC2CLK (ADC clock)
        PLLSAI2R OFFSET(25) NUMBITS(2) [
            Div2 = 0,
            Div4 = 1,
            Div6 = 2,
            Div8 = 3
        ],
        /// PLLSAI2 PLLADC2CLK output enable
        PLLSAI2REN OFFSET(24) NUMBITS(1) [],
        /// SAI2PLL division factor for PLLSAI2CLK (SAI1 or SAI2 clock)
        PLLSAI2P OFFSET(17) NUMBITS(1) [
            Div7 = 0,
            Div17 = 1
        ],
        /// SAI2PLL PLLSAI2CLK output enable
        PLLSAI2PEN OFFSET(16) NUMBITS(1) [],
        /// SAI2PLL multiplication factor for VCO
        PLLSAI2N OFFSET(8) NUMBITS(7) []
    ],
    CIER [
        /// LSE clock security system interrupt enable
        LSECSSIE OFFSET(9) NUMBITS(1) [],
        /// PLL SAI2 ready interrupt enable
        PLLSAI2RDYIE OFFSET(7) NUMBITS(1) [],
        /// PLL SAI1 ready interrupt enable
        PLLSAI1RDYIE OFFSET(6) NUMBITS(1) [],
        /// PLL ready interrupt enable
        PLLRDYIE OFFSET(5) NUMBITS(1) [],
        /// HSE ready interrupt enable
        HSERDYIE OFFSET(4) NUMBITS(1) [],
        /// HSI16 ready interrupt enable
        HSIRDYIE OFFSET(3) NUMBITS(1) [],
        /// MSI ready interrupt enable
        MSIRDYIE OFFSET(2) NUMBITS(1) [],
        /// LSE ready interrupt enable
        LSERDYIE OFFSET(1) NUMBITS(1) [],
        /// LSI ready interrupt enable
        LSIRDYIE OFFSET(0) NUMBITS(1) []
    ],
    CIFR [
        /// LSE Clock security system interrupt flag
        LSECSSF OFFSET(9) NUMBITS(1) [],
        /// Clock security system interrupt flag
        CSSF OFFSET(8) NUMBITS(1) [],
        /// PLLSAI2 ready interrupt flag
        PLLSAI2RDYF OFFSET(7) NUMBITS(1) [],
        /// PLLSAI1 ready interrupt flag
        PLLSAI1RDYF OFFSET(6) NUMBITS(1) [],
        /// PLL ready interrupt flag
        PLLRDYF OFFSET(5) NUMBITS(1) [],
        /// HSE ready interrupt flag
        HSERDYF OFFSET(4) NUMBITS(1) [],
        /// HSI16 ready interrupt flag
        HSIRDYF OFFSET(3) NUMBITS(1) [],
        /// MSI ready interrupt flag
        MSIRDYF OFFSET(2) NUMBITS(1) [],
        /// LSE ready interrupt flag
        LSERDYF OFFSET(1) NUMBITS(1) [],
        /// LSI ready interrupt flag
        LSIRDYF OFFSET(0) NUMBITS(1) []
    ],
    CICR [
        /// LSE Clock security system interrupt clear
        LSECSSC OFFSET(9) NUMBITS(1) [],
        /// Clock security system interrupt clear
        CSSC OFFSET(8) NUMBITS(1) [],
        /// PLLSAI2 ready interrupt clear
        PLLSAI2RDYC OFFSET(7) NUMBITS(1) [],
        /// PLLSAI1 ready interrupt clear
        PLLSAI1RDYC OFFSET(6) NUMBITS(1) [],
        /// PLL ready interrupt clear
        PLLRDYC OFFSET(5) NUMBITS(1) [],
        /// HSE ready interrupt clear
        HSERDYC OFFSET(4) NUMBITS(1) [],
        /// HSI16 ready interrupt clear
        HSIRDYC OFFSET(3) NUMBITS(1) [],
        /// MSI ready interrupt clear
        MSIRDYC OFFSET(2) NUMBITS(1) [],
        /// LSE ready interrupt clear
        LSERDYC OFFSET(1) NUMBITS(1) [],
        /// LSI ready interrupt clear
        LSIRDYC OFFSET(0) NUMBITS(1) []
    ],
    AHB1RSTR [
        /// Touch Sensing Controller reset
        TSCRST OFFSET(16) NUMBITS(1) [],
        /// CRC reset
        CRCRST OFFSET(12) NUMBITS(1) [],
        /// Flash memory interface reset
        FLASHRST OFFSET(8) NUMBITS(1) [],
        /// DMA2 reset
        DMA2RST OFFSET(1) NUMBITS(1) [],
        /// DMA1 reset
        DMA1RST OFFSET(0) NUMBITS(1) []
    ],
    AHB2RSTR [
        /// Random number generator reset
        RNGRST OFFSET(18) NUMBITS(1) [],
        /// AES hardware accelerator reset
        AESRST OFFSET(16) NUMBITS(1) [],
        /// ADC reset
        ADCRST OFFSET(13) NUMBITS(1) [],
        /// USB OTG FS reset
        OTGFSRST OFFSET(12) NUMBITS(1) [],
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
    AHB3RSTR [
        /// Quad SPI memory interface reset
        QSPIRST OFFSET(8) NUMBITS(1) [],
        /// Flexible memory controller reset
        FMCRST OFFSET(0) NUMBITS(1) []
    ],
    APB1RSTR1 [
        /// Low Power Timer 1 reset
        LPTIM1RST OFFSET(31) NUMBITS(1) [],
        /// OPAMP interface reset
        OPAMPRST OFFSET(30) NUMBITS(1) [],
        /// DAC1 interface reset
        DAC1RST OFFSET(29) NUMBITS(1) [],
        /// Power interface reset
        PWRRST OFFSET(28) NUMBITS(1) [],
        /// CAN1 reset
        CAN1RST OFFSET(25) NUMBITS(1) [],
        /// I2C3 reset
        I2C3RST OFFSET(23) NUMBITS(1) [],
        /// I2C2 reset
        I2C2RST OFFSET(22) NUMBITS(1) [],
        /// I2C1 reset
        I2C1RST OFFSET(21) NUMBITS(1) [],
        /// UART5 reset
        UART5RST OFFSET(20) NUMBITS(1) [],
        /// UART4 reset
        UART4RST OFFSET(19) NUMBITS(1) [],
        /// USART3 reset
        USART3RST OFFSET(18) NUMBITS(1) [],
        /// USART2 reset
        USART2RST OFFSET(17) NUMBITS(1) [],
        /// SPI3 reset
        SPI3RST OFFSET(15) NUMBITS(1) [],
        /// SPI2 reset
        SPI2RST OFFSET(14) NUMBITS(1) [],
        /// Window watchdog reset
        WWDGRST OFFSET(11) NUMBITS(1) [],
        /// TIM7 timer reset
        TIM7RST OFFSET(5) NUMBITS(1) [],
        /// TIM6 timer reset
        TIM6RST OFFSET(4) NUMBITS(1) [],
        /// TIM5 timer reset
        TIM5RST OFFSET(3) NUMBITS(1) [],
        /// TIM4 timer reset
        TIM4RST OFFSET(2) NUMBITS(1) [],
        /// TIM3 timer reset
        TIM3RST OFFSET(1) NUMBITS(1) [],
        /// TIM2 timer reset
        TIM2RST OFFSET(0) NUMBITS(1) []
    ],
    APB1RSTR2 [
        /// Low-power timer 2 reset
        LPTIM2RST OFFSET(5) NUMBITS(1) [],
        /// Single wire protocol reset
        SWPMI1RST OFFSET(2) NUMBITS(1) [],
        /// Low-power UART 1 reset
        LPUART1RST OFFSET(0) NUMBITS(1) []
    ],
    APB2RSTR [
        /// DFSDM filter reset
        DFSDMRST OFFSET(24) NUMBITS(1) [],
        /// Serial audio interface 2 (SAI2) reset
        SAI2RST OFFSET(22) NUMBITS(1) [],
        /// Serial audio interface 1 (SAI1) reset
        SAI1RST OFFSET(21) NUMBITS(1) [],
        /// TIM17 timer reset
        TIM17RST OFFSET(18) NUMBITS(1) [],
        /// TIM16 timer reset
        TIM16RST OFFSET(17) NUMBITS(1) [],
        /// TIM15 timer reset
        TIM15RST OFFSET(16) NUMBITS(1) [],
        /// USART1 reset
        USART1RST OFFSET(14) NUMBITS(1) [],
        /// TIM8 timer reset
        TIM8RST OFFSET(13) NUMBITS(1) [],
        /// SPI1 reset
        SPI1RST OFFSET(12) NUMBITS(1) [],
        /// TIM1 timer reset
        TIM1RST OFFSET(11) NUMBITS(1) [],
        /// SDMMC reset
        SDMMCRST OFFSET(10) NUMBITS(1) [],
        /// System configuration (SYSCFG) reset
        SYSCFGRST OFFSET(0) NUMBITS(1) []
    ],
    AHB1ENR [
        /// Touch Sensing Controller clock enable
        TSCEN OFFSET(16) NUMBITS(1) [],
        /// CRC clock enable
        CRCEN OFFSET(12) NUMBITS(1) [],
        /// Flash memory interface clock enable
        FLASHEN OFFSET(8) NUMBITS(1) [],
        /// DMA2 clock enable
        DMA2EN OFFSET(1) NUMBITS(1) [],
        /// DMA1 clock enable
        DMA1EN OFFSET(0) NUMBITS(1) []
    ],
    AHB2ENR [
        /// Random Number Generator clock enable
        RNGEN OFFSET(18) NUMBITS(1) [],
        /// AES accelerator clock enable
        AESEN OFFSET(16) NUMBITS(1) [],
        /// ADC clock enable
        ADCEN OFFSET(13) NUMBITS(1) [],
        /// OTG full speed clock enable
        OTGFSEN OFFSET(12) NUMBITS(1) [],
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
    AHB3ENR [
        /// QSPIEN
        QSPIEN OFFSET(8) NUMBITS(1) [],
        /// Flexible memory controller clock enable
        FMCEN OFFSET(0) NUMBITS(1) []
    ],
    APB1ENR1 [
        /// Low power timer 1 clock enable
        LPTIM1EN OFFSET(31) NUMBITS(1) [],
        /// OPAMP interface clock enable
        OPAMPEN OFFSET(30) NUMBITS(1) [],
        /// DAC1 interface clock enable
        DAC1EN OFFSET(29) NUMBITS(1) [],
        /// Power interface clock enable
        PWREN OFFSET(28) NUMBITS(1) [],
        /// CAN1 clock enable
        CAN1EN OFFSET(25) NUMBITS(1) [],
        /// I2C3 clock enable
        I2C3EN OFFSET(23) NUMBITS(1) [],
        /// I2C2 clock enable
        I2C2EN OFFSET(22) NUMBITS(1) [],
        /// I2C1 clock enable
        I2C1EN OFFSET(21) NUMBITS(1) [],
        /// UART5 clock enable
        UART5EN OFFSET(20) NUMBITS(1) [],
        /// UART4 clock enable
        UART4EN OFFSET(19) NUMBITS(1) [],
        /// USART3 clock enable
        USART3EN OFFSET(18) NUMBITS(1) [],
        /// USART2 clock enable
        USART2EN OFFSET(17) NUMBITS(1) [],
        /// SPI3 clock enable
        SPI3EN OFFSET(15) NUMBITS(1) [],
        /// SPI2 clock enable
        SPI2EN OFFSET(14) NUMBITS(1) [],
        /// Window watchdog clock enable
        WWDGEN OFFSET(11) NUMBITS(1) [],
        /// RTC APB clock enable
        RTCAPBEN OFFSET(10) NUMBITS(1) [],
        /// LCD clock enable
        LCDEN OFFSET(9) NUMBITS(1) [],
        /// TIM7 timer clock enable
        TIM7EN OFFSET(5) NUMBITS(1) [],
        /// TIM6 timer clock enable
        TIM6EN OFFSET(4) NUMBITS(1) [],
        /// TIM5 timer clock enable
        TIM5EN OFFSET(3) NUMBITS(1) [],
        /// TIM4 timer clock enable
        TIM4EN OFFSET(2) NUMBITS(1) [],
        /// TIM3 timer clock enable
        TIM3EN OFFSET(1) NUMBITS(1) [],
        /// TIM2 timer clock enable
        TIM2EN OFFSET(0) NUMBITS(1) []
    ],
    APB1ENR2 [
        /// LPTIM2EN
        LPTIM2EN OFFSET(5) NUMBITS(1) [],
        /// Single wire protocol clock enable
        SWPMI1EN OFFSET(2) NUMBITS(1) [],
        /// Low power UART 1 clock enable
        LPUART1EN OFFSET(0) NUMBITS(1) []
    ],
    APB2ENR [
        /// DFSDM timer clock enable
        DFSDMEN OFFSET(24) NUMBITS(1) [],
        /// SAI2 clock enable
        SAI2EN OFFSET(22) NUMBITS(1) [],
        /// SAI1 clock enable
        SAI1EN OFFSET(21) NUMBITS(1) [],
        /// TIM17 timer clock enable
        TIM17EN OFFSET(18) NUMBITS(1) [],
        /// TIM16 timer clock enable
        TIM16EN OFFSET(17) NUMBITS(1) [],
        /// TIM15 timer clock enable
        TIM15EN OFFSET(16) NUMBITS(1) [],
        /// USART1clock enable
        USART1EN OFFSET(14) NUMBITS(1) [],
        /// TIM8 timer clock enable
        TIM8EN OFFSET(13) NUMBITS(1) [],
        /// SPI1 clock enable
        SPI1EN OFFSET(12) NUMBITS(1) [],
        /// TIM1 timer clock enable
        TIM1EN OFFSET(11) NUMBITS(1) [],
        /// SDMMC clock enable
        SDMMCEN OFFSET(10) NUMBITS(1) [],
        /// Firewall clock enable
        FWEN OFFSET(7) NUMBITS(1) [],
        /// SYSCFG clock enable
        SYSCFGEN OFFSET(0) NUMBITS(1) []
    ],
    AHB1SMENR [
        /// Touch Sensing Controller clocks enable during Sleep and Stop modes
        TSCSMEN OFFSET(16) NUMBITS(1) [],
        /// CRCSMEN
        CRCSMEN OFFSET(12) NUMBITS(1) [],
        /// SRAM1 interface clocks enable during Sleep and Stop modes
        SRAM1SMEN OFFSET(9) NUMBITS(1) [],
        /// Flash memory interface clocks enable during Sleep and Stop modes
        FLASHSMEN OFFSET(8) NUMBITS(1) [],
        /// DMA2 clocks enable during Sleep and Stop modes
        DMA2SMEN OFFSET(1) NUMBITS(1) [],
        /// DMA1 clocks enable during Sleep and Stop modes
        DMA1SMEN OFFSET(0) NUMBITS(1) []
    ],
    AHB2SMENR [
        /// Random Number Generator clocks enable during Sleep and Stop modes
        RNGSMEN OFFSET(18) NUMBITS(1) [],
        /// AES accelerator clocks enable during Sleep and Stop modes
        AESSMEN OFFSET(16) NUMBITS(1) [],
        /// ADC clocks enable during Sleep and Stop modes
        ADCSMEN OFFSET(13) NUMBITS(1) [],
        /// OTG full speed clocks enable during Sleep and Stop modes
        OTGFSSMEN OFFSET(12) NUMBITS(1) [],
        /// SRAM2 interface clocks enable during Sleep and Stop modes
        SRAM2SMEN OFFSET(9) NUMBITS(1) [],
        /// IO port H clocks enable during Sleep and Stop modes
        GPIOHSMEN OFFSET(7) NUMBITS(1) [],
        /// IO port G clocks enable during Sleep and Stop modes
        GPIOGSMEN OFFSET(6) NUMBITS(1) [],
        /// IO port F clocks enable during Sleep and Stop modes
        GPIOFSMEN OFFSET(5) NUMBITS(1) [],
        /// IO port E clocks enable during Sleep and Stop modes
        GPIOESMEN OFFSET(4) NUMBITS(1) [],
        /// IO port D clocks enable during Sleep and Stop modes
        GPIODSMEN OFFSET(3) NUMBITS(1) [],
        /// IO port C clocks enable during Sleep and Stop modes
        GPIOCSMEN OFFSET(2) NUMBITS(1) [],
        /// IO port B clocks enable during Sleep and Stop modes
        GPIOBSMEN OFFSET(1) NUMBITS(1) [],
        /// IO port A clocks enable during Sleep and Stop modes
        GPIOASMEN OFFSET(0) NUMBITS(1) []
    ],
    AHB3SMENR [
        /// QSPISMEN
        QSPISMEN OFFSET(8) NUMBITS(1) [],
        /// Flexible memory controller clocks enable during Sleep and Stop modes
        FMCSMEN OFFSET(0) NUMBITS(1) []
    ],
    APB1SMENR1 [
        /// Low power timer 1 clocks enable during Sleep and Stop modes
        LPTIM1SMEN OFFSET(31) NUMBITS(1) [],
        /// OPAMP interface clocks enable during Sleep and Stop modes
        OPAMPSMEN OFFSET(30) NUMBITS(1) [],
        /// DAC1 interface clocks enable during Sleep and Stop modes
        DAC1SMEN OFFSET(29) NUMBITS(1) [],
        /// Power interface clocks enable during Sleep and Stop modes
        PWRSMEN OFFSET(28) NUMBITS(1) [],
        /// CAN1 clocks enable during Sleep and Stop modes
        CAN1SMEN OFFSET(25) NUMBITS(1) [],
        /// I2C3 clocks enable during Sleep and Stop modes
        I2C3SMEN OFFSET(23) NUMBITS(1) [],
        /// I2C2 clocks enable during Sleep and Stop modes
        I2C2SMEN OFFSET(22) NUMBITS(1) [],
        /// I2C1 clocks enable during Sleep and Stop modes
        I2C1SMEN OFFSET(21) NUMBITS(1) [],
        /// UART5 clocks enable during Sleep and Stop modes
        UART5SMEN OFFSET(20) NUMBITS(1) [],
        /// UART4 clocks enable during Sleep and Stop modes
        UART4SMEN OFFSET(19) NUMBITS(1) [],
        /// USART3 clocks enable during Sleep and Stop modes
        USART3SMEN OFFSET(18) NUMBITS(1) [],
        /// USART2 clocks enable during Sleep and Stop modes
        USART2SMEN OFFSET(17) NUMBITS(1) [],
        /// SPI3 clocks enable during Sleep and Stop modes
        SPI3SMEN OFFSET(15) NUMBITS(1) [],
        /// SPI2 clocks enable during Sleep and Stop modes
        SPI2SMEN OFFSET(14) NUMBITS(1) [],
        /// Window watchdog clocks enable during Sleep and Stop modes
        WWDGSMEN OFFSET(11) NUMBITS(1) [],
        /// RTC APB clock enable during Sleep and Stop modes
        RTCAPBSMEN OFFSET(10) NUMBITS(1) [],
        /// LCD clocks enable during Sleep and Stop modes
        LCDSMEN OFFSET(9) NUMBITS(1) [],
        /// TIM7 timer clocks enable during Sleep and Stop modes
        TIM7SMEN OFFSET(5) NUMBITS(1) [],
        /// TIM6 timer clocks enable during Sleep and Stop modes
        TIM6SMEN OFFSET(4) NUMBITS(1) [],
        /// TIM5 timer clocks enable during Sleep and Stop modes
        TIM5SMEN OFFSET(3) NUMBITS(1) [],
        /// TIM4 timer clocks enable during Sleep and Stop modes
        TIM4SMEN OFFSET(2) NUMBITS(1) [],
        /// TIM3 timer clocks enable during Sleep and Stop modes
        TIM3SMEN OFFSET(1) NUMBITS(1) [],
        /// TIM2 timer clocks enable during Sleep and Stop modes
        TIM2SMEN OFFSET(0) NUMBITS(1) []
    ],
    APB1SMENR2 [
        /// LPTIM2SMEN
        LPTIM2SMEN OFFSET(5) NUMBITS(1) [],
        /// Single wire protocol clocks enable during Sleep and Stop modes
        SWPMI1SMEN OFFSET(2) NUMBITS(1) [],
        /// Low power UART 1 clocks enable during Sleep and Stop modes
        LPUART1SMEN OFFSET(0) NUMBITS(1) []
    ],
    APB2SMENR [
        /// DFSDM timer clocks enable during Sleep and Stop modes
        DFSDMSMEN OFFSET(24) NUMBITS(1) [],
        /// SAI2 clocks enable during Sleep and Stop modes
        SAI2SMEN OFFSET(22) NUMBITS(1) [],
        /// SAI1 clocks enable during Sleep and Stop modes
        SAI1SMEN OFFSET(21) NUMBITS(1) [],
        /// TIM17 timer clocks enable during Sleep and Stop modes
        TIM17SMEN OFFSET(18) NUMBITS(1) [],
        /// TIM16 timer clocks enable during Sleep and Stop modes
        TIM16SMEN OFFSET(17) NUMBITS(1) [],
        /// TIM15 timer clocks enable during Sleep and Stop modes
        TIM15SMEN OFFSET(16) NUMBITS(1) [],
        /// USART1clocks enable during Sleep and Stop modes
        USART1SMEN OFFSET(14) NUMBITS(1) [],
        /// TIM8 timer clocks enable during Sleep and Stop modes
        TIM8SMEN OFFSET(13) NUMBITS(1) [],
        /// SPI1 clocks enable during Sleep and Stop modes
        SPI1SMEN OFFSET(12) NUMBITS(1) [],
        /// TIM1 timer clocks enable during Sleep and Stop modes
        TIM1SMEN OFFSET(11) NUMBITS(1) [],
        /// SDMMC clocks enable during Sleep and Stop modes
        SDMMCSMEN OFFSET(10) NUMBITS(1) [],
        /// SYSCFG clocks enable during Sleep and Stop modes
        SYSCFGSMEN OFFSET(0) NUMBITS(1) []
    ],
    CCIPR [
        /// DFSDM clock source selection
        DFSDMSEL OFFSET(31) NUMBITS(1) [
            PCLK = 0,
            SYSCLK = 1
        ],
        /// SWPMI1 clock source selection
        SWPMI1SEL OFFSET(30) NUMBITS(1) [
            PCLK = 0,
            HSI16 = 1
        ],
        /// ADCs clock source selection
        ADCSEL OFFSET(28) NUMBITS(2) [
            NoClock = 0,
            PLLSAI1 = 1,
            PLLSAI2 = 2,
            SYSCLK = 3
        ],
        /// 48 MHz clock source selection
        CLK48SEL OFFSET(26) NUMBITS(2) [
            NoClock = 0,
            PLLSAI1 = 1,
            PLL = 2,
            MSI = 3
        ],
        /// SAI2 clock source selection
        SAI2SEL OFFSET(24) NUMBITS(2) [
            PLLSAI1 = 0,
            PLLSAI2 = 1,
            PLL = 2,
            EXTCLK = 3
        ],
        /// SAI1 clock source selection
        SAI1SEL OFFSET(22) NUMBITS(2) [
            PLLSAI1 = 0,
            PLLSAI2 = 1,
            PLL = 2,
            EXTCLK = 3
        ],
        /// Low power timer 2 clock source selection
        LPTIM2SEL OFFSET(20) NUMBITS(2) [
            PCLK = 0,
            LSI = 1,
            HSI16 = 2,
            LSE = 3
        ],
        /// Low power timer 1 clock source selection
        LPTIM1SEL OFFSET(18) NUMBITS(2) [
            PCLK = 0,
            LSI = 1,
            HSI16 = 2,
            LSE = 3
        ],
        /// I2C3 clock source selection
        I2C3SEL OFFSET(16) NUMBITS(2) [
            PCLK = 0,
            SYSCLK = 1,
            HSI16 = 2
        ],
        /// I2C2 clock source selection
        I2C2SEL OFFSET(14) NUMBITS(2) [
            PCLK = 0,
            SYSCLK = 1,
            HSI16 = 2
        ],
        /// I2C1 clock source selection
        I2C1SEL OFFSET(12) NUMBITS(2) [
            PCLK = 0,
            SYSCLK = 1,
            HSI16 = 2
        ],
        /// LPUART1 clock source selection
        LPUART1SEL OFFSET(10) NUMBITS(2) [
            PCLK = 0,
            SYSCLK = 1,
            HSI16 = 2,
            LSE = 3
        ],
        /// UART5 clock source selection
        UART5SEL OFFSET(8) NUMBITS(2) [
            PCLK = 0,
            SYSCLK = 1,
            HSI16 = 2,
            LSE = 3
        ],
        /// UART4 clock source selection
        UART4SEL OFFSET(6) NUMBITS(2) [
            PCLK = 0,
            SYSCLK = 1,
            HSI16 = 2,
            LSE = 3
        ],
        /// USART3 clock source selection
        USART3SEL OFFSET(4) NUMBITS(2) [
            PCLK = 0,
            SYSCLK = 1,
            HSI16 = 2,
            LSE = 3
        ],
        /// USART2 clock source selection
        USART2SEL OFFSET(2) NUMBITS(2) [
            PCLK = 0,
            SYSCLK = 1,
            HSI16 = 2,
            LSE = 3
        ],
        /// USART1 clock source selection
        USART1SEL OFFSET(0) NUMBITS(2) [
            PCLK = 0,
            SYSCLK = 1,
            HSI16 = 2,
            LSE = 3
        ]
    ],
    BDCR [
        /// Low speed clock output selection
        LSCOSEL OFFSET(25) NUMBITS(1) [
            LSI = 0,
            LSE = 1
        ],
        /// Low speed clock output enable
        LSCOEN OFFSET(24) NUMBITS(1) [],
        /// Backup domain software reset
        BDRST OFFSET(16) NUMBITS(1) [],
        /// RTC clock enable
        RTCEN OFFSET(15) NUMBITS(1) [],
        /// RTC clock source selection
        RTCSEL OFFSET(8) NUMBITS(2) [
            NoClock = 0,
            LSE = 1,
            LSI = 2,
            HSE = 3
        ],
        /// CSS on LSE enable
        LSECSSON OFFSET(5) NUMBITS(1) [],
        /// CSS on LSE failure detection
        LSECSSD OFFSET(6) NUMBITS(1) [],
        /// SE oscillator drive capability
        LSEDRV OFFSET(3) NUMBITS(2) [
            Low = 0,
            MediumLow = 1,
            MediumHigh = 2,
            High = 3
        ],
        /// LSE oscillator bypass
        LSEBYP OFFSET(2) NUMBITS(1) [],
        /// LSE oscillator ready
        LSERDY OFFSET(1) NUMBITS(1) [],
        /// LSE oscillator enable
        LSEON OFFSET(0) NUMBITS(1) []
    ],
    CSR [
        /// Low-power reset flag
        LPWRRSTF OFFSET(31) NUMBITS(1) [],
        /// Window watchdog reset flag
        WWDGRSTF OFFSET(30) NUMBITS(1) [],
        /// Independent window watchdog reset flag
        IWDGRSTF OFFSET(29) NUMBITS(1) [],
        /// Software reset flag
        SFTRSTF OFFSET(28) NUMBITS(1) [],
        /// BOR flag
        BORRSTF OFFSET(27) NUMBITS(1) [],
        /// Pin reset flag
        PINRSTF OFFSET(26) NUMBITS(1) [],
        /// Option byte loader reset flag
        OBLRSTF OFFSET(25) NUMBITS(1) [],
        /// Firewall reset flag
        FIREWALLRSTF OFFSET(24) NUMBITS(1) [],
        /// Remove reset flag
        RMVF OFFSET(23) NUMBITS(1) [],
        /// SI1 oscillator ready
        LSIRDY OFFSET(1) NUMBITS(1) [],
        /// LSI oscillator enable
        LSION OFFSET(0) NUMBITS(1) []
    ]
];

const RCC_BASE: StaticRef<RccRegisters> =
    unsafe { StaticRef::new(0x40021000 as *const RccRegisters) };

pub(crate) const DEFAULT_PLLM_VALUE: PLLM = PLLM::DivideBy1;
pub(crate) const DEFAULT_PLLN_VALUE: usize = 0x28; // 40
pub(crate) const DEFAULT_PLLR_VALUE: PLLR = PLLR::DivideBy2;

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

    // Init the PLL clock.
    fn init_pll_clock(&self) {
        self.set_pll_clock_source(PllSource::MSI);
        self.set_pll_clock_m_divider(DEFAULT_PLLM_VALUE);
        self.set_pll_clock_n_multiplier(DEFAULT_PLLN_VALUE);
        self.set_pll_clock_r_divider(DEFAULT_PLLR_VALUE);
    }

    // Get the current system clock source
    pub(crate) fn get_sys_clock_source(&self) -> SysClockSource {
        match self.registers.cfgr.read(CFGR::SWS) {
            0b00 => SysClockSource::MSI,
            0b01 => SysClockSource::HSI,
            0b11 => SysClockSource::PLL,
            _ => todo!(),
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
            || system_clock_source == SysClockSource::PLL
                && self.registers.pllcfgr.read(PLLCFGR::PLLSRC) == PllSource::MSI as u32
    }

    pub(crate) fn is_hsi_clock_system_clock(&self) -> bool {
        let system_clock_source = self.get_sys_clock_source();
        system_clock_source == SysClockSource::HSI
            || system_clock_source == SysClockSource::PLL
                && self.registers.pllcfgr.read(PLLCFGR::PLLSRC) == PllSource::HSI as u32
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

    /* Main PLL clock*/

    // The main PLL clock must not be configured as the system clock
    // when you want disable pll clock.
    //First you need set sysclk source for other like MSI
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

    pub(crate) fn get_pll_clock_source(&self) -> PllSource {
        match self.registers.pllcfgr.read(PLLCFGR::PLLSRC) {
            0b00 => PllSource::NoClock,
            0b01 => PllSource::MSI,
            0b10 => PllSource::HSI,
            _ => todo!("Unexpected PllSource!"),
        }
    }

    // This method must be called only when all PLL clocks are disabled
    pub(crate) fn set_pll_clock_source(&self, source: PllSource) {
        self.registers
            .pllcfgr
            .modify(PLLCFGR::PLLSRC.val(source as u32));
    }

    // This method must be called only when all PLL clocks are disabled
    pub(crate) fn set_pll_clock_m_divider(&self, m: PLLM) {
        self.registers.pllcfgr.modify(PLLCFGR::PLLM.val(m as u32));
    }

    // This method must be called only if the main PLL clock is disabled
    pub(crate) fn set_pll_clock_n_multiplier(&self, n: usize) {
        self.registers.pllcfgr.modify(PLLCFGR::PLLN.val(n as u32));
    }

    // This method must be called only if the main PLL clock is disabled
    pub(crate) fn set_pll_clock_r_divider(&self, n: PLLR) {
        self.registers.pllcfgr.modify(PLLCFGR::PLLR.val(n as u32));
    }

    /* MSI clock */

    pub(crate) fn enable_msi_clock(&self) {
        self.registers.cr.modify(CR::MSION::SET);
    }

    pub(crate) fn disable_msi_clock(&self) {
        self.registers.cr.modify(CR::MSION::CLEAR);
    }

    pub(crate) fn is_enabled_msi_clock(&self) -> bool {
        self.registers.cr.is_set(CR::MSION)
    }

    // Indicates whether the MSI oscillator is stable
    pub(crate) fn is_ready_msi_clock(&self) -> bool {
        self.registers.cr.is_set(CR::MSIRDY)
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
        self.registers.ahb2enr.is_set(AHB2ENR::GPIOHEN)
    }

    pub(crate) fn enable_gpioh_clock(&self) {
        self.registers.ahb2enr.modify(AHB2ENR::GPIOHEN::SET)
    }

    pub(crate) fn disable_gpioh_clock(&self) {
        self.registers.ahb2enr.modify(AHB2ENR::GPIOHEN::CLEAR)
    }

    // GPIOG clock

    pub(crate) fn is_enabled_gpiog_clock(&self) -> bool {
        self.registers.ahb2enr.is_set(AHB2ENR::GPIOGEN)
    }

    pub(crate) fn enable_gpiog_clock(&self) {
        self.registers.ahb2enr.modify(AHB2ENR::GPIOGEN::SET)
    }

    pub(crate) fn disable_gpiog_clock(&self) {
        self.registers.ahb2enr.modify(AHB2ENR::GPIOGEN::CLEAR)
    }

    // GPIOF clock

    pub(crate) fn is_enabled_gpiof_clock(&self) -> bool {
        self.registers.ahb2enr.is_set(AHB2ENR::GPIOFEN)
    }

    pub(crate) fn enable_gpiof_clock(&self) {
        self.registers.ahb2enr.modify(AHB2ENR::GPIOFEN::SET)
    }

    pub(crate) fn disable_gpiof_clock(&self) {
        self.registers.ahb2enr.modify(AHB2ENR::GPIOFEN::CLEAR)
    }

    // GPIOE clock

    pub(crate) fn is_enabled_gpioe_clock(&self) -> bool {
        self.registers.ahb2enr.is_set(AHB2ENR::GPIOEEN)
    }

    pub(crate) fn enable_gpioe_clock(&self) {
        self.registers.ahb2enr.modify(AHB2ENR::GPIOEEN::SET)
    }

    pub(crate) fn disable_gpioe_clock(&self) {
        self.registers.ahb2enr.modify(AHB2ENR::GPIOEEN::CLEAR)
    }

    // GPIOD clock

    pub(crate) fn is_enabled_gpiod_clock(&self) -> bool {
        self.registers.ahb2enr.is_set(AHB2ENR::GPIODEN)
    }

    pub(crate) fn enable_gpiod_clock(&self) {
        self.registers.ahb2enr.modify(AHB2ENR::GPIODEN::SET)
    }

    pub(crate) fn disable_gpiod_clock(&self) {
        self.registers.ahb2enr.modify(AHB2ENR::GPIODEN::CLEAR)
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

    // USART3 clock

    pub(crate) fn is_enabled_usart3_clock(&self) -> bool {
        self.registers.apb1enr1.is_set(APB1ENR1::USART3EN)
    }

    pub(crate) fn enable_usart3_clock(&self) {
        self.registers.apb1enr1.modify(APB1ENR1::USART3EN::SET)
    }

    pub(crate) fn disable_usart3_clock(&self) {
        self.registers.apb1enr1.modify(APB1ENR1::USART3EN::CLEAR)
    }
}

pub(crate) enum PLLM {
    DivideBy1 = 0,
    DivideBy2 = 1,
    DivideBy3 = 2,
    DivideBy4 = 3,
    DivideBy5 = 4,
    DivideBy6 = 5,
    DivideBy7 = 6,
    DivideBy8 = 7,
}

impl From<usize> for PLLM {
    fn from(item: usize) -> PLLM {
        match item {
            1 => PLLM::DivideBy1,
            2 => PLLM::DivideBy2,
            3 => PLLM::DivideBy3,
            4 => PLLM::DivideBy4,
            5 => PLLM::DivideBy5,
            6 => PLLM::DivideBy6,
            7 => PLLM::DivideBy7,
            8 => PLLM::DivideBy8,
            _ => todo!(),
        }
    }
}

impl From<PLLM> for usize {
    fn from(item: PLLM) -> usize {
        match item {
            PLLM::DivideBy1 => 1,
            PLLM::DivideBy2 => 2,
            PLLM::DivideBy3 => 3,
            PLLM::DivideBy4 => 4,
            PLLM::DivideBy5 => 5,
            PLLM::DivideBy6 => 6,
            PLLM::DivideBy7 => 7,
            PLLM::DivideBy8 => 8,
        }
    }
}

pub(crate) enum PLLR {
    DivideBy2 = 0,
    DivideBy4 = 1,
    DivideBy6 = 2,
    DivideBy8 = 3,
}

impl From<usize> for PLLR {
    fn from(item: usize) -> PLLR {
        match item {
            2 => PLLR::DivideBy2,
            4 => PLLR::DivideBy4,
            6 => PLLR::DivideBy6,
            8 => PLLR::DivideBy8,
            _ => todo!(),
        }
    }
}

impl From<PLLR> for usize {
    fn from(item: PLLR) -> usize {
        match item {
            PLLR::DivideBy2 => 2,
            PLLR::DivideBy4 => 4,
            PLLR::DivideBy6 => 6,
            PLLR::DivideBy8 => 8,
        }
    }
}

/// Clock sources for the CPU
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SysClockSource {
    MSI = 0b00,
    HSI = 0b01,
    //HSE = 0b10, Uncomment this when support for HSE is added
    PLL = 0b11,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum PllSource {
    NoClock = 0b00,
    MSI = 0b01,
    HSI = 0b10,
    // HSE = 0b11, Uncomment this when support for HSE is added
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

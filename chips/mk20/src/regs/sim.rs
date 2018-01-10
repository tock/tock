use common::regs::{ReadWrite, ReadOnly};

#[repr(C, packed)]
pub struct Registers {
    pub sopt2: ReadWrite<u32>,
    _reserved0: ReadWrite<u32>,
    pub sopt4: ReadWrite<u32>,
    pub sopt5: ReadWrite<u32>,
    _reserved1: ReadWrite<u32>,
    pub sopt7: ReadWrite<u32>,
    pub sopt8: ReadWrite<u32>,
    pub sopt9: ReadWrite<u32>,
    pub sdid: ReadOnly<u32>,
    pub scgc1: ReadWrite<u32, SystemClockGatingControl1>,
    pub scgc2: ReadWrite<u32, SystemClockGatingControl2>,
    pub scgc3: ReadWrite<u32, SystemClockGatingControl3>,
    pub scgc4: ReadWrite<u32, SystemClockGatingControl4>,
    pub scgc5: ReadWrite<u32, SystemClockGatingControl5>,
    pub scgc6: ReadWrite<u32, SystemClockGatingControl6>,
    pub scgc7: ReadWrite<u32, SystemClockGatingControl7>,
    pub clkdiv1: ReadWrite<u32, ClockDivider1>,
    pub clkdiv2: ReadWrite<u32>,
    pub fcfg1: ReadOnly<u32>,
    pub fcfg2: ReadOnly<u32>,
    pub uidh: ReadOnly<u32>,
    pub uidmh: ReadOnly<u32>,
    pub uidml: ReadOnly<u32>,
    pub uidl: ReadOnly<u32>,
    pub clkdiv3: ReadWrite<u32>,
    pub clkdiv4: ReadWrite<u32>,
}

pub const SIM: *mut Registers = 0x40048004 as *mut Registers;

bitfields![u32,
    SCGC1 SystemClockGatingControl1 [
        UART4 10,
        I2C3 7,
        I2C2 6
    ],
    SCGC2 SystemClockGatingControl2 [
        DAC1 13,
        DAC0 12,
        TPM2 10,
        TPM1 9,
        LPUART0 4,
        ENET 0
    ],
    SCGC3 SystemClockGatingControl3 [
        ADC1 27,
        FTM3 25,
        FTM2 24,
        SDHC 17,
        SPI2 12,
        FLEXCAN1 4,
        USBHSDCD 3,
        USBHSPHY 2,
        USBHS 1,
        RNGA 0
    ],
    SCGC4 SystemClockGatingControl4 [
        VREF 20,
        CMP 19,
        USBOTG 18,
        UART3 13,
        UART2 12,
        UART1 11,
        UART0 10,
        I2C1 7,
        I2C0 6,
        CMT 2,
        EWM 1
    ],
    SCGC5 SystemClockGatingControl5 [
        PORT (9, Mask(0b11111)) [
            All = 0b11111,
            A = 0b1,
            B = 0b10,
            C = 0b100,
            D = 0b1000,
            E = 0b10000
        ],
        TSI 5 [],
        LPTMR 0 []
    ],
    SCGC6 SystemClockGatingControl6 [
        DAC0 0,
        RTC 29,
        ADC0 27,
        FTM2 26,
        FTM1 25,
        FTM0 24,
        PIT 23,
        PDB 22,
        USBDCD 21,
        CRC 18,
        I2S 15,
        SPI1 13,
        SPI0 12,
        RNGA 9,
        FLEXCAN0 4,
        DMAMUX 1,
        FTF 0
    ],
    SCGC7 SystemClockGatingControl7 [
        SDRAMC 3,
        MPU 2,
        DMA 1,
        FLEXBUS 0
    ],
    CLKDIV1 ClockDivider1 [
        Core (28, Mask(0b1111)) [],
        Bus (24, Mask(0b1111)) [],
        FlexBus (20, Mask(0b1111)) [],
        Flash (16, Mask(0b1111)) []
    ]
];

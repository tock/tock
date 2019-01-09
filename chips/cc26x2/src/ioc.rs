use crate::gpio;
use kernel::common::registers::{register_bitfields, ReadWrite};

pub const MIN_ANALOG_CAPABLE: usize = 23 + 1;
pub const MAX_ANALOG_CAPABLE: usize = 27;

#[repr(C)]
pub struct Registers {
    pub cfg: [ReadWrite<u32, Config::Register>; gpio::NUM_PINS],
}

register_bitfields![
    u32,
    Config[
        HYST_EN     OFFSET(30) NUMBITS(1) [],
        INPUT_EN OFFSET(29) NUMBITS(1) [],
        IO_MODE     OFFSET(24) NUMBITS(3) [
            Normal = 0x0,
            Inverted = 0x1,
            OpenDrain = 0x4,
            OpenDrainInverted = 0x5,
            OpenSource = 0x6,
            OpenSourceInverted = 0x7
        ],
        WAKEUP_CFG OFFSET(27) NUMBITS (2) [
            WakeupGoingLow = 0b10,
            WakeupGoingHigh = 0b00
        ],
        EDGE_IRQ_EN OFFSET(18) NUMBITS(1) [], // Interrupt enable
        EDGE_DET    OFFSET(16) NUMBITS(2) [
            None        = 0x0,
            FallingEdge = 0x1,
            RisingEdge  = 0x2,
            BothEdges   = 0x3
        ],
        PULL    OFFSET(13) NUMBITS(2) [
            Down = 0x1,
            Up   = 0x2,
            None = 0x3
        ],
        SLEW_RED  OFFSET(12) NUMBITS(1) [], // Reduce slew rate
        CURRENT_MODE  OFFSET(10) NUMBITS(2) [
            Low      = 0,            // 2mA
            High     = 0x01,        // 4mA
            Extended = 0x02     // 8mA for double drive strength IOs
        ],
        DRIVE_STRENGTH  OFFSET(8) NUMBITS(2) [ // only applies if CURRENT_MODE = Low
            Auto = 0,           // controlled by AON BATMON
            Min  = 0x01,         // controlled by AON_IOC:IOSTRMIN
            Med  = 0x02,         // controlled by AON_IOC:IOSTRMED
            Max  = 0x3           // controlled by AON_IOC:IOSTRMAX
        ],
        IOEV_RTC_EN OFFSET(7) NUMBITS(1) [],   // Event asserted by this IO when edge detection is enabled
        IOEV_MCU_WU_EN OFFSET(6) NUMBITS(1) [], // Event asserted by this IO when edge detection is enabled
        PORT_ID     OFFSET(0) NUMBITS(6) [
            // From p.1072
            GPIO = 0,
            AON_CLK32K = 7,
            AUX_DOMAIN_IO = 8,
            SSI0_RX = 9,
            SSI0_TX = 10,
            SSI0_FSS = 11,
            SSI0_CLK = 12,
            I2C_MSSDA = 13,
            I2C_MSSCL = 14,
            UART0_RX = 15,
            UART0_TX = 16,
            UART0_CTS = 17,
            UART0_RTS = 18,
            UART1_RX = 19,
            UART1_TX = 20,
            UART1_CTS = 21,
            UART1_RTS = 22,
            PORT_EVENT0 = 23,
            PORT_EVENT1 = 24,
            PORT_EVENT2 = 25,
            PORT_EVENT3 = 26,
            PORT_EVENT4 = 27,
            PORT_EVENT5 = 28,
            PORT_EVENT6 = 29,
            PORT_EVENT7 = 30,
            CPU_SWV = 32,
            SSI1_RX = 33,
            SSI1_TX = 34,
            SSI1_FSS = 35,
            SSI1_CLK = 36,
            I2S_AD0 = 37,
            I2S_AD1 = 38,
            I2S_WCLK = 39,
            I2S_BCLK = 40,
            I2S_MCLK = 41,
            RFC_GPO0 = 47,
            RFC_GPO1 = 48,
            RFC_GPO2 = 49,
            RFC_GPO3 = 50
        ]
    ]
];

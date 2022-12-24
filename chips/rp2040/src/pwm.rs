//! PWM driver for RP2040.

//use kernel::hil;
use kernel::utilities::registers::{register_bitfields, ReadWrite, ReadOnly, WriteOnly};

register_bitfields![u32,
    CSR [
        EN OFFSET(0) NUMBITS(1) [], /// Enable PWM channel
        PH_CORRECT OFFSET(1) NUMBITS(1) [], /// Enable phase-correct modulation
        A_INV OFFSET(2) NUMBITS(1) [], /// Invert output A
        B_INV OFFSET(3) NUMBITS(1) [], /// Invert output B
        /// PWM slice event selection for fractional clock divider
        /// Default value = FREE_RUNNING (always on)
        /// If the event is different from FREE_RUNNING, then pin B becomes
        /// an input pin
        DIVMOD OFFSET(4) NUMBITS(2) [
            FREE_RUNNING = 0, /// Free-running counting at rate dictated by fractional divider
            B_HIGH = 1, /// Fractional divider operation is gated by the PWM B pin
            B_RISING = 2, /// Counter advances with each rising edge of the PWM B pin
            B_FALLING = 3 /// Counter advances with each falling edge of the PWM B pin
        ],
        /// Retard the phase of the counter by 1 count, while it is running
        /// Self-clearing. Write a 1, and poll until low. Counter must be running.
        PH_RET OFFSET(6) NUMBITS(1) [],
        /// Advance the phase of the counter by 1 count, while it is running
        /// Self clearing. Write a 1, and poll until low. Counter must be running.
        PH_ADV OFFSET(7) NUMBITS(1) []
    ],

    /// DIV register
    /// INT and FRAC form a fixed-point fractional number.
    /// Counting rate is system clock frequency divided by this number.
    /// Fractional division uses simple 1st-order sigma-delta.
    DIV [
        FRAC OFFSET(0) NUMBITS(4) [],
        INT OFFSET(4) NUMBITS(8) []
    ],

    /// Direct access to the PWM counter
    CTR [
        CTR OFFSET(0) NUMBITS(16) []
    ],

    /// Counter compare values
    CC [
        A OFFSET(0) NUMBITS(16) [],
        B OFFSET(16) NUMBITS(16) []
    ],

    /// Counter wrap value
    TOP [
        TOP OFFSET(0) NUMBITS(16) []
    ],

    /// Control multiple channels at once.
    /// Each bit controls one channel.
    CH [
        CH0 0,
        CH1 1,
        CH2 2,
        CH3 3,
        CH4 4,
        CH5 5,
        CH6 6,
        CH7 7
    ]
];

#[repr(C)]
struct Ch {
    /// Control and status register
    csr: ReadWrite<u32>,
    /// Division register
    div: ReadWrite<u32>,
    /// Direct access to the PWM counter register
    ctr: ReadWrite<u32>,
    /// Counter compare values register
    cc: ReadWrite<u32>,
    /// Counter wrap value register
    top: ReadWrite<u32>
}

#[repr(C)]
struct PwmRegisters {
    /// Channel registers
    ch: [Ch; 7],
    /// Enable register
    /// This register aliases the CSR_EN bits for all channels.
    /// Writing to this register allows multiple channels to be enabled or disabled
    /// or disables simultaneously, so they can run in perfect sync.
    en: ReadWrite<u32>,
    /// Raw interrupts register
    intr: WriteOnly<u32>,
    /// Interrupt enable register
    inte: ReadWrite<u32>,
    /// Interrupt force register
    intf: ReadWrite<u32>,
    /// Interrupt status after masking & forcing
    ints: ReadOnly<u32>
}

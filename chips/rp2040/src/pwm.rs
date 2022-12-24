//! PWM driver for RP2040.

//use kernel::hil;
use kernel::utilities::registers::{register_bitfields, ReadWrite, ReadOnly, WriteOnly};

register_bitfields![u32,
    CSR [
        EN OFFSET(0) NUMBITS(1) [], // Enable PWM channel
        PH_CORRECT OFFSET(1) NUMBITS(1) [], // Enable phase-correct modulation
        A_INV OFFSET(2) NUMBITS(1) [], // Invert output A
        B_INV OFFSET(3) NUMBITS(1) [], // Invert output B
        // PWM slice event selection for fractional clock divider
        // Default value = FREE_RUNNING (always on)
        // If the event is different from FREE_RUNNING, then pin B becomes
        // an input pin
        DIVMOD OFFSET(4) NUMBITS(2) [
            FREE_RUNNING = 0, // Free-running counting at rate dictated by fractional divider
            B_HIGH = 1, // Fractional divider operation is gated by the PWM B pin
            B_RISING = 2, // Counter advances with each rising edge of the PWM B pin
            B_FALLING = 3 // Counter advances with each falling edge of the PWM B pin
        ],
        // Retard the phase of the counter by 1 count, while it is running
        // Self-clearing. Write a 1, and poll until low. Counter must be running.
        PH_RET OFFSET(6) NUMBITS(1) [],
        // Advance the phase of the counter by 1 count, while it is running
        // Self clearing. Write a 1, and poll until low. Counter must be running.
        PH_ADV OFFSET(7) NUMBITS(1) []
    ],

    DIV [
        FRAC OFFSET(0) NUMBITS(4) [],
        INT OFFSET(4) NUMBITS(8) []
    ],

    CTR [
        CTR OFFSET(0) NUMBITS(16) []
    ],

    CC [
        A OFFSET(0) NUMBITS(16) [],
        B OFFSET(16) NUMBITS(16) []
    ],

    TOP [
        TOP OFFSET(0) NUMBITS(16) []
    ]
];

#[repr(C)]
struct Ch {
    csr: ReadWrite<u32>,
    div: ReadWrite<u32>,
    ctr: ReadWrite<u32>,
    cc: ReadWrite<u32>,
    top: ReadWrite<u32>
}

#[repr(C)]
struct PwmRegisters {
    ch: [Ch; 7],
    en: ReadWrite<u32>,
    intr: WriteOnly<u32>,
    inte: ReadWrite<u32>,
    intf: ReadWrite<u32>,
    ints: ReadOnly<u32>
}
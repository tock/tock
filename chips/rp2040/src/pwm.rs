//! PWM driver for RP2040.

//use kernel::hil;
use kernel::utilities::registers::{ReadWrite, ReadOnly, WriteOnly};

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

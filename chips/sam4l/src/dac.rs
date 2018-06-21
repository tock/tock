//! Implementation of the SAM4L DACC.
//!
//! Ensure that the `ADVREFP` pin is tied to `ADDANA`.
//!
//! - Author: Justin Hsieh <hsiehju@umich.edu>
//! - Date: May 26th, 2017

use core::cell::Cell;
use kernel::common::regs::{ReadOnly, ReadWrite, WriteOnly};
use kernel::common::StaticRef;
use kernel::hil;
use kernel::ReturnCode;
use pm::{self, Clock, PBAClock};

#[repr(C)]
pub struct DacRegisters {
    // From page 905 of SAM4L manual
    cr: WriteOnly<u32, Control::Register>, //             Control                       (0x00)
    mr: ReadWrite<u32, Mode::Register>,    //                Mode                          (0x04)
    cdr: WriteOnly<u32, ConversionData::Register>, //     Conversion Data Register      (0x08)
    ier: WriteOnly<u32, InterruptEnable::Register>, //    Interrupt Enable Register     (0x0c)
    idr: WriteOnly<u32, InterruptDisable::Register>, //   Interrupt Disable Register    (0x10)
    imr: ReadOnly<u32, InterruptMask::Register>, //       Interrupt Mask Register       (0x14)
    isr: ReadOnly<u32, InterruptStatus::Register>, //     Interrupt Status Register     (0x18)
    _reserved0: [u32; 50], //                                                               (0x1c - 0xe0)
    wpmr: ReadWrite<u32, WriteProtectMode::Register>, //  Write Protect Mode Register   (0xe4)
    wpsr: ReadOnly<u32, WriteProtectStatus::Register>, // Write Protect Status Register (0xe8)
    _reserved1: [u32; 4], //                                                                (0xec - 0xf8)
    version: ReadOnly<u32, Version::Register>, //         Version Register              (0xfc)
}

register_bitfields![u32,
    Control [
        SWRST 0
    ],

    /// Mode of the DAC peripheral.
    Mode [
        /// Clock divider for internal trigger
        CLKDIV   OFFSET(16)  NUMBITS(16) [],
        /// Startup time selection
        STARTUP  OFFSET( 8)  NUMBITS(8) [],
        /// Word transfer
        WORD     OFFSET( 5)  NUMBITS(1) [
            HalfWordTransfer = 0b0,
            FullWordTransfer = 0b1
        ],
        /// DAC enable
        DACEN    OFFSET( 4)  NUMBITS(1) [],
        /// Trigger selection
        TRGSEL   OFFSET( 1)  NUMBITS(3) [
            ExternalTrigger = 0b000,
            PeripheralTrigger = 0b001
        ],
        /// Trigger enable
        TRGEN    OFFSET( 0)  NUMBITS(1) [
            InternalTrigger = 0b0,
            ExternalTrigger = 0b1
        ]
    ],

    /// Conversion Data Register
    ConversionData [
        /// Data to convert
        DATA OFFSET(0) NUMBITS(32) []
    ],

    /// Interupt Enable Register
    InterruptEnable [
        /// TX ready
        TXRDY 0
    ],

    /// Interrupt Disable Register
    InterruptDisable [
        /// TX ready
        TXRDY 0
    ],

    /// Interrupt Mask Register
    InterruptMask [
        /// TX ready
        TXRDY 0
    ],

    /// Interrupt Status Register
    InterruptStatus [
        /// TX ready
        TXRDY 0
    ],

    /// Write Protect Mode Register
    WriteProtectMode [
        /// Write protect key
        WPKEY OFFSET(8) NUMBITS(24) [],
        /// Write protect enable
        WPEN OFFSET(0) NUMBITS(1) []
    ],

    /// Write Protect Status Register
    WriteProtectStatus [
        /// Write protection error address
        WPROTADDR OFFSET(8) NUMBITS(8) [],
        /// Write protection error
        WPROTERR OFFSET(0) NUMBITS(1) []
    ],

    /// Version Register
    Version [
        VARIANT OFFSET(16) NUMBITS(3) [],
        VERSION OFFSET( 0) NUMBITS(12) []
    ]
];

// Page 59 of SAM4L data sheet
const DAC_BASE: StaticRef<DacRegisters> =
    unsafe { StaticRef::new(0x4003C000 as *const DacRegisters) };

pub struct Dac {
    registers: StaticRef<DacRegisters>,
    enabled: Cell<bool>,
}

pub static mut DAC: Dac = Dac::new(DAC_BASE);

impl Dac {
    const fn new(base_address: StaticRef<DacRegisters>) -> Dac {
        Dac {
            registers: base_address,
            enabled: Cell::new(false),
        }
    }

    // Not currently using interrupt.
    pub fn handle_interrupt(&mut self) {}
}

impl hil::dac::DacChannel for Dac {
    fn initialize(&self) -> ReturnCode {
        let regs: &DacRegisters = &*self.registers;
        if !self.enabled.get() {
            self.enabled.set(true);

            // Start the APB clock (CLK_DACC)
            pm::enable_clock(Clock::PBA(PBAClock::DACC));

            // Reset DACC
            regs.cr.write(Control::SWRST::SET);

            // Set Mode Register
            // -half-word transfer mode
            // -start up time max (0xFF)
            // -clock divider from 48 MHz to 500 kHz (0x60)
            // -internal trigger
            // -enable dacc
            let mr = Mode::WORD::HalfWordTransfer + Mode::STARTUP.val(0xff) + Mode::CLKDIV.val(0x60)
                + Mode::TRGEN::InternalTrigger + Mode::DACEN::SET;
            regs.mr.write(mr);
        }
        ReturnCode::SUCCESS
    }

    fn set_value(&self, value: usize) -> ReturnCode {
        let regs: &DacRegisters = &*self.registers;
        if !self.enabled.get() {
            ReturnCode::EOFF
        } else {
            // Check if ready to write to CDR
            if !regs.isr.is_set(InterruptStatus::TXRDY) {
                return ReturnCode::EBUSY;
            }

            // Write to CDR
            regs.cdr.write(ConversionData::DATA.val(value as u32));
            ReturnCode::SUCCESS
        }
    }
}

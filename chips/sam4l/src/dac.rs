//! Implementation of the SAM4L DACC.
//!
//! Ensure that the `ADVREFP` pin is tied to `ADDANA`.
//!
//! - Author: Justin Hsieh <hsiehju@umich.edu>
//! - Date: May 26th, 2017

use core::cell::Cell;
use core::mem;
use kernel::ReturnCode;
use kernel::common::VolatileCell;
use kernel::hil;
use pm::{self, Clock, PBAClock};

#[repr(C, packed)]
pub struct DacRegisters {
    // From page 905 of SAM4L manual
    cr: VolatileCell<u32>, //      Control                       (0x00)
    mr: VolatileCell<u32>, //      Mode                          (0x04)
    cdr: VolatileCell<u32>, //     Conversion Data Register      (0x08)
    ier: VolatileCell<u32>, //     Interrupt Enable Register     (0x0c)
    idr: VolatileCell<u32>, //     Interrupt Disable Register    (0x10)
    imr: VolatileCell<u32>, //     Interrupt Mask Register       (0x14)
    isr: VolatileCell<u32>, //     Interrupt Status Register     (0x18)
    _reserved0: [u32; 50], //                                    (0x1c - 0xe0)
    wpmr: VolatileCell<u32>, //    Write Protect Mode Register   (0xe4)
    wpsr: VolatileCell<u32>, //    Write Protect Status Register (0xe8)
    _reserved1: [u32; 4], //                                     (0xec - 0xf8)
    version: VolatileCell<u32>, // Version Register              (0xfc)
}

// Page 59 of SAM4L data sheet
const BASE_ADDRESS: *mut DacRegisters = 0x4003C000 as *mut DacRegisters;

pub struct Dac {
    registers: *mut DacRegisters,
    enabled: Cell<bool>,
}

pub static mut DAC: Dac = Dac::new(BASE_ADDRESS);

impl Dac {
    const fn new(base_address: *mut DacRegisters) -> Dac {
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
        let regs: &mut DacRegisters = unsafe { mem::transmute(self.registers) };
        if !self.enabled.get() {
            self.enabled.set(true);

            // Start the APB clock (CLK_DACC)
            unsafe {
                pm::enable_clock(Clock::PBA(PBAClock::DACC));
            }

            // Reset DACC
            regs.cr.set(1);

            // Set Mode Register
            // -half-word transfer mode
            // -start up time max (0xFF)
            // -clock divider from 48 MHz to 500 kHz (0x60)
            // -internal trigger
            // -enable dacc
            //       word       startup       clkdiv         dacen
            let mr = (0 << 5) | (0xff << 8) | (0x60 << 16) | (1 << 4);
            regs.mr.set(mr);
        }
        ReturnCode::SUCCESS
    }


    fn set_value(&self, value: usize) -> ReturnCode {
        let regs: &mut DacRegisters = unsafe { mem::transmute(self.registers) };
        if !self.enabled.get() {
            ReturnCode::EOFF
        } else {
            let isr = regs.isr.get();

            // Check if ready to write to CDR
            if (isr & 0x01) == 0 {
                return ReturnCode::EBUSY;
            }

            // Write to CDR
            regs.cdr.set(value as u32);
            ReturnCode::SUCCESS
        }
    }
}

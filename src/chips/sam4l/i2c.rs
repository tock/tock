/*
 * I2C Support for the Atmel SAM4L.
 *
 * Uses the TWIM peripheral.
 */

use core::intrinsics;

use hil;
use pm;



// Listing of all registers related to the TWIM peripheral.
// Section 27.9 of the datasheet
#[repr(C, packed)]
#[allow(dead_code)]
struct Registers {
    control:                         usize,
    clock_waveform_generator:        usize,
    smbus_timing:                    usize,
    command:                         usize,
    next_command:                    usize,
    receive_holding:                 usize,
    transmit_holding:                usize,
    status:                          usize,
    interrupt_enable:                usize,
    interrupt_disable:               usize,
    interrupt_mask:                  usize,
    status_clear:                    usize,
    parameter:                       usize,
    version:                         usize,
    hsmode_clock_waveform_generator: usize,
    slew_rate:                       usize,
    hsmod_slew_rate:                 usize
}

// The addresses in memory (7.1 of manual) of the TWIM peripherals
const I2C_BASE_ADDRS: [usize; 4] = [0x40018000, 0x4001C000, 0x40078000, 0x4007C000];

// There are four TWIM (two wire master interface) peripherals on the SAM4L.
// These likely won't all be used for I2C, but we let the platform decide
// which one to use.
#[derive(Clone,Copy)]
pub enum Location {
    I2C00,  // TWIMS0
    I2C01,  // TWIMS1
    I2C02,  // TWIM2
    I2C03   // TWIM3
}

// Three main I2C speeds
#[derive(Clone,Copy)]
pub enum Speed {
    Standard100k,
    Fast400k,
    FastPlus1M
}

// These parameters are passed in from the platform's device tree.
#[derive(Clone,Copy)]
pub struct I2CParams {
    pub location: Location,
    pub bus_speed: Speed
}

// This is instantiated when an I2C device is created by the device tree.
// This represents an abstraction of the peripheral hardware.
pub struct I2CDevice {
    registers: &'static mut Registers,  // Pointer to the I2C registers in memory
    bus_speed: Speed,
    clock: ::pm::Clock
}

// Need to implement the `new` function on the I2C device as a constructor.
// This gets called from the device tree.
impl I2CDevice {
    pub fn new (location: Location, bus_speed: Speed) -> I2CDevice {
        let address: usize = I2C_BASE_ADDRS[location as usize];

        // Create the actual device
        let device = I2CDevice {
            registers: unsafe { intrinsics::transmute(address) },
            bus_speed: bus_speed,
            clock: match location {
                Location::I2C00 => pm::Clock::PBA(pm::PBAClock::TWIM0),
                Location::I2C01 => pm::Clock::PBA(pm::PBAClock::TWIM1),
                Location::I2C02 => pm::Clock::PBA(pm::PBAClock::TWIM2),
                Location::I2C03 => pm::Clock::PBA(pm::PBAClock::TWIM3)
            }
        };

        // return
        device
    }

    /// Set the clock prescaler and the time widths of the I2C signals
    /// in the CWGR register to make the bus run at a particular I2C speed.
    pub fn set_bus_speed (&mut self) {

        // Set the clock speed parameters. This could be made smarter, but for
        // now we just use precomputed constants based on a 48MHz clock.
        // See line 320 in asf-2.31.0/sam/drivers/twim/twim.c for where I
        // got these values.
        // clock_speed / bus_speed / 2
        let (exp, data, stasto, high, low) = match self.bus_speed {
            /*Speed::Standard100k => (0, 0, 120, 120, 120),
            Speed::Fast400k =>     (0, 0,  30,  30,  30),
            Speed::FastPlus1M =>   (0, 0,  12,  12,  12),*/
            _ => (7, 10, 200, 100, 100)
        };

        let cwgr = ((exp & 0x7) << 28) |
                   ((data & 0xF) << 24) |
                   ((stasto & 0xFF) << 16) |
                   ((high & 0xFF) << 8) |
                   ((low & 0xFF) << 0);
        volatile!(self.registers.clock_waveform_generator = cwgr);
    }
}


impl hil::i2c::I2C for I2CDevice {

    /// This enables the entire I2C peripheral
    fn enable (&mut self) {
        // Enable the clock for the TWIM module
        unsafe {
            pm::enable_clock(self.clock);
        }

        // enable, reset, disable
        volatile!(self.registers.control = 0x1 << 0);
        volatile!(self.registers.control = 0x1 << 7);
        volatile!(self.registers.control = 0x1 << 1);

        // Init the bus speed
        self.set_bus_speed();

        // slew
        volatile!(self.registers.slew_rate = (0x2 << 28) | (7<<16) | (7<<0));

        // clear interrupts
        volatile!(self.registers.status_clear = 0xFFFFFFFF);
    }

    /// This disables the entire I2C peripheral
    fn disable (&mut self) {
        volatile!(self.registers.control = 0x1 << 1);
        unsafe {
            pm::disable_clock(self.clock);
        }
    }

    #[inline(never)]
    fn write_sync (&mut self, addr: u16, data: &[u8]) {

        // enable, reset, disable
        volatile!(self.registers.control = 0x1 << 0);
        volatile!(self.registers.control = 0x1 << 7);
        volatile!(self.registers.control = 0x1 << 1);

        // Configure the command register to instruct the TWIM peripheral
        // to execute the I2C transaction
        let command = (data.len() << 16) |             // NBYTES
                      (0x1 << 15) |                    // VALID
                      (0x1 << 14) |                    // STOP
                      (0x1 << 13) |                    // START
                      (0x0 << 11) |                    // TENBIT
                      ((addr as usize) << 1) |         // SADR
                      (0x0 << 0);                      // READ
        volatile!(self.registers.next_command = command);

        // Enable TWIM to send command
        volatile!(self.registers.control = 0x1 << 0);

        // Write all bytes in the data buffer to the I2C peripheral
        for c in data {
            // Wait for the peripheral to tell us that we can
            // write to the TX register
            while volatile!(self.registers.status) & 2 != 2 {}
            volatile!(self.registers.transmit_holding = *c as usize);
        }

        // Wait for the end of the TWIM command
        loop {
            let status = volatile!(self.registers.status);
            // CCOMP
            if status & (1 << 3) == (1 << 3) {
                break;
            }
        }
    }

    fn read_sync (&mut self, addr: u16, buffer: &mut[u8]) {

        // enable, reset, disable
        volatile!(self.registers.control = 0x1 << 0);
        volatile!(self.registers.control = 0x1 << 7);
        volatile!(self.registers.control = 0x1 << 1);

        // Configure the command register to instruct the TWIM peripheral
        // to execute the I2C transaction
        let command = (buffer.len() << 16) |           // NBYTES
                      (0x1 << 15) |                    // VALID
                      (0x1 << 14) |                    // STOP
                      (0x1 << 13) |                    // START
                      (0x0 << 11) |                    // TENBIT
                      ((addr as usize) << 1) |         // SADR
                      (0x1 << 0);                      // READ
        volatile!(self.registers.command = command);

        volatile!(self.registers.control = 0x1 << 0);

        // Read bytes in to the buffer
        for i in 0..buffer.len() {
            // Wait for the peripheral to tell us that we can
            // read from the RX register
            loop {
                let status = volatile!(self.registers.status);
                // TODO: define these constants somewhere
                // RXRDY
                if status & (1 << 0) == (1 << 0) {
                    break;
                }
            }
            buffer[i] = (volatile!(self.registers.receive_holding)) as u8;
        }

        // Wait for the end of the TWIM command
        loop {
            let status = volatile!(self.registers.status);
            // CCOMP
            if status & (1 << 3) == (1 << 3) {
                break;
            }
        }
    }
}

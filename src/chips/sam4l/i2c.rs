/*
 * I2C Support for the Atmel SAM4L.
 *
 * Uses the TWIM peripheral.
 */

use helpers::*;
use core::mem;

use common::take_cell::TakeCell;

use hil;
use pm;
use dma::{DMAChannel, DMAClient, DMAPeripheral};
use nvic;

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

// This is instantiated when an I2C device is created by the device tree.
// This represents an abstraction of the peripheral hardware.
pub struct I2CDevice {
    registers: *mut Registers,  // Pointer to the I2C registers in memory
    clock: pm::Clock,
    dma: TakeCell<&'static DMAChannel>,
    dma_pids: (DMAPeripheral, DMAPeripheral),
    nvic: nvic::NvicIdx,
    client: TakeCell<&'static hil::i2c::I2CClient>,
}

pub static mut I2C0 : I2CDevice = I2CDevice {
    registers: I2C_BASE_ADDRS[0] as *mut Registers,
    clock: pm::Clock::PBA(pm::PBAClock::TWIM0),
    dma: TakeCell::empty(),
    dma_pids: (DMAPeripheral::TWIM0_RX, DMAPeripheral::TWIM0_TX),
    nvic: nvic::NvicIdx::TWIM0,
    client: TakeCell::empty(),
};

pub static mut I2C1 : I2CDevice = I2CDevice {
    registers: I2C_BASE_ADDRS[1] as *mut Registers,
    clock: pm::Clock::PBA(pm::PBAClock::TWIM1),
    dma: TakeCell::empty(),
    dma_pids: (DMAPeripheral::TWIM1_RX, DMAPeripheral::TWIM1_TX),
    nvic: nvic::NvicIdx::TWIM1,
    client: TakeCell::empty(),
};

pub static mut I2C2 : I2CDevice = I2CDevice {
    registers: I2C_BASE_ADDRS[2] as *mut Registers,
    clock: pm::Clock::PBA(pm::PBAClock::TWIM2),
    dma: TakeCell::empty(),
    dma_pids: (DMAPeripheral::TWIM2_RX, DMAPeripheral::TWIM2_TX),
    nvic: nvic::NvicIdx::TWIM2,
    client: TakeCell::empty(),
};

pub static mut I2C3 : I2CDevice = I2CDevice {
    registers: I2C_BASE_ADDRS[3] as *mut Registers,
    clock: pm::Clock::PBA(pm::PBAClock::TWIM0),
    dma: TakeCell::empty(),
    dma_pids: (DMAPeripheral::TWIM3_RX, DMAPeripheral::TWIM3_TX),
    nvic: nvic::NvicIdx::TWIM3,
    client: TakeCell::empty(),
};

pub const START : usize = 1 << 13;
pub const STOP : usize = 1 << 14;
pub const ACKLAST : usize = 1 << 24;

// Need to implement the `new` function on the I2C device as a constructor.
// This gets called from the device tree.
impl I2CDevice {
    /// Set the clock prescaler and the time widths of the I2C signals
    /// in the CWGR register to make the bus run at a particular I2C speed.
    fn set_bus_speed (&self) {

        // Set the clock speed parameters. This could be made smarter, but for
        // now we just use precomputed constants based on a 48MHz clock.
        // See line 320 in asf-2.31.0/sam/drivers/twim/twim.c for where I
        // got these values.
        // clock_speed / bus_speed / 2
        let (exp, data, stasto, high, low) = (7, 10, 200, 100, 100);

        let cwgr = ((exp & 0x7) << 28) |
                   ((data & 0xF) << 24) |
                   ((stasto & 0xFF) << 16) |
                   ((high & 0xFF) << 8) |
                   ((low & 0xFF) << 0);
        let regs : &mut Registers = unsafe {mem::transmute(self.registers)};
        volatile_store(&mut regs.clock_waveform_generator, cwgr);
    }

    pub fn set_dma(&self, dma: &'static DMAChannel) {
        self.dma.replace(dma);
    }

    pub fn set_client(&self, client: &'static hil::i2c::I2CClient) {
        self.client.replace(client);
    }

    pub fn handle_interrupt(&self) {
        use hil::i2c::Error;
        let regs : &mut Registers = unsafe {mem::transmute(self.registers)};

        let old_status = volatile_load(&regs.status);

        volatile_store(&mut regs.command, 0);
        volatile_store(&mut regs.status_clear, !0);

        let err = match old_status {
            x if x & (1 <<  8) != 0 /*ANACK*/ => Some(Error::AddressNak),
            x if x & (1 <<  9) != 0 /*DNACK*/ => Some(Error::DataNak),
            x if x & (1 << 10) != 0 /*ARBLST*/ => Some(Error::ArbitrationLost),
            _ => None
        };

        err.map(|err| {
            self.client.map(|client| {
                let buf = self.dma.map(|dma| { dma.abort_xfer() });
                buf.map(|buf| {
                    client.command_error(buf, err);
                });
            });
        });
    }

    fn setup_xfer(&self, chip: u8, flags: usize, read: bool, len: usize) {
        let regs : &mut Registers = unsafe {mem::transmute(self.registers)};

        // disable before configuring
        volatile_store(&mut regs.control, 0x1 << 1);

        let read = if read { 1 } else { 0 };
        let command = (0 << 1) // Not read
                    | ((chip as usize) << 1) // 7 bit address at offset 1 (8th
                                             // bit is ignored anyway)
                    | flags  // START, STOP & ACKLAST flags
                    | (1 << 15) // VALID
                    | len << 16 // NBYTES (at most 255)
                    | read;
        volatile_store(&mut regs.command, command);

        // Enable to begin transfer
        volatile_store(&mut regs.control, 0x1 << 0);

        // Enable transaction error interrupts
        volatile_store(&mut regs.interrupt_enable,
                         (1 << 8)    // ANAK   - Address not ACKd
                       | (1 << 9)    // DNAK   - Data not ACKd
                       | (1 << 10)); // ARBLST - Abitration lost
    }

    pub fn write(&self, chip: u8, flags: usize, data: &'static mut [u8])
            -> usize {
        let len = data.len() & 0xFF;

        self.dma.map(move |dma| {
            dma.enable();
            dma.prepare_xfer(self.dma_pids.1 as usize, data, len);
            self.setup_xfer(chip, flags, false, len);
            dma.start_xfer();
        });

        len
    }

    pub fn read(&self, chip: u8, flags: usize, data: &'static mut [u8])
            -> usize {
        let len = data.len() & 0xFF;

        self.dma.map(move |dma| {
            dma.enable();
            dma.prepare_xfer(self.dma_pids.0 as usize, data, len);
            self.setup_xfer(chip, flags, true, len);
            dma.start_xfer();
        });

        len
    }

    fn enable_interrupts(&self) {
        unsafe {
            nvic::enable(self.nvic);
        }
    }

    fn disable_interrupts(&self) {
        unsafe {
            nvic::disable(self.nvic);
        }
    }
}

impl DMAClient for I2CDevice {
    fn xfer_done(&mut self, _pid: usize, buffer: &'static mut [u8]) {
        self.client.map(move |client| {
            client.command_complete(buffer);
        });
    }
}

impl hil::i2c::I2C for I2CDevice {

    /// This enables the entire I2C peripheral
    fn enable(&self) {
        // Enable the clock for the TWIM module
        unsafe {
            pm::enable_clock(self.clock);
        }

        let regs : &mut Registers = unsafe {mem::transmute(self.registers)};

        // enable, reset, disable
        volatile_store(&mut regs.control, 0x1 << 0);
        volatile_store(&mut regs.control, 0x1 << 7);
        volatile_store(&mut regs.control, 0x1 << 1);

        // Init the bus speed
        self.set_bus_speed();

        // slew
        volatile_store(&mut regs.slew_rate, (0x2 << 28) | (7<<16) | (7<<0));

        // clear interrupts
        volatile_store(&mut regs.status_clear, !0);

        self.enable_interrupts();
    }

    /// This disables the entire I2C peripheral
    fn disable (&self) {
        let regs : &mut Registers = unsafe {mem::transmute(self.registers)};
        volatile_store(&mut regs.control, 0x1 << 1);
        unsafe {
            pm::disable_clock(self.clock);
        }
        self.disable_interrupts();
    }

    fn write_sync (&self, addr: u16, data: &[u8]) {
        let regs : &mut Registers = unsafe {mem::transmute(self.registers)};

        // enable, reset, disable
        volatile_store(&mut regs.control, 0x1 << 0);
        volatile_store(&mut regs.control, 0x1 << 7);
        volatile_store(&mut regs.control, 0x1 << 1);

        // Configure the command register to instruct the TWIM peripheral
        // to execute the I2C transaction
        let command = (data.len() << 16) |             // NBYTES
                      (0x1 << 15) |                    // VALID
                      (0x1 << 14) |                    // STOP
                      (0x1 << 13) |                    // START
                      (0x0 << 11) |                    // TENBIT
                      ((addr as usize) << 1) |         // SADR
                      (0x0 << 0);                      // READ
        volatile_store(&mut regs.next_command, command);

        // Enable TWIM to send command
        volatile_store(&mut regs.control, 0x1 << 0);

        // Write all bytes in the data buffer to the I2C peripheral
        for c in data {
            // Wait for the peripheral to tell us that we can
            // write to the TX register
            while volatile_load(&regs.status) & 2 != 2 {}
            volatile_store(&mut regs.transmit_holding, *c as usize);
        }

        // Wait for the end of the TWIM command
        loop {
            let status = volatile_load(&regs.status);
            // CCOMP
            if status & (1 << 3) == (1 << 3) {
                break;
            }
        }
    }

    fn read_sync (&self, addr: u16, buffer: &mut[u8]) {
        let regs : &mut Registers = unsafe {mem::transmute(self.registers)};

        // enable, reset, disable
        volatile_store(&mut regs.control, 0x1);
        volatile_store(&mut regs.control, 0x1 << 7);
        volatile_store(&mut regs.control, 0x1 << 1);

        // Configure the command register to instruct the TWIM peripheral
        // to execute the I2C transaction
        let command = (buffer.len() << 16) |           // NBYTES
                      (0x1 << 15) |                    // VALID
                      (0x1 << 14) |                    // STOP
                      (0x1 << 13) |                    // START
                      (0x0 << 11) |                    // TENBIT
                      ((addr as usize) << 1) |         // SADR
                      (0x1 << 0);                      // READ
        volatile_store(&mut regs.command, command);

        volatile_store(&mut regs.control, 0x1 << 0);

        // Read bytes in to the buffer
        for i in 0..buffer.len() {
            // Wait for the peripheral to tell us that we can
            // read from the RX register
            loop {
                let status = volatile_load(&regs.status);
                // TODO: define these constants somewhere
                // RXRDY
                if status & (1 << 0) == (1 << 0) {
                    break;
                }
            }
            buffer[i] = (volatile_load(&regs.receive_holding)) as u8;
        }

        // Wait for the end of the TWIM command
        loop {
            let status = volatile_load(&regs.status);
            // CCOMP
            if status & (1 << 3) == (1 << 3) {
                break;
            }
        }
    }
}

pub unsafe extern "C" fn twim2_interrupt() {
    use chip;
    use common::Queue;

    let dev = &I2C2;
    dev.disable_interrupts();
    chip::INTERRUPT_QUEUE.as_mut().unwrap().enqueue(dev.nvic);
}


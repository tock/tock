//! TWIM Driver for the SAM4L
//!
//! The implementation, especially of repeated starts, is quite sensitive to the
//! ordering of operations (e.g. setup DMA, then set command register, then next
//! command register, then enable, then start the DMA transfer). The placement
//! of writes to interrupt enable/disable registers is also significant, but not
//! refactored in such a way that's very logical right now.
//!
//! The point is that until this changes, and this notice is taken away: IF YOU
//! CHANGE THIS DRIVER, TEST RIGOROUSLY!!!
//!



use common::take_cell::TakeCell;
use core::mem;
use dma::{DMAChannel, DMAClient, DMAPeripheral};
use helpers::*;

use hil;
use nvic;
use pm;

// Listing of all registers related to the TWIM peripheral.
// Section 27.9 of the datasheet
#[repr(C, packed)]
#[allow(dead_code)]
struct Registers {
    control: usize,
    clock_waveform_generator: usize,
    smbus_timing: usize,
    command: usize,
    next_command: usize,
    receive_holding: usize,
    transmit_holding: usize,
    status: usize,
    interrupt_enable: usize,
    interrupt_disable: usize,
    interrupt_mask: usize,
    status_clear: usize,
    parameter: usize,
    version: usize,
    hsmode_clock_waveform_generator: usize,
    slew_rate: usize,
    hsmod_slew_rate: usize,
}

// The addresses in memory (7.1 of manual) of the TWIM peripherals
const I2C_BASE_ADDRS: [*mut Registers; 4] = [0x40018000 as *mut Registers,
                                             0x4001C000 as *mut Registers,
                                             0x40078000 as *mut Registers,
                                             0x4007C000 as *mut Registers];

// There are four TWIM (two wire master interface) peripherals on the SAM4L.
// These likely won't all be used for I2C, but we let the platform decide
// which one to use.
#[derive(Clone,Copy)]
pub enum Location {
    I2C00, // TWIMS0
    I2C01, // TWIMS1
    I2C02, // TWIM2
    I2C03, // TWIM3
}

// Three main I2C speeds
#[derive(Clone,Copy)]
pub enum Speed {
    Standard100k,
    Fast400k,
    FastPlus1M,
}

// This is instantiated when an I2C device is created by the device tree.
// This represents an abstraction of the peripheral hardware.
pub struct I2CDevice {
    registers: *mut Registers, // Pointer to the I2C registers in memory
    clock: pm::Clock,
    dma: TakeCell<&'static DMAChannel>,
    dma_pids: (DMAPeripheral, DMAPeripheral),
    nvic: nvic::NvicIdx,
    client: TakeCell<&'static hil::i2c::I2CClient>,
    on_deck: TakeCell<(DMAPeripheral, usize)>,
}

pub static mut I2C0: I2CDevice = I2CDevice::new(I2C_BASE_ADDRS[0],
                                                pm::PBAClock::TWIM0,
                                                nvic::NvicIdx::TWIM0,
                                                DMAPeripheral::TWIM0_RX,
                                                DMAPeripheral::TWIM0_TX);
pub static mut I2C1: I2CDevice = I2CDevice::new(I2C_BASE_ADDRS[1],
                                                pm::PBAClock::TWIM1,
                                                nvic::NvicIdx::TWIM1,
                                                DMAPeripheral::TWIM1_RX,
                                                DMAPeripheral::TWIM1_TX);
pub static mut I2C2: I2CDevice = I2CDevice::new(I2C_BASE_ADDRS[2],
                                                pm::PBAClock::TWIM2,
                                                nvic::NvicIdx::TWIM2,
                                                DMAPeripheral::TWIM2_RX,
                                                DMAPeripheral::TWIM2_TX);
pub static mut I2C3: I2CDevice = I2CDevice::new(I2C_BASE_ADDRS[3],
                                                pm::PBAClock::TWIM3,
                                                nvic::NvicIdx::TWIM3,
                                                DMAPeripheral::TWIM3_RX,
                                                DMAPeripheral::TWIM3_TX);

pub const START: usize = 1 << 13;
pub const STOP: usize = 1 << 14;
pub const ACKLAST: usize = 1 << 24;

// Need to implement the `new` function on the I2C device as a constructor.
// This gets called from the device tree.
impl I2CDevice {
    const fn new(base_addr: *mut Registers,
                 clock: pm::PBAClock,
                 nvic: nvic::NvicIdx,
                 dma_rx: DMAPeripheral,
                 dma_tx: DMAPeripheral)
                 -> I2CDevice {
        I2CDevice {
            registers: base_addr as *mut Registers,
            clock: pm::Clock::PBA(clock),
            dma: TakeCell::empty(),
            dma_pids: (dma_rx, dma_tx),
            nvic: nvic,
            client: TakeCell::empty(),
            on_deck: TakeCell::empty(),
        }
    }

    /// Set the clock prescaler and the time widths of the I2C signals
    /// in the CWGR register to make the bus run at a particular I2C speed.
    fn set_bus_speed(&self) {

        // These parameters are copied from the Michael's TinyOS implementation. Are they correct?
        // Who knows... Michael probably knows. We were originally copying the parameters from the
        // Atmel Software Framework, but those parameters didn't agree with the accelerometer and
        // nearly burned my fingers. Michael's parameters seem to work without danger of injury.
        //
        // Ultimately we should understand what the heck these parameters actually mean and either
        // confirm them or replace them. They almost certainly depend on the clock speed of the
        // CPU, so we'll need to change them if we change the CPU clock speed.
        let (exp, data, stasto, high, low) = (3, 4, 10, 10, 10);

        let cwgr = ((exp & 0x7) << 28) | ((data & 0xF) << 24) | ((stasto & 0xFF) << 16) |
                   ((high & 0xFF) << 8) | ((low & 0xFF) << 0);
        let regs: &mut Registers = unsafe { mem::transmute(self.registers) };
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
        let regs: &mut Registers = unsafe { mem::transmute(self.registers) };

        let old_status = volatile_load(&regs.status);

        volatile_store(&mut regs.status_clear, !0);

        let err = match old_status {
            x if x & (1 <<  8) != 0 /*ANACK*/  => Some(Error::AddressNak),
            x if x & (1 <<  9) != 0 /*DNACK*/  => Some(Error::DataNak),
            x if x & (1 << 10) != 0 /*ARBLST*/ => Some(Error::ArbitrationLost),
            x if x & (1 <<  3) != 0 /*CCOMP*/   => Some(Error::CommandComplete),
            _ => None
        };

        match self.on_deck.take() {
            None => {
                volatile_store(&mut regs.command, 0);
                volatile_store(&mut regs.next_command, 0);

                err.map(|err| {
                    // enable, reset, disable
                    volatile_store(&mut regs.control, 0x1 << 0);
                    volatile_store(&mut regs.control, 0x1 << 7);
                    volatile_store(&mut regs.control, 0x1 << 1);

                    self.client.map(|client| {
                        let buf = match self.dma.take() {
                            Some(dma) => {
                                let b = dma.abort_xfer();
                                self.dma.replace(dma);
                                b
                            }
                            None => None,
                        };
                        buf.map(|buf| {
                            client.command_complete(buf, err);
                        });
                    });
                });
            }
            Some((dma_periph, len)) => {
                // Enable transaction error interrupts
                volatile_store(&mut regs.interrupt_enable,
                               (1 << 3)    // CCOMP   - Command completed
                               | (1 << 8)    // ANAK   - Address not ACKd
                               | (1 << 9)    // DNAK   - Data not ACKd
                               | (1 << 10)); // ARBLST - Abitration lost
                self.dma.map(|dma| {
                    let buf = dma.abort_xfer().unwrap();
                    dma.prepare_xfer(dma_periph, buf, len);
                    dma.start_xfer();
                });
            }
        }
    }

    fn setup_xfer(&self, chip: u8, flags: usize, read: bool, len: u8) {
        let regs: &mut Registers = unsafe { mem::transmute(self.registers) };

        // disable before configuring
        volatile_store(&mut regs.control, 0x1 << 1);

        let read = if read { 1 } else { 0 };
        let command = ((chip as usize) << 1) // 7 bit address at offset 1 (8th
                                             // bit is ignored anyway)
                    | flags  // START, STOP & ACKLAST flags
                    | (1 << 15) // VALID
                    | (len as usize) << 16 // NBYTES (at most 255)
                    | read;
        volatile_store(&mut regs.command, command);
        volatile_store(&mut regs.next_command, 0);

        // Enable transaction error interrupts
        volatile_store(&mut regs.interrupt_enable,
                       (1 << 3)    // CCOMP   - Command completed
                       | (1 << 8)    // ANAK   - Address not ACKd
                       | (1 << 9)    // DNAK   - Data not ACKd
                       | (1 << 10)); // ARBLST - Abitration lost
    }

    fn setup_nextfer(&self, chip: u8, flags: usize, read: bool, len: u8) {
        let regs: &mut Registers = unsafe { mem::transmute(self.registers) };

        // disable before configuring
        volatile_store(&mut regs.control, 0x1 << 1);

        let read = if read { 1 } else { 0 };
        let command = ((chip as usize) << 1) // 7 bit address at offset 1 (8th
                                             // bit is ignored anyway)
                    | flags  // START, STOP & ACKLAST flags
                    | (1 << 15) // VALID
                    | (len as usize) << 16 // NBYTES (at most 255)
                    | read;
        volatile_store(&mut regs.next_command, command);

        // Enable
        volatile_store(&mut regs.control, 0x1 << 0);
    }

    fn master_enable(&self) {
        let regs: &mut Registers = unsafe { mem::transmute(self.registers) };

        // Enable to begin transfer
        volatile_store(&mut regs.control, 0x1 << 0);

    }

    pub fn write(&self, chip: u8, flags: usize, data: &'static mut [u8], len: u8) {
        self.dma.map(move |dma| {
            dma.enable();
            dma.prepare_xfer(self.dma_pids.1, data, len as usize);
            self.setup_xfer(chip, flags, false, len);
            self.master_enable();
            dma.start_xfer();
        });
    }

    pub fn read(&self, chip: u8, flags: usize, data: &'static mut [u8], len: u8) {
        self.dma.map(move |dma| {
            dma.enable();
            dma.prepare_xfer(self.dma_pids.0, data, len as usize);
            self.setup_xfer(chip, flags, true, len);
            self.master_enable();
            dma.start_xfer();
        });
    }

    pub fn write_read(&self, chip: u8, data: &'static mut [u8], split: u8, read_len: u8) {
        self.dma.map(move |dma| {
            dma.enable();
            dma.prepare_xfer(self.dma_pids.1, data, split as usize);
            self.setup_xfer(chip, START, false, split);
            self.setup_nextfer(chip, START | STOP, true, read_len);
            self.on_deck.replace((self.dma_pids.0, read_len as usize));
            dma.start_xfer();
        });
    }

    fn enable_interrupts(&self) {
        unsafe {
            nvic::enable(self.nvic);
        }
    }

    fn disable_interrupts(&self) {
        let regs: &mut Registers = unsafe { mem::transmute(self.registers) };
        volatile_store(&mut regs.interrupt_disable, !0);
        unsafe {
            nvic::disable(self.nvic);
        }
    }
}

impl DMAClient for I2CDevice {
    fn xfer_done(&mut self, _pid: usize) {}
}

impl hil::i2c::I2CController for I2CDevice {
    /// This enables the entire I2C peripheral
    fn enable(&self) {
        // Enable the clock for the TWIM module
        unsafe {
            pm::enable_clock(self.clock);
        }

        let regs: &mut Registers = unsafe { mem::transmute(self.registers) };

        // enable, reset, disable
        volatile_store(&mut regs.control, 0x1 << 0);
        volatile_store(&mut regs.control, 0x1 << 7);
        volatile_store(&mut regs.control, 0x1 << 1);

        // Init the bus speed
        self.set_bus_speed();

        // slew
        volatile_store(&mut regs.slew_rate, (0x2 << 28) | (7 << 16) | (7 << 0));

        // clear interrupts
        volatile_store(&mut regs.status_clear, !0);

        self.enable_interrupts();
    }

    /// This disables the entire I2C peripheral
    fn disable(&self) {
        let regs: &mut Registers = unsafe { mem::transmute(self.registers) };
        volatile_store(&mut regs.control, 0x1 << 1);
        unsafe {
            pm::disable_clock(self.clock);
        }
        self.disable_interrupts();
    }

    fn write(&self, addr: u8, data: &'static mut [u8], len: u8) {
        I2CDevice::write(self, addr, START | STOP, data, len);
    }

    fn read(&self, addr: u8, data: &'static mut [u8], len: u8) {
        I2CDevice::read(self, addr, START | STOP, data, len);
    }

    fn write_read(&self, addr: u8, data: &'static mut [u8], write_len: u8, read_len: u8) {
        I2CDevice::write_read(self, addr, data, write_len, read_len)
    }
}

interrupt_handler!(twim0_handler, TWIM0, {
    I2C0.disable_interrupts()
});
interrupt_handler!(twim1_handler, TWIM1, {
    I2C1.disable_interrupts()
});
interrupt_handler!(twim2_handler, TWIM2, {
    I2C2.disable_interrupts()
});
interrupt_handler!(twim3_handler, TWIM3, {
    I2C3.disable_interrupts()
});

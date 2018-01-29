//! Implementation of I2C for nRF52 using EasyDMA.
//!
//! This module supports nRF52's two I2C master (`TWIM`) peripherals,
//! but not I2C slave (`TWIS`).
//!
//! - Author: Jay Kickliter
//! - Author: Andrew Thompson
//! - Date: Nov 4, 2017

use core::cell::Cell;
use kernel::common::take_cell::TakeCell;
use kernel::hil;
use nrf5x::pinmux::Pinmux;

/// An I2C master device.
///
/// A `TWIM` instance wraps a `registers::TWIM` together with
/// additional data necessary to implement an asynchronous interface.
pub struct TWIM {
    registers: *const registers::TWIM,
    client: Cell<Option<&'static hil::i2c::I2CHwMasterClient>>,
    buf: TakeCell<'static, [u8]>,
}

/// I2C bus speed.
#[repr(u32)]
pub enum Speed {
    K100 = 0x01980000,
    K250 = 0x04000000,
    K400 = 0x06400000,
}

impl TWIM {
    const fn new(instance: usize) -> TWIM {
        TWIM {
            registers: registers::INSTANCES[instance],
            client: Cell::new(None),
            buf: TakeCell::empty(),
        }
    }

    pub fn set_client(&self, client: &'static hil::i2c::I2CHwMasterClient) {
        debug_assert!(self.client.get().is_none());
        self.client.set(Some(client));
    }

    fn regs(&self) -> &registers::TWIM {
        unsafe { &*self.registers }
    }

    /// Configures an already constructed `TWIM`.
    pub fn configure(&self, scl: Pinmux, sda: Pinmux) {
        let regs = self.regs();
        regs.psel_scl.set(scl);
        regs.psel_sda.set(sda);
    }

    /// Sets the I2C bus speed to one of three possible values
    /// enumerated in `Speed`.
    pub fn set_speed(&self, speed: Speed) {
        let regs = self.regs();
        regs.frequency.set(speed as u32);
    }

    /// Enables hardware TWIM peripheral.
    pub fn enable(&self) {
        self.regs().enable.set(6);
    }

    /// Disables hardware TWIM peripheral.
    pub fn disable(&self) {
        self.regs().enable.set(0);
    }

    pub fn handle_interrupt(&self) {
        if self.regs().events_stopped.get() == 1 {
            self.regs().events_stopped.set(0);
            match self.client.get() {
                None => (),
                Some(client) => match self.buf.take() {
                    None => (),
                    Some(buf) => {
                        client.command_complete(buf, hil::i2c::Error::CommandComplete);
                    }
                },
            };
        }

        if self.regs().events_error.get() == 1 {
            self.regs().events_error.set(0);
            let errorsrc = self.regs().errorsrc.get();
            self.regs().errorsrc.set(registers::ErrorSrc::None);
            match self.client.get() {
                None => (),
                Some(client) => match self.buf.take() {
                    None => (),
                    Some(buf) => {
                        client.command_complete(buf, errorsrc.into());
                    }
                },
            };
        }

        // We can blindly clear the following events since we're not using them.
        self.regs().events_suspended.set(0);
        self.regs().events_rxstarted.set(0);
        self.regs().events_lastrx.set(0);
        self.regs().events_lasttx.set(0);
    }

    pub fn is_enabled(&self) -> bool {
        self.regs().enable.get() == 6
    }
}

impl hil::i2c::I2CMaster for TWIM {
    fn enable(&self) {
        self.enable();
    }

    fn disable(&self) {
        self.disable();
    }

    fn write_read(&self, addr: u8, data: &'static mut [u8], write_len: u8, read_len: u8) {
        self.regs().address.set((addr >> 1) as u32);
        self.regs().txd_ptr.set(data.as_mut_ptr());
        self.regs().txd_maxcnt.set(write_len as u32);
        self.regs().rxd_ptr.set(data.as_mut_ptr());
        self.regs().rxd_maxcnt.set(read_len as u32);
        self.regs().shorts.set({
            let mut shorts = registers::Shorts(0);
            // Use the NRF52 shortcut register to configure the peripheral to
            // switch to RX after TX is complete, and then to switch to the STOP
            // state once TX is done. This avoids us having to juggle tasks in
            // the interrupt handler.
            shorts.set_lasttx_startrx(1);
            shorts.set_lastrx_stop(1);
            shorts
        });
        self.regs().intenset.set({
            let mut intenset = registers::InterruptEnable(0);
            intenset.set_stopped(1);
            intenset.set_error(1);
            intenset
        });
        // start the transfer
        self.regs().tasks_starttx.set(1);
        self.buf.replace(data);
    }

    fn write(&self, addr: u8, data: &'static mut [u8], len: u8) {
        self.regs().address.set((addr >> 1) as u32);
        self.regs().txd_ptr.set(data.as_mut_ptr());
        self.regs().txd_maxcnt.set(len as u32);
        self.regs().shorts.set({
            let mut shorts = registers::Shorts(0);
            // Use the NRF52 shortcut register to switch to the STOP state once
            // the TX is complete.
            shorts.set_lasttx_stop(1);
            shorts
        });
        self.regs().intenset.set({
            let mut intenset = registers::InterruptEnable(0);
            intenset.set_stopped(1);
            intenset.set_error(1);
            intenset
        });
        // start the transfer
        self.regs().tasks_starttx.set(1);
        self.buf.replace(data);
    }

    fn read(&self, addr: u8, buffer: &'static mut [u8], len: u8) {
        self.regs().address.set((addr >> 1) as u32);
        self.regs().rxd_ptr.set(buffer.as_mut_ptr());
        self.regs().rxd_maxcnt.set(len as u32);
        self.regs().shorts.set({
            let mut shorts = registers::Shorts(0);
            // Use the NRF52 shortcut register to switch to the STOP state once
            // the RX is complete.
            shorts.set_lastrx_stop(1);
            shorts
        });
        self.regs().intenset.set({
            let mut intenset = registers::InterruptEnable(0);
            intenset.set_stopped(1);
            intenset.set_error(1);
            intenset
        });
        // start the transfer
        self.regs().tasks_startrx.set(1);
        self.buf.replace(buffer);
    }
}

impl hil::i2c::I2CSlave for TWIM {
    fn enable(&self) {
        panic!("I2C slave not implemented for nRF52");
    }
    fn disable(&self) {
        panic!("I2C slave not implemented for nRF52");
    }
    fn set_address(&self, _addr: u8) {
        panic!("I2C slave not implemented for nRF52");
    }
    fn write_receive(&self, _data: &'static mut [u8], _max_len: u8) {
        panic!("I2C slave not implemented for nRF52");
    }
    fn read_send(&self, _data: &'static mut [u8], _max_len: u8) {
        panic!("I2C slave not implemented for nRF52");
    }
    fn listen(&self) {
        panic!("I2C slave not implemented for nRF52");
    }
}

impl hil::i2c::I2CMasterSlave for TWIM {}

/// I2C master instace 0.
pub static mut TWIM0: TWIM = TWIM::new(0);
/// I2C master instace 1.
pub static mut TWIM1: TWIM = TWIM::new(1);

// SPI0_TWI0_Handler and SPI1_TWI1_Handler live in
// `spi.rs`. `service_pending_interrupts` dispatches the correct
// handler based on which peripheral is enabled.

mod registers {
    #![allow(dead_code)]

    use kernel::common::VolatileCell;
    use kernel::hil;
    use nrf5x::pinmux::Pinmux;

    /// Represents allowable values of `errorsrc` register.
    #[repr(u32)]
    #[derive(Debug, Copy, Clone)]
    pub enum ErrorSrc {
        None = 0,
        AddressNack = 1 << 1,
        DataNack = 1 << 2,
    }

    impl From<ErrorSrc> for hil::i2c::Error {
        fn from(errorsrc: ErrorSrc) -> hil::i2c::Error {
            match errorsrc {
                ErrorSrc::None => hil::i2c::Error::CommandComplete,
                ErrorSrc::AddressNack => hil::i2c::Error::AddressNak,
                ErrorSrc::DataNack => hil::i2c::Error::DataNak,
            }
        }
    }

    bitfield!{
        /// Represents bitfields in `shorts` register.
        #[derive(Copy, Clone)]
        pub struct Shorts(u32);
        impl Debug;
        pub lasttx_startrx, set_lasttx_startrx:  7,  7;
        pub lasttx_suspend, set_lasttx_suspend:  8,  8;
        pub lasttx_stop,    set_lasttx_stop:     9,  9;
        pub lastrx_starttx, set_lastrx_starttx: 10, 10;
        pub lastrx_stop,    set_lastrx_stop:    12, 12;
    }

    bitfield!{
        /// Represents bitfields in `intenset` and `intenclr` registers.
        #[derive(Copy, Clone)]
        pub struct InterruptEnable(u32);
        impl Debug;
        pub stopped,   set_stopped:    1,  1;
        pub error,     set_error:      9,  9;
        pub suspended, set_suspended: 18, 18;
        pub rxstarted, set_rxstarted: 19, 19;
        pub txstarted, set_txstarted: 20, 20;
        pub lastrx,    set_lastrx:    23, 23;
        pub lasttx,    set_lasttx:    24, 24;
    }

    /// Uninitialized `TWIM` instances.
    pub const INSTANCES: [*const TWIM; 2] = [0x40003000 as *const TWIM, 0x40004000 as *const TWIM];

    pub struct TWIM {
        /// Start TWI receive sequence
        ///
        /// addr = base + 0x000
        pub tasks_startrx: VolatileCell<u32>,
        _reserved_0: [u32; 1],
        /// Start TWI transmit sequence
        ///
        /// addr = base + 0x008
        pub tasks_starttx: VolatileCell<u32>,
        _reserved_1: [u32; 2],
        /// Stop TWI transaction_ Must be issued while the TWI master is not suspended_
        ///
        /// addr = base + 0x014
        pub tasks_stop: VolatileCell<u32>,
        _reserved_2: [u32; 1],
        /// Suspend TWI transaction
        ///
        /// addr = base + 0x01C
        pub tasks_suspend: VolatileCell<u32>,
        /// Resume TWI transaction
        ///
        /// addr = base + 0x020
        pub tasks_resume: VolatileCell<u32>,
        _reserved_3: [u32; 56],
        /// TWI stopped
        ///
        /// addr = base + 0x104
        pub events_stopped: VolatileCell<u32>,
        _reserved_4: [u32; 7],
        /// TWI error
        ///
        /// addr = base + 0x124
        pub events_error: VolatileCell<u32>,
        _reserved_5: [u32; 8],
        /// Last byte has been sent out after the SUSPEND task has
        /// been issued, TWI traffic is now suspended
        ///
        /// addr = base + 0x148
        pub events_suspended: VolatileCell<u32>,
        /// Receive sequence started
        ///
        /// addr = base + 0x14C
        pub events_rxstarted: VolatileCell<u32>,
        /// Transmit sequence started
        ///
        /// addr = base + 0x150
        pub events_txstarted: VolatileCell<u32>,
        _reserved_6: [u32; 2],
        /// Byte boundary, starting to receive the last byte
        ///
        /// addr = base + 0x15C
        pub events_lastrx: VolatileCell<u32>,
        /// Byte boundary, starting to transmit the last byte
        ///
        /// addr = base + 0x160
        pub events_lasttx: VolatileCell<u32>,
        _reserved_7: [u32; 39],
        /// Shortcut register
        ///
        /// addr = base + 0x200
        pub shorts: VolatileCell<Shorts>,
        _reserved_8: [u32; 63],
        /// Enable or disable interrupt
        ///
        /// addr = base + 0x300
        pub inten: VolatileCell<InterruptEnable>,
        /// Enable interrupt
        ///
        /// addr = base + 0x304
        pub intenset: VolatileCell<InterruptEnable>,
        /// Disable interrupt
        ///
        /// addr = base + 0x308
        pub intenclr: VolatileCell<InterruptEnable>,
        _reserved_9: [u32; 110],
        /// Error source
        ///
        /// addr = base + 0x4C4
        pub errorsrc: VolatileCell<ErrorSrc>,
        _reserved_10: [u32; 14],
        /// Enable TWIM
        ///
        /// addr = base + 0x500
        pub enable: VolatileCell<u32>,
        _reserved_11: [u32; 1],
        /// Pin select for SCL signal
        ///
        /// addr = base + 0x508
        pub psel_scl: VolatileCell<Pinmux>,
        /// Pin select for SDA signal
        ///
        /// addr = base + 0x50C
        pub psel_sda: VolatileCell<Pinmux>,
        _reserved_12: [u32; 5],
        /// TWI frequency
        ///
        /// addr = base + 0x524
        pub frequency: VolatileCell<u32>,
        _reserved_13: [u32; 3],
        /// Data pointer
        ///
        /// addr = base + 0x534
        pub rxd_ptr: VolatileCell<*mut u8>,
        /// Maximum number of bytes in receive buffer
        ///
        /// addr = base + 0x538
        pub rxd_maxcnt: VolatileCell<u32>,
        /// Number of bytes transferred in the last transaction
        ///
        /// addr = base + 0x53C
        pub rxd_amount: VolatileCell<u32>,
        /// EasyDMA list type
        ///
        /// addr = base + 0x540
        pub rxd_list: VolatileCell<u32>,
        /// Data pointer
        ///
        /// addr = base + 0x544
        pub txd_ptr: VolatileCell<*mut u8>,
        /// Maximum number of bytes in transmit buffer
        ///
        /// addr = base + 0x548
        pub txd_maxcnt: VolatileCell<u32>,
        /// Number of bytes transferred in the last transaction
        ///
        /// addr = base + 0x54C
        pub txd_amount: VolatileCell<u32>,
        /// EasyDMA list type
        ///
        /// addr = base + 0x550
        pub txd_list: VolatileCell<u32>,
        _reserved_14: [u32; 13],
        /// Address used in the TWI transfer
        ///
        /// addr = base + 0x588
        pub address: VolatileCell<u32>,
    }
}

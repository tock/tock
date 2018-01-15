//! Implementation of SPI for NRF52 using EasyDMA.
//!
//! This file only implments support for the three SPI master (`SPIM`)
//! peripherals, and not SPI slave (`SPIS`).
//!
//! Although `kernel::hil::spi::SpiMaster` is implemented for `SPIM`,
//! only the functions marked with `x` are fully defined:
//!
//! - [x] set_client
//! - [x] init
//! - [x] is_busy
//! - [x] read_write_bytes
//! - [ ] write_byte
//! - [ ] read_byte
//! - [ ] read_write_byte
//! - [x] specify_chip_select
//! - [x] set_rate
//! - [x] get_rate
//! - [x] set_clock
//! - [x] get_clock
//! - [x] set_phase
//! - [x] get_phase
//! - [ ] hold_low
//! - [ ] release_low
//!
//! - Author: Jay Kickliter
//! - Date: Sep 10, 2017

use core::cell::Cell;
use core::cmp;
use core::ptr;
use kernel::ReturnCode;
use kernel::common::take_cell::TakeCell;
use kernel::hil;
use nrf5x::pinmux::Pinmux;

/// SPI master instance 0.
pub static mut SPIM0: SPIM = SPIM::new(0);
/// SPI master instance 1.
pub static mut SPIM1: SPIM = SPIM::new(1);
/// SPI master instance 2.
pub static mut SPIM2: SPIM = SPIM::new(2);

mod registers {
    pub mod spim {
        //! NRF52 `SPIM` registers and utility types.
        #![allow(dead_code)]
        use kernel::common::VolatileCell;
        use nrf5x::pinmux::Pinmux;

        /// Uninitialized `SPIM` instances.
        pub const INSTANCES: [*const SPIM; 3] = [
            0x40003000 as *const SPIM,
            0x40004000 as *const SPIM,
            0x40023000 as *const SPIM,
        ];

        bitfield!{
            /// Represents bitfields in `intenset` and `intenclr` registers.
            #[derive(Copy, Clone)]
            pub struct InterruptEnable(u32);
            impl Debug;
            pub stopped, set_stopped:  1,  1;
            pub end_rx,  set_end_rx:   4,  4;
            pub end,     set_end:      6,  6;
            pub end_tx,  set_end_tx:   8,  8;
            pub started, set_started: 19, 19;
        }

        bitfield!{
            /// Represents bitfields in `config` register.
            #[derive(Copy, Clone)]
            pub struct Config(u32);
            impl Debug;
            pub order,          set_order:          0,  0;
            pub clock_phase,    set_clock_phase:    1,  1;
            pub clock_polarity, set_clock_polarity: 2,  2;
        }

        /// An enum representing all allowable `frequency` register values.
        #[repr(u32)]
        #[derive(Copy, Clone)]
        pub enum Frequency {
            K125 = 0x02000000,
            K250 = 0x04000000,
            K500 = 0x08000000,
            M1 = 0x10000000,
            M2 = 0x20000000,
            M4 = 0x40000000,
            M8 = 0x80000000,
        }

        impl From<Frequency> for u32 {
            fn from(freq: Frequency) -> u32 {
                match freq {
                    Frequency::K125 => 125_000,
                    Frequency::K250 => 250_000,
                    Frequency::K500 => 500_000,
                    Frequency::M1 => 1_000_000,
                    Frequency::M2 => 2_000_000,
                    Frequency::M4 => 4_000_000,
                    Frequency::M8 => 8_000_000,
                }
            }
        }

        impl From<u32> for Frequency {
            fn from(freq: u32) -> Frequency {
                if freq < 250_000 {
                    Frequency::K125
                } else if freq < 500_000 {
                    Frequency::K250
                } else if freq < 1_000_000 {
                    Frequency::K500
                } else if freq < 2_000_000 {
                    Frequency::M1
                } else if freq < 4_000_000 {
                    Frequency::M2
                } else if freq < 8_000_000 {
                    Frequency::M4
                } else {
                    Frequency::M8
                }
            }
        }

        /// Represents one of NRF52's three `SPIM` instances.
        #[repr(C)]
        pub struct SPIM {
            _reserved0: [u32; 4],
            /// Start SPI transaction
            ///
            /// addr = base + 0x010
            pub tasks_start: VolatileCell<u32>,
            /// Stop SPI transaction
            ///
            /// addr = base + 0x014
            pub tasks_stop: VolatileCell<u32>,
            _reserved1: u32,
            /// Suspend SPI transaction
            ///
            /// addr = base + 0x01C
            pub tasks_suspend: VolatileCell<u32>,
            /// Resume SPI transaction
            ///
            /// addr = base + 0x020
            pub tasks_resume: VolatileCell<u32>,
            _reserved2: [u32; 56],
            /// SPI transaction has stopped
            ///
            /// addr = base + 0x104
            pub events_stopped: VolatileCell<u32>,
            _reserved3: [u32; 2],
            /// End of RXD buffer reached
            ///
            /// addr = base + 0x110
            pub events_endrx: VolatileCell<u32>,
            _reserved4: u32,
            /// End of RXD buffer and TXD buffer reached
            ///
            /// addr = base + 0x118
            pub events_end: VolatileCell<u32>,
            _reserved5: u32,
            /// End of TXD buffer reached
            ///
            /// addr = base + 0x120
            pub events_endtx: VolatileCell<u32>,
            _reserved6: [u32; 10],
            /// Transaction started
            ///
            /// addr = base + 0x14C
            pub events_started: VolatileCell<u32>,
            _reserved7: [u32; 44],
            /// Shortcut register
            ///
            /// addr = base + 0x200
            pub shorts: VolatileCell<u32>,
            _reserved8: [u32; 64],
            /// Enable interrupt
            ///
            /// addr = base + 0x304
            pub intenset: VolatileCell<InterruptEnable>,
            /// Disable interrupt
            ///
            /// base + addr = 0x308
            pub intenclr: VolatileCell<InterruptEnable>,
            _reserved9: [u32; 125],
            /// Enable SPIM
            ///
            /// addr = base + 0x500
            pub enable: VolatileCell<u32>,
            _reserved10: u32,
            /// Pin select for SCK
            ///
            /// addr = base + 0x508
            pub psel_sck: VolatileCell<Pinmux>,
            /// Pin select for MOSI signal
            ///
            /// addr = base + 0x50C
            pub psel_mosi: VolatileCell<Pinmux>,
            /// Pin select for MISO signal
            ///
            /// addr = base + 0x510
            pub psel_miso: VolatileCell<Pinmux>,
            _reserved11: [u32; 4],
            /// SPI frequency
            ///
            /// addr = base + 0x524
            pub frequency: VolatileCell<Frequency>,
            _reserved12: [u32; 3],
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
            /// base + addr = 0x544
            pub txd_ptr: VolatileCell<*const u8>,
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
            /// Configuration register
            ///
            /// addr = base + 0x554
            pub config: VolatileCell<Config>,
            _reserved13: [u32; 26],
            /// Over-read character. Character clocked out in case and over-read of the TXD buffer.
            ///
            /// addr = base + 0x5C0
            pub orc: VolatileCell<u32>,
        }
    }
}

/// A SPI master device.
///
/// A `SPIM` instance wraps a `registers::spim::SPIM` together with
/// addition data necessary to implement an asynchronous interface.
pub struct SPIM {
    registers: *const registers::spim::SPIM,
    client: Cell<Option<&'static hil::spi::SpiMasterClient>>,
    chip_select: Cell<Option<&'static hil::gpio::Pin>>,
    initialized: Cell<bool>,
    busy: Cell<bool>,
    tx_buf: TakeCell<'static, [u8]>,
    rx_buf: TakeCell<'static, [u8]>,
    transfer_len: Cell<usize>,
}

impl SPIM {
    const fn new(instance: usize) -> SPIM {
        SPIM {
            registers: registers::spim::INSTANCES[instance],
            client: Cell::new(None),
            chip_select: Cell::new(None),
            initialized: Cell::new(false),
            busy: Cell::new(false),
            tx_buf: TakeCell::empty(),
            rx_buf: TakeCell::empty(),
            transfer_len: Cell::new(0),
        }
    }

    fn regs(&self) -> &registers::spim::SPIM {
        unsafe { &*self.registers }
    }

    #[inline(never)]
    pub fn handle_interrupt(&self) {
        if self.regs().events_end.get() == 1 {
            // End of RXD buffer and TXD buffer reached
            match self.chip_select.get() {
                Some(cs) => cs.set(),
                None => {
                    debug_assert!(false, "Invariant violated. Chip-select must be Some.");
                    return;
                }
            }
            self.regs().events_end.set(0);

            match self.client.get() {
                None => (),
                Some(client) => match self.tx_buf.take() {
                    None => (),
                    Some(tx_buf) => {
                        client.read_write_done(tx_buf, self.rx_buf.take(), self.transfer_len.take())
                    }
                },
            };

            self.busy.set(false);
        }

        // Although we only configured the chip interrupt on the
        // above 'end' event, the other event fields also get set by
        // the chip. Let's clear those flags.

        if self.regs().events_stopped.get() == 1 {
            // SPI transaction has stopped
            self.regs().events_stopped.set(0);
        }

        if self.regs().events_endrx.get() == 1 {
            // End of RXD buffer reached
            self.regs().events_endrx.set(0);
        }

        if self.regs().events_endtx.get() == 1 {
            // End of TXD buffer reached
            self.regs().events_endtx.set(0);
        }

        if self.regs().events_started.get() == 1 {
            // Transaction started
            self.regs().events_started.set(0);
        }
    }

    /// Configures an already constructed `SPIM`.
    pub fn configure(&self, mosi: Pinmux, miso: Pinmux, sck: Pinmux) {
        let regs = self.regs();
        regs.psel_mosi.set(mosi);
        regs.psel_miso.set(miso);
        regs.psel_sck.set(sck);
        self.enable();
    }

    /// Enables `SPIM` peripheral.
    pub fn enable(&self) {
        self.regs().enable.set(7);
    }

    /// Disables `SPIM` peripheral.
    pub fn disable(&self) {
        self.regs().enable.set(0);
    }

    pub fn is_enabled(&self) -> bool {
        self.regs().enable.get() == 7
    }
}

impl hil::spi::SpiMaster for SPIM {
    type ChipSelect = &'static hil::gpio::Pin;

    fn set_client(&self, client: &'static hil::spi::SpiMasterClient) {
        self.client.set(Some(client));
    }

    fn init(&self) {
        use self::registers::spim::InterruptEnable;
        let mut enabled_ints = InterruptEnable(0);
        enabled_ints.set_end(1);
        self.regs().intenset.set(enabled_ints);
        self.initialized.set(true);
    }

    fn is_busy(&self) -> bool {
        self.busy.get()
    }

    fn read_write_bytes(
        &self,
        tx_buf: &'static mut [u8],
        rx_buf: Option<&'static mut [u8]>,
        len: usize,
    ) -> ReturnCode {
        debug_assert!(self.initialized.get());
        debug_assert!(!self.busy.get());
        debug_assert!(self.tx_buf.is_none());
        debug_assert!(self.rx_buf.is_none());

        // Clear (set to low) chip-select
        match self.chip_select.get() {
            Some(cs) => cs.clear(),
            None => return ReturnCode::ENODEVICE,
        }

        // Setup transmit data registers
        let tx_len: u32 = cmp::min(len, tx_buf.len()) as u32;
        self.regs().txd_ptr.set(tx_buf.as_ptr());
        self.regs().txd_maxcnt.set(tx_len);
        self.tx_buf.replace(tx_buf);

        // Setup receive data registers
        match rx_buf {
            None => {
                self.regs().rxd_ptr.set(ptr::null_mut());
                self.regs().rxd_maxcnt.set(0);
                self.transfer_len.set(tx_len as usize);
                self.rx_buf.put(None);
            }
            Some(buf) => {
                self.regs().rxd_ptr.set(buf.as_mut_ptr());
                let rx_len: u32 = cmp::min(len, buf.len()) as u32;
                self.regs().rxd_maxcnt.set(rx_len);
                self.transfer_len.set(cmp::min(tx_len, rx_len) as usize);
                self.rx_buf.put(Some(buf));
            }
        }

        // Start the transfer
        self.busy.set(true);
        self.regs().tasks_start.set(1);
        ReturnCode::SUCCESS
    }

    fn write_byte(&self, _val: u8) {
        debug_assert!(self.initialized.get());
        unimplemented!("SPI: Use `read_write_bytes()` instead.");
    }

    fn read_byte(&self) -> u8 {
        debug_assert!(self.initialized.get());
        unimplemented!("SPI: Use `read_write_bytes()` instead.");
    }

    fn read_write_byte(&self, _val: u8) -> u8 {
        debug_assert!(self.initialized.get());
        unimplemented!("SPI: Use `read_write_bytes()` instead.");
    }

    // Tell the SPI peripheral what to use as a chip select pin.
    // The type of the argument is based on what makes sense for the
    // peripheral when this trait is implemented.
    fn specify_chip_select(&self, cs: Self::ChipSelect) {
        cs.make_output();
        cs.set();
        self.chip_select.set(Some(cs));
    }

    // Returns the actual rate set
    fn set_rate(&self, rate: u32) -> u32 {
        debug_assert!(self.initialized.get());
        let f = registers::spim::Frequency::from(rate);
        self.regs().frequency.set(f);
        f.into()
    }

    fn get_rate(&self) -> u32 {
        debug_assert!(self.initialized.get());
        self.regs().frequency.get().into()
    }

    fn set_clock(&self, polarity: hil::spi::ClockPolarity) {
        debug_assert!(self.initialized.get());
        debug_assert!(self.initialized.get());
        use self::hil::spi::ClockPolarity;
        let mut config = self.regs().config.get();
        config.set_clock_polarity(match polarity {
            ClockPolarity::IdleLow => 0,
            ClockPolarity::IdleHigh => 1,
        });
        self.regs().config.set(config);
    }

    fn get_clock(&self) -> hil::spi::ClockPolarity {
        debug_assert!(self.initialized.get());
        use self::hil::spi::ClockPolarity;
        let config = self.regs().config.get();
        match config.clock_polarity() {
            0 => ClockPolarity::IdleLow,
            1 => ClockPolarity::IdleHigh,
            _ => unreachable!(),
        }
    }

    fn set_phase(&self, phase: hil::spi::ClockPhase) {
        debug_assert!(self.initialized.get());
        use self::hil::spi::ClockPhase;
        let mut config = self.regs().config.get();
        config.set_clock_phase(match phase {
            ClockPhase::SampleLeading => 0,
            ClockPhase::SampleTrailing => 1,
        });
        self.regs().config.set(config);
    }

    fn get_phase(&self) -> hil::spi::ClockPhase {
        debug_assert!(self.initialized.get());
        use self::hil::spi::ClockPhase;
        let config = self.regs().config.get();
        match config.clock_phase() {
            0 => ClockPhase::SampleLeading,
            1 => ClockPhase::SampleTrailing,
            _ => unreachable!(),
        }
    }

    // The following two trait functions are not implemented for
    // SAM4L, and appear to not provide much functionality. Let's not
    // bother implementing them unless needed.
    fn hold_low(&self) {
        unimplemented!("SPI: Use `read_write_bytes()` instead.");
    }

    fn release_low(&self) {
        unimplemented!("SPI: Use `read_write_bytes()` instead.");
    }
}

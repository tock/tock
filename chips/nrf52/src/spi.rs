//! Implementation of SPI for NRF52 using EasyDMA.
//!
//! This file only implements support for the three SPI master (`SPIM`)
//! peripherals, and not SPI slave (`SPIS`).
//!
//! Although `kernel::hil::spi::SpiMaster` is implemented for `SPIM`,
//! only the functions marked with `x` are fully defined:
//!
//! * [x] set_client
//! * [x] init
//! * [x] is_busy
//! * [x] read_write_bytes
//! * [] write_byte
//! * [] read_byte
//! * [] read_write_byte
//! * [x] specify_chip_select
//! * [x] set_rate
//! * [x] get_rate
//! * [x] set_clock
//! * [x] get_clock
//! * [x] set_phase
//! * [x] get_phase
//! * [] hold_low
//! * [] release_low
//!
//! Author
//! -------------------
//!
//! * Author: Jay Kickliter
//! * Date: Sep 10, 2017

use core::cell::Cell;
use core::cmp;
use core::ptr;
use kernel::common::cells::TakeCell;
use kernel::common::cells::VolatileCell;
use kernel::common::regs::{ReadWrite, WriteOnly};
use kernel::hil;
use kernel::ReturnCode;
use nrf5x::pinmux::Pinmux;

/// SPI master instance 0.
pub static mut SPIM0: SPIM = SPIM::new(0);
/// SPI master instance 1.
pub static mut SPIM1: SPIM = SPIM::new(1);
/// SPI master instance 2.
pub static mut SPIM2: SPIM = SPIM::new(2);

const INSTANCES: [*const SpimRegisters; 3] = [
    0x40003000 as *const SpimRegisters,
    0x40004000 as *const SpimRegisters,
    0x40023000 as *const SpimRegisters,
];

#[repr(C)]
struct SpimRegisters {
    _reserved0: [u8; 16],                            // reserved
    tasks_start: WriteOnly<u32, TASK::Register>,     // Start SPI transaction
    tasks_stop: WriteOnly<u32, TASK::Register>,      // Stop SPI transaction
    _reserved1: [u8; 4],                             // reserved
    tasks_suspend: WriteOnly<u32, TASK::Register>,   // Suspend SPI transaction
    tasks_resume: WriteOnly<u32, TASK::Register>,    // Resume SPI transaction
    _reserved2: [u8; 224],                           // reserved
    events_stopped: ReadWrite<u32, EVENT::Register>, // SPI transaction has stopped
    _reserved3: [u8; 8],                             // reserved
    events_endrx: ReadWrite<u32, EVENT::Register>,   // End of RXD buffer reached
    _reserved4: [u8; 4],                             // reserved
    events_end: ReadWrite<u32, EVENT::Register>,     // End of RXD buffer and TXD buffer reached
    _reserved5: [u8; 4],                             // reserved
    events_endtx: ReadWrite<u32, EVENT::Register>,   // End of TXD buffer reached
    _reserved6: [u8; 40],                            // reserved
    events_started: ReadWrite<u32, EVENT::Register>, // Transaction started
    _reserved7: [u8; 176],                           // reserved
    shorts: ReadWrite<u32>,                          // Shortcut register
    _reserved8: [u8; 256],                           // reserved
    intenset: ReadWrite<u32, INTE::Register>,        // Enable interrupt
    intenclr: ReadWrite<u32, INTE::Register>,        // Disable interrupt
    _reserved9: [u8; 500],                           // reserved
    enable: ReadWrite<u32, ENABLE::Register>,        // Enable SPIM
    _reserved10: [u8; 4],                            // reserved
    psel_sck: VolatileCell<Pinmux>,                  // Pin select for SCK
    psel_mosi: VolatileCell<Pinmux>,                 // Pin select for MOSI signal
    psel_miso: VolatileCell<Pinmux>,                 // Pin select for MISO signal
    _reserved11: [u8; 16],                           // reserved
    frequency: ReadWrite<u32>,                       // SPI frequency
    _reserved12: [u8; 12],                           // reserved
    rxd_ptr: VolatileCell<*mut u8>,                  // Data pointer
    rxd_maxcnt: ReadWrite<u32, MAXCNT::Register>,    // Maximum number of bytes in receive buffer
    rxd_amount: ReadWrite<u32>,                      // Number of bytes transferred
    rxd_list: ReadWrite<u32>,                        // EasyDMA list type
    txd_ptr: VolatileCell<*const u8>,                // Data pointer
    txd_maxcnt: ReadWrite<u32, MAXCNT::Register>,    // Maximum number of bytes in transmit buffer
    txd_amount: ReadWrite<u32>,                      // Number of bytes transferred
    txd_list: ReadWrite<u32>,                        // EasyDMA list type
    config: ReadWrite<u32, CONFIG::Register>,        // Configuration register
    _reserved13: [u8; 104],                          // reserved
    orc: ReadWrite<u32>,                             // Over-read character.
}

register_bitfields![u32,
    INTE [
        /// Write '1' to Enable interrupt on EVENTS_STOPPED event
        STOPPED OFFSET(1) NUMBITS(1) [
            /// Read: Disabled
            ReadDisabled = 0,
            /// Enable
            Enable = 1
        ],
        /// Write '1' to Enable interrupt on EVENTS_ENDRX event
        ENDRX OFFSET(4) NUMBITS(1) [
            /// Read: Disabled
            ReadDisabled = 0,
            /// Enable
            Enable = 1
        ],
        /// Write '1' to Enable interrupt on EVENTS_END event
        END OFFSET(6) NUMBITS(1) [
            /// Read: Disabled
            ReadDisabled = 0,
            /// Enable
            Enable = 1
        ],
        /// Write '1' to Enable interrupt on EVENTS_ENDTX event
        ENDTX OFFSET(8) NUMBITS(1) [
            /// Read: Disabled
            ReadDisabled = 0,
            /// Enable
            Enable = 1
        ],
        /// Write '1' to Enable interrupt on EVENTS_STARTED event
        STARTED OFFSET(19) NUMBITS(1) [
            /// Read: Disabled
            ReadDisabled = 0,
            /// Enable
            Enable = 1
        ]
    ],
    MAXCNT [
        /// Maximum number of bytes in buffer
        MAXCNT OFFSET(0) NUMBITS(16)
    ],
    CONFIG [
        /// Bit order
        ORDER OFFSET(0) NUMBITS(1) [
            /// Most significant bit shifted out first
            MostSignificantBitShiftedOutFirst = 0,
            /// Least significant bit shifted out first
            LeastSignificantBitShiftedOutFirst = 1
        ],
        /// Serial clock (SCK) phase
        CPHA OFFSET(1) NUMBITS(1) [
            /// Sample on leading edge of clock, shift serial data on trailing edge
            SampleOnLeadingEdge = 0,
            /// Sample on trailing edge of clock, shift serial data on leading edge
            SampleOnTrailingEdge = 1
        ],
        /// Serial clock (SCK) polarity
        CPOL OFFSET(2) NUMBITS(1) [
            /// Active high
            ActiveHigh = 0,
            /// Active low
            ActiveLow = 1
        ]
    ],
    ENABLE [
        ENABLE OFFSET(0) NUMBITS(4) [
            Disable = 0,
            Enable = 7
        ]
    ],
    EVENT [
        EVENT 0
    ],
    TASK [
        TASK 0
    ]
];

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

/// A SPI master device.
///
/// A `SPIM` instance wraps a `registers::spim::SPIM` together with
/// addition data necessary to implement an asynchronous interface.
pub struct SPIM {
    registers: *const SpimRegisters,
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
            registers: INSTANCES[instance],
            client: Cell::new(None),
            chip_select: Cell::new(None),
            initialized: Cell::new(false),
            busy: Cell::new(false),
            tx_buf: TakeCell::empty(),
            rx_buf: TakeCell::empty(),
            transfer_len: Cell::new(0),
        }
    }

    fn regs(&self) -> &SpimRegisters {
        unsafe { &*self.registers }
    }

    #[inline(never)]
    pub fn handle_interrupt(&self) {
        if self.regs().events_end.is_set(EVENT::EVENT) {
            // End of RXD buffer and TXD buffer reached
            match self.chip_select.get() {
                Some(cs) => cs.set(),
                None => {
                    debug_assert!(false, "Invariant violated. Chip-select must be Some.");
                    return;
                }
            }
            self.regs().events_end.write(EVENT::EVENT::CLEAR);

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

        if self.regs().events_stopped.is_set(EVENT::EVENT) {
            // SPI transaction has stopped
            self.regs().events_stopped.write(EVENT::EVENT::CLEAR);
        }

        if self.regs().events_endrx.is_set(EVENT::EVENT) {
            // End of RXD buffer reached
            self.regs().events_endrx.write(EVENT::EVENT::CLEAR);
        }

        if self.regs().events_endtx.is_set(EVENT::EVENT) {
            // End of TXD buffer reached
            self.regs().events_endtx.write(EVENT::EVENT::CLEAR);
        }

        if self.regs().events_started.is_set(EVENT::EVENT) {
            // Transaction started
            self.regs().events_started.write(EVENT::EVENT::CLEAR);
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
        self.regs().enable.write(ENABLE::ENABLE::Enable);
    }

    /// Disables `SPIM` peripheral.
    pub fn disable(&self) {
        self.regs().enable.write(ENABLE::ENABLE::Disable);
    }

    pub fn is_enabled(&self) -> bool {
        self.regs().enable.matches_all(ENABLE::ENABLE::Enable)
    }
}

impl hil::spi::SpiMaster for SPIM {
    type ChipSelect = &'static hil::gpio::Pin;

    fn set_client(&self, client: &'static hil::spi::SpiMasterClient) {
        self.client.set(Some(client));
    }

    fn init(&self) {
        self.regs().intenset.write(INTE::END::Enable);
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
        self.regs().txd_maxcnt.write(MAXCNT::MAXCNT.val(tx_len));
        self.tx_buf.replace(tx_buf);

        // Setup receive data registers
        match rx_buf {
            None => {
                self.regs().rxd_ptr.set(ptr::null_mut());
                self.regs().rxd_maxcnt.write(MAXCNT::MAXCNT.val(0));
                self.transfer_len.set(tx_len as usize);
                self.rx_buf.put(None);
            }
            Some(buf) => {
                self.regs().rxd_ptr.set(buf.as_mut_ptr());
                let rx_len: u32 = cmp::min(len, buf.len()) as u32;
                self.regs().rxd_maxcnt.write(MAXCNT::MAXCNT.val(rx_len));
                self.transfer_len.set(cmp::min(tx_len, rx_len) as usize);
                self.rx_buf.put(Some(buf));
            }
        }

        // Start the transfer
        self.busy.set(true);
        self.regs().tasks_start.write(TASK::TASK::SET);
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
        let f = Frequency::from(rate);
        self.regs().frequency.set(f as u32);
        f.into()
    }

    fn get_rate(&self) -> u32 {
        debug_assert!(self.initialized.get());
        self.regs().frequency.get().into()
    }

    fn set_clock(&self, polarity: hil::spi::ClockPolarity) {
        debug_assert!(self.initialized.get());
        debug_assert!(self.initialized.get());
        let new_polarity = match polarity {
            hil::spi::ClockPolarity::IdleLow => CONFIG::CPOL::ActiveHigh,
            hil::spi::ClockPolarity::IdleHigh => CONFIG::CPOL::ActiveLow,
        };
        self.regs().config.modify(new_polarity);
    }

    fn get_clock(&self) -> hil::spi::ClockPolarity {
        debug_assert!(self.initialized.get());
        match self.regs().config.read(CONFIG::CPOL) {
            0 => hil::spi::ClockPolarity::IdleLow,
            1 => hil::spi::ClockPolarity::IdleHigh,
            _ => unreachable!(),
        }
    }

    fn set_phase(&self, phase: hil::spi::ClockPhase) {
        debug_assert!(self.initialized.get());
        let new_phase = match phase {
            hil::spi::ClockPhase::SampleLeading => CONFIG::CPHA::SampleOnLeadingEdge,
            hil::spi::ClockPhase::SampleTrailing => CONFIG::CPHA::SampleOnTrailingEdge,
        };
        self.regs().config.modify(new_phase);
    }

    fn get_phase(&self) -> hil::spi::ClockPhase {
        debug_assert!(self.initialized.get());
        match self.regs().config.read(CONFIG::CPHA) {
            0 => hil::spi::ClockPhase::SampleLeading,
            1 => hil::spi::ClockPhase::SampleTrailing,
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

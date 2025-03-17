// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Provides driver for accessing an SD Card and a userspace Driver.
//!
//! This allows initialization and block reads or writes on top of SPI.
//!
//! Usage
//! -----
//!
//! ```rust,ignore
//! # use kernel::static_init;
//! # use capsules::virtual_alarm::VirtualMuxAlarm;
//! # use kernel::hil::spi::SpiMasterDevice;
//!
//! let spi_mux = components::spi::SpiMuxComponent::new(
//!         &base_peripherals.spim0,
//!         dynamic_deferred_caller
//!     ).finalize(components::spi_mux_component_helper!(nrf52833::spi::SPIM));
//! base_peripherals.spim0.configure(
//!     nrf52833::pinmux::Pinmux::new(SPI_MOSI_PIN as u32), // SD MOSI
//!     nrf52833::pinmux::Pinmux::new(SPI_MISO_PIN as u32), // SD MISO
//!     nrf52833::pinmux::Pinmux::new(SPI_SCK_PIN as u32),  // SD SCK
//! );
//!
//! let sdcard_spi = components::spi::SpiComponent::new(
//!         spi_mux,
//!         &nrf52833_peripherals.gpio_port[SPI_CS_PIN]
//!     ).finalize(components::spi_component_helper!(nrf52833::spi::SPIM));
//!
//! let sdcard_virtual_alarm = static_init!(
//!      capsules::virtual_alarm::VirtualMuxAlarm<'static, nrf52833::rtc::Rtc>,
//!      capsules::virtual_alarm::VirtualMuxAlarm::new(mux_alarm));
//! sdcard_virtual_alarm.setup();
//!
//! let sdcard_tx_buffer = static_init!([u8; capsules::sdcard::TXRX_BUFFER_LENGTH],
//!                                     [0; capsules::sdcard::TXRX_BUFFER_LENGTH]);
//! let sdcard_rx_buffer = static_init!([u8; capsules::sdcard::TXRX_BUFFER_LENGTH],
//!                                     [0; capsules::sdcard::TXRX_BUFFER_LENGTH]);
//!
//! let sdcard = static_init!(
//!     capsules::sdcard::SDCard<
//!         'static,
//!         capsules::virtual_alarm::VirtualMuxAlarm<'static, nrf52833::rtc::Rtc>>,
//!     capsules::sdcard::SDCard::new(sdcard_spi,
//!                                   sdcard_virtual_alarm,
//!                                   Some(&SD_DETECT_PIN),
//!                                   sdcard_tx_buffer,
//!                                   sdcard_rx_buffer));
//! sdcard_spi.set_client(sdcard);
//! sdcard_virtual_alarm.set_alarm_client(sdcard);
//! SD_DETECT_PIN.set_client(sdcard);
//!
//! let sdcard_kernel_buffer = static_init!([u8; capsules::sdcard::KERNEL_BUFFER_LENGTH],
//!                                         [0; capsules::sdcard::KERNEL_BUFFER_LENGTH]);
//!
//! let sdcard_driver = static_init!(
//!     capsules::sdcard::SDCardDriver<
//!         'static,
//!         capsules::virtual_alarm::VirtualMuxAlarm<'static, nrf52833::rtc::Rtc>>,
//!     capsules::sdcard::SDCardDriver::new(
//!         sdcard,
//!         sdcard_kernel_buffer,
//!         board_kernel.create_grant(
//!             capsules::sdcard::DRIVER_NUM,
//!             &memory_allocation_capability)));
//! sdcard.set_client(sdcard_driver);
//! ```

// Resources for SD Card API:
//  * elm-chan.org/docs/mmc/mmc_e.html
//  * alumni.cs.ucr.edu/~amitra/sdcard/Additional/sdcard_appnote_foust.pdf
//  * luckyresistor.me/cat-protector/software/sdcard-2/
//  * http://users.ece.utexas.edu/~valvano/EE345M/SD_Physical_Layer_Spec.pdf

use core::cell::Cell;
use core::cmp;

use kernel::grant::{AllowRoCount, AllowRwCount, Grant, UpcallCount};
use kernel::hil;
use kernel::hil::time::ConvertTicks;
use kernel::processbuffer::{ReadableProcessBuffer, WriteableProcessBuffer};
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::utilities::leasable_buffer::SubSliceMut;
use kernel::{ErrorCode, ProcessId};

/// Syscall driver number.
use capsules_core::driver;
pub const DRIVER_NUM: usize = driver::NUM::SdCard as usize;

/// Ids for read-only allow buffers
mod ro_allow {
    pub const WRITE: usize = 0;
    /// The number of allow buffers the kernel stores for this grant
    pub const COUNT: u8 = 1;
}

/// Ids for read-write allow buffers
mod rw_allow {
    pub const READ: usize = 0;
    /// The number of allow buffers the kernel stores for this grant
    pub const COUNT: u8 = 1;
}

/// Buffers used for SD card transactions, assigned in board `main.rs` files
/// Constraints:
///
///  * RXBUFFER must be greater than or equal to TXBUFFER in length
///  * Both RXBUFFER and TXBUFFER must be longer  than the SD card's block size
pub const TXRX_BUFFER_LENGTH: usize = 515;

/// SD Card capsule, capable of being built on top of by other kernel capsules
pub struct SDCard<'a, A: hil::time::Alarm<'a>> {
    spi: &'a dyn hil::spi::SpiMasterDevice<'a>,
    state: Cell<SpiState>,
    after_state: Cell<SpiState>,

    alarm: &'a A,
    alarm_state: Cell<AlarmState>,
    alarm_count: Cell<u8>,

    is_initialized: Cell<bool>,
    card_type: Cell<SDCardType>,

    detect_pin: Cell<Option<&'a dyn hil::gpio::InterruptPin<'a>>>,

    txbuffer: TakeCell<'static, [u8]>,
    rxbuffer: TakeCell<'static, [u8]>,

    client: OptionalCell<&'a dyn SDCardClient>,
    client_buffer: TakeCell<'static, [u8]>,
    client_offset: Cell<usize>,
}

/// SD card command codes
#[allow(dead_code, non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq)]
enum SDCmd {
    CMD0_Reset = 0,                       //                  Reset
    CMD1_Init = 1,                        //                   Generic init
    CMD8_CheckVoltage = 8,                //           Check voltage range
    CMD9_ReadCSD = 9,                     //                Read chip specific data (CSD) register
    CMD12_StopRead = 12,                  //             Stop multiple block read
    CMD16_SetBlockSize = 16,              //         Set blocksize
    CMD17_ReadSingle = 17,                //           Read single block
    CMD18_ReadMultiple = 18,              //         Read multiple blocks
    CMD24_WriteSingle = 24,               //          Write single block
    CMD25_WriteMultiple = 25,             //        Write multiple blocks
    CMD55_ManufSpecificCommand = 55,      // Next command will be manufacturer specific
    CMD58_ReadOCR = 58,                   //              Read operation condition register (OCR)
    ACMD41_ManufSpecificInit = 0x80 + 41, // Manufacturer specific Init
}

/// SD card response codes
#[allow(dead_code, non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq)]
enum SDResponse {
    R1_Status,         //         Status response, single byte
    R2_ExtendedStatus, // Extended response, two bytes, unused in practice
    R3_OCR,            //            OCR response, status + four bytes
    R7_CheckVoltage,   //   Check voltage response, status + four bytes
}

/// SPI states
#[derive(Clone, Copy, Debug, PartialEq)]
enum SpiState {
    Idle,

    SendManufSpecificCmd { cmd: SDCmd, arg: u32 },

    InitReset,
    InitCheckVersion,
    InitRepeatHCSInit,
    InitCheckCapacity,
    InitAppSpecificInit,
    InitRepeatAppSpecificInit,
    InitRepeatGenericInit,
    InitSetBlocksize,
    InitComplete,

    StartReadBlocks { count: u32 },
    WaitReadBlock,
    ReadBlockComplete,
    WaitReadBlocks { count: u32 },
    ReceivedBlock { count: u32 },
    ReadBlocksComplete,

    StartWriteBlocks { count: u32 },
    WriteBlockResponse,
    WriteBlockBusy,
    WaitWriteBlockBusy,
}

/// Alarm states
#[derive(Clone, Copy, Debug, PartialEq)]
enum AlarmState {
    Idle,

    DetectionChange,

    RepeatHCSInit,
    RepeatAppSpecificInit,
    RepeatGenericInit,

    WaitForDataBlock,
    WaitForDataBlocks { count: u32 },

    WaitForWriteBusy,
}

/// Error codes returned if an SD card transaction fails
#[derive(Clone, Copy, Debug, PartialEq)]
enum SdCardError {
    CardStateChanged = -10001,
    InitializationFailure = -10002,
    ReadFailure = -10003,
    WriteFailure = -10004,
    TimeoutFailure = -10005,
}

/// SD card types, determined during initialization
#[derive(Clone, Copy, Debug, PartialEq)]
enum SDCardType {
    Uninitialized = 0x00,
    MMC = 0x01,
    SDv1 = 0x02,
    SDv2 = 0x04,
    SDv2BlockAddressable = 0x04 | 0x08,
}

// Constants used in driver
const SUCCESS_STATUS: u8 = 0x00;
const INITIALIZING_STATUS: u8 = 0x01;
const DATA_TOKEN: u8 = 0xFE;

/// Callback functions from SDCard
pub trait SDCardClient {
    fn card_detection_changed(&self, installed: bool);
    fn init_done(&self, block_size: u32, total_size: u64);
    fn read_done(&self, data: &'static mut [u8], len: usize);
    fn write_done(&self, buffer: &'static mut [u8]);
    fn error(&self, error: u32);
}

/// Functions for initializing and accessing an SD card
impl<'a, A: hil::time::Alarm<'a>> SDCard<'a, A> {
    /// Create a new SD card interface
    ///
    /// spi - virtualized SPI to use for communication with SD card
    /// alarm - virtualized Timer with a granularity of at least 1 ms
    /// detect_pin - active low GPIO pin used to detect if an SD card is
    ///     installed
    /// txbuffer - buffer for holding SPI write data, at least 515 bytes in
    ///     length
    /// rxbuffer - buffer for holding SPI read data, at least 515 bytes in
    ///     length
    pub fn new(
        spi: &'a dyn hil::spi::SpiMasterDevice,
        alarm: &'a A,
        detect_pin: Option<&'static dyn hil::gpio::InterruptPin<'a>>,
        txbuffer: &'static mut [u8; 515],
        rxbuffer: &'static mut [u8; 515],
    ) -> SDCard<'a, A> {
        // initialize buffers
        for byte in txbuffer.iter_mut() {
            *byte = 0xFF;
        }
        for byte in rxbuffer.iter_mut() {
            *byte = 0xFF;
        }

        // handle optional detect pin
        let pin = detect_pin.map_or(None, |pin| {
            pin.make_input();
            Some(pin)
        });

        // set up and return struct
        SDCard {
            spi,
            state: Cell::new(SpiState::Idle),
            after_state: Cell::new(SpiState::Idle),
            alarm,
            alarm_state: Cell::new(AlarmState::Idle),
            alarm_count: Cell::new(0),
            is_initialized: Cell::new(false),
            card_type: Cell::new(SDCardType::Uninitialized),
            detect_pin: Cell::new(pin),
            txbuffer: TakeCell::new(txbuffer),
            rxbuffer: TakeCell::new(rxbuffer),
            client: OptionalCell::empty(),
            client_buffer: TakeCell::empty(),
            client_offset: Cell::new(0),
        }
    }

    fn set_spi_slow_mode(&self) -> Result<(), ErrorCode> {
        // need to be in slow mode while initializing the SD card
        // set to CPHA=0, CPOL=0, 400 kHZ
        self.spi.configure(
            hil::spi::ClockPolarity::IdleLow,
            hil::spi::ClockPhase::SampleLeading,
            400000,
        )
    }

    fn set_spi_fast_mode(&self) -> Result<(), ErrorCode> {
        // can read/write in fast mode after the SD card is initialized
        // set to CPHA=0, CPOL=0, 4 MHz
        self.spi.configure(
            hil::spi::ClockPolarity::IdleLow,
            hil::spi::ClockPhase::SampleLeading,
            4000000,
        )
    }

    /// send a command over SPI and collect the response
    /// Handles encoding of command, checksum, and padding bytes. The response
    /// still needs to be parsed out of the read_buffer when complete
    fn send_command(
        &self,
        cmd: SDCmd,
        arg: u32,
        write_buffer: &'static mut [u8],
        read_buffer: &'static mut [u8],
        recv_len: usize,
    ) {
        // Note: a good default recv_len is 10 bytes. Reading too many bytes
        //  rarely matters. However, it occasionally matters a lot, so we
        //  provide a settable recv_len

        if self.is_initialized() {
            // device is already initialized
            // TODO verify SPI return value
            let _ = self.set_spi_fast_mode();
        } else {
            // device is still being initialized
            // TODO verify SPI return value
            let _ = self.set_spi_slow_mode();
        }

        // send dummy bytes to start
        write_buffer[0] = 0xFF;
        write_buffer[1] = 0xFF;

        // command
        if (0x80 & cmd as u8) != 0x00 {
            // application-specific command
            write_buffer[2] = 0x40 | (0x7F & cmd as u8);
        } else {
            // normal command
            write_buffer[2] = 0x40 | cmd as u8;
        }

        // argument, MSB first
        write_buffer[3] = ((arg >> 24) & 0xFF) as u8;
        write_buffer[4] = ((arg >> 16) & 0xFF) as u8;
        write_buffer[5] = ((arg >> 8) & 0xFF) as u8;
        write_buffer[6] = ((arg >> 0) & 0xFF) as u8;

        // CRC is ignored except for CMD0 and maybe CMD8
        // Established practice it to just always use the CMD0 CRC unless we
        // are sending a CMD8
        if cmd == SDCmd::CMD8_CheckVoltage {
            write_buffer[7] = 0x87; // valid crc for CMD8(0x1AA)
        } else {
            write_buffer[7] = 0x95; // valid crc for CMD0
        }

        // append dummy bytes to transmission after command bytes
        // Limit to minimum length between write_buffer and recv_len
        for byte in write_buffer.iter_mut().skip(8).take(recv_len) {
            *byte = 0xFF;
        }

        // Length is command bytes (8) plus recv_len
        let len = cmp::min(
            cmp::min(8 + recv_len, write_buffer.len()),
            read_buffer.len(),
        );

        let mut wb: SubSliceMut<'static, u8> = write_buffer.into();
        wb.slice(0..len);
        let mut rb: SubSliceMut<'static, u8> = read_buffer.into();
        rb.slice(0..len);

        // start SPI transaction
        let _ = self.spi.read_write_bytes(wb, Some(rb));
    }

    /// wrapper for easy reading of bytes over SPI
    fn read_bytes(
        &self,
        write_buffer: &'static mut [u8],
        read_buffer: &'static mut [u8],
        recv_len: usize,
    ) {
        // TODO verify SPI return value
        let _ = self.set_spi_fast_mode();

        // set write buffer to null transactions
        // Limit to minimum length between write_buffer and recv_len.
        // Note: this could be optimized in the future by allowing SPI to read
        //  without a write buffer passed in
        for byte in write_buffer.iter_mut().take(recv_len) {
            *byte = 0xFF;
        }

        let mut wb: SubSliceMut<'static, u8> = write_buffer.into();
        wb.slice(0..recv_len);
        let mut rb: SubSliceMut<'static, u8> = read_buffer.into();
        rb.slice(0..recv_len);

        let _ = self.spi.read_write_bytes(wb, Some(rb));
    }

    /// wrapper for easy writing of bytes over SPI
    fn write_bytes(
        &self,
        write_buffer: &'static mut [u8],
        read_buffer: &'static mut [u8],
        recv_len: usize,
    ) {
        // TODO verify SPI return value
        let _ = self.set_spi_fast_mode();

        let mut wb: SubSliceMut<'static, u8> = write_buffer.into();
        wb.slice(0..recv_len);
        let mut rb: SubSliceMut<'static, u8> = read_buffer.into();
        rb.slice(0..recv_len);

        let _ = self.spi.read_write_bytes(wb, Some(rb));
    }

    /// parse response bytes from SPI read buffer
    /// Unfortunately there is a variable amount of delay in SD card responses,
    /// so these bytes must be searched for
    fn get_response(&self, response: SDResponse, read_buffer: &[u8]) -> (u8, u8, u32) {
        let mut r1: u8 = 0xFF;
        let mut r2: u8 = 0xFF;
        let mut r3: u32 = 0xFFFFFFFF;

        // scan through read buffer for response byte
        for (i, &byte) in read_buffer.iter().enumerate() {
            if (byte & 0x80) == 0x00 {
                // status byte is always included
                r1 = byte;

                match response {
                    SDResponse::R2_ExtendedStatus => {
                        // status, then read/write status. Unused in practice
                        if i + 1 < read_buffer.len() {
                            r2 = read_buffer[i + 1];
                        }
                    }
                    SDResponse::R3_OCR | SDResponse::R7_CheckVoltage => {
                        // status, then Operating Condition Register
                        if i + 4 < read_buffer.len() {
                            r3 = (read_buffer[i + 1] as u32) << 24
                                | (read_buffer[i + 2] as u32) << 16
                                | (read_buffer[i + 3] as u32) << 8
                                | (read_buffer[i + 4] as u32);
                        }
                    }
                    _ => {
                        // R1, no bytes left to parse
                    }
                }

                // response found
                break;
            }
        }

        // return a tuple of the parsed bytes
        (r1, r2, r3)
    }

    /// updates SD card state on SPI transaction returns
    fn process_spi_states(
        &self,
        write_buffer: &'static mut [u8],
        read_buffer: &'static mut [u8],
        _: usize,
    ) {
        match self.state.get() {
            SpiState::SendManufSpecificCmd { cmd, arg } => {
                // send the application-specific command and resume the state
                //  machine
                self.state.set(self.after_state.get());
                self.after_state.set(SpiState::Idle);
                self.send_command(cmd, arg, write_buffer, read_buffer, 10);
            }

            SpiState::InitReset => {
                // check response
                let (r1, _, _) = self.get_response(SDResponse::R1_Status, read_buffer);

                // only continue if we are in idle state
                if r1 == INITIALIZING_STATUS {
                    // next send Check Voltage Range command that is only valid
                    //  on SDv2 cards. This is used to check which SD card
                    //  version is installed. Note that 0xAA is an arbitrary
                    //  check pattern that will be duplicated in the response
                    //  and 0x100 specifies that the card is running between
                    //  2.7 and 3.6 volts
                    self.state.set(SpiState::InitCheckVersion);
                    self.send_command(
                        SDCmd::CMD8_CheckVoltage,
                        0x1AA,
                        write_buffer,
                        read_buffer,
                        10,
                    );
                } else {
                    // error, send callback and quit
                    self.txbuffer.replace(write_buffer);
                    self.rxbuffer.replace(read_buffer);
                    self.state.set(SpiState::Idle);
                    self.alarm_state.set(AlarmState::Idle);
                    self.alarm_count.set(0);
                    self.client.map(move |client| {
                        client.error(SdCardError::InitializationFailure as u32);
                    });
                }
            }

            SpiState::InitCheckVersion => {
                // check response
                let (r1, _, r7) = self.get_response(SDResponse::R7_CheckVoltage, read_buffer);

                // look for test pattern duplicated in R7
                if r1 == INITIALIZING_STATUS && r7 == 0x1AA {
                    // we have an SDv2 card
                    // send application-specific initialization in high capacity mode (HCS)
                    self.state.set(SpiState::SendManufSpecificCmd {
                        cmd: SDCmd::ACMD41_ManufSpecificInit,
                        arg: 0x40000000,
                    });
                    self.after_state.set(SpiState::InitRepeatHCSInit);
                    self.send_command(
                        SDCmd::CMD55_ManufSpecificCommand,
                        0x0,
                        write_buffer,
                        read_buffer,
                        10,
                    );
                } else {
                    // we have either an SDv1 or MMCv3 card
                    // send application-specific initialization
                    self.state.set(SpiState::SendManufSpecificCmd {
                        cmd: SDCmd::ACMD41_ManufSpecificInit,
                        arg: 0x0,
                    });
                    self.after_state.set(SpiState::InitAppSpecificInit);
                    self.send_command(
                        SDCmd::CMD55_ManufSpecificCommand,
                        0x0,
                        write_buffer,
                        read_buffer,
                        10,
                    );
                }
            }

            SpiState::InitRepeatHCSInit => {
                // check response
                let (r1, _, _) = self.get_response(SDResponse::R1_Status, read_buffer);

                if r1 == SUCCESS_STATUS {
                    // card initialized
                    // check card capacity
                    self.alarm_count.set(0);
                    self.state.set(SpiState::InitCheckCapacity);
                    self.send_command(SDCmd::CMD58_ReadOCR, 0x0, write_buffer, read_buffer, 10);
                } else if r1 == INITIALIZING_STATUS {
                    // replace buffers
                    self.txbuffer.replace(write_buffer);
                    self.rxbuffer.replace(read_buffer);

                    // try again after 10 ms
                    self.alarm_state.set(AlarmState::RepeatHCSInit);
                    let delay = self.alarm.ticks_from_ms(10);
                    self.alarm.set_alarm(self.alarm.now(), delay);
                } else {
                    // error, send callback and quit
                    self.txbuffer.replace(write_buffer);
                    self.rxbuffer.replace(read_buffer);
                    self.state.set(SpiState::Idle);
                    self.alarm_state.set(AlarmState::Idle);
                    self.alarm_count.set(0);
                    self.client.map(move |client| {
                        client.error(SdCardError::InitializationFailure as u32);
                    });
                }
            }

            SpiState::InitCheckCapacity => {
                // check response
                let (r1, _, r7) = self.get_response(SDResponse::R3_OCR, read_buffer);

                if r1 == SUCCESS_STATUS {
                    if (r7 & 0x40000000) != 0x00000000 {
                        self.card_type.set(SDCardType::SDv2BlockAddressable);
                    } else {
                        self.card_type.set(SDCardType::SDv2);
                    }

                    // Read CSD register
                    // Note that the receive length needs to be increased here
                    //  to capture the 16-byte register (plus some slack)
                    self.state.set(SpiState::InitComplete);
                    self.send_command(SDCmd::CMD9_ReadCSD, 0x0, write_buffer, read_buffer, 28);
                } else {
                    // error, send callback and quit
                    self.txbuffer.replace(write_buffer);
                    self.rxbuffer.replace(read_buffer);
                    self.state.set(SpiState::Idle);
                    self.alarm_state.set(AlarmState::Idle);
                    self.alarm_count.set(0);
                    self.client.map(move |client| {
                        client.error(SdCardError::InitializationFailure as u32);
                    });
                }
            }

            SpiState::InitAppSpecificInit => {
                // check response
                let (r1, _, _) = self.get_response(SDResponse::R1_Status, read_buffer);

                if r1 == INITIALIZING_STATUS || r1 == SUCCESS_STATUS {
                    // SDv1 card
                    // send application-specific initialization
                    self.card_type.set(SDCardType::SDv1);
                    self.state.set(SpiState::SendManufSpecificCmd {
                        cmd: SDCmd::ACMD41_ManufSpecificInit,
                        arg: 0x0,
                    });
                    self.after_state.set(SpiState::InitRepeatAppSpecificInit);
                    self.send_command(
                        SDCmd::CMD55_ManufSpecificCommand,
                        0x0,
                        write_buffer,
                        read_buffer,
                        10,
                    );
                } else {
                    // MMCv3 card
                    // send generic intialization
                    self.card_type.set(SDCardType::MMC);
                    self.state.set(SpiState::InitRepeatGenericInit);
                    self.send_command(SDCmd::CMD1_Init, 0x0, write_buffer, read_buffer, 10);
                }
            }

            SpiState::InitRepeatAppSpecificInit => {
                // check response
                let (r1, _, _) = self.get_response(SDResponse::R1_Status, read_buffer);

                if r1 == SUCCESS_STATUS {
                    // card initialized
                    // set blocksize to 512
                    self.alarm_count.set(0);
                    self.state.set(SpiState::InitSetBlocksize);
                    self.send_command(
                        SDCmd::CMD16_SetBlockSize,
                        512,
                        write_buffer,
                        read_buffer,
                        10,
                    );
                } else if r1 == INITIALIZING_STATUS {
                    // replace buffers
                    self.txbuffer.replace(write_buffer);
                    self.rxbuffer.replace(read_buffer);

                    // try again after 10 ms
                    self.alarm_state.set(AlarmState::RepeatAppSpecificInit);
                    let delay = self.alarm.ticks_from_ms(10);
                    self.alarm.set_alarm(self.alarm.now(), delay);
                } else {
                    // error, send callback and quit
                    self.txbuffer.replace(write_buffer);
                    self.rxbuffer.replace(read_buffer);
                    self.state.set(SpiState::Idle);
                    self.alarm_state.set(AlarmState::Idle);
                    self.alarm_count.set(0);
                    self.client.map(move |client| {
                        client.error(SdCardError::InitializationFailure as u32);
                    });
                }
            }

            SpiState::InitRepeatGenericInit => {
                // check response
                let (r1, _, _) = self.get_response(SDResponse::R1_Status, read_buffer);

                if r1 == SUCCESS_STATUS {
                    // card initialized
                    // set blocksize to 512
                    self.alarm_count.set(0);
                    self.state.set(SpiState::InitSetBlocksize);
                    self.send_command(
                        SDCmd::CMD16_SetBlockSize,
                        512,
                        write_buffer,
                        read_buffer,
                        10,
                    );
                } else if r1 == INITIALIZING_STATUS {
                    // replace buffers
                    self.txbuffer.replace(write_buffer);
                    self.rxbuffer.replace(read_buffer);

                    // try again after 10 ms
                    self.alarm_state.set(AlarmState::RepeatGenericInit);
                    let delay = self.alarm.ticks_from_ms(10);
                    self.alarm.set_alarm(self.alarm.now(), delay);
                } else {
                    // error, send callback and quit
                    self.txbuffer.replace(write_buffer);
                    self.rxbuffer.replace(read_buffer);
                    self.state.set(SpiState::Idle);
                    self.alarm_state.set(AlarmState::Idle);
                    self.alarm_count.set(0);
                    self.client.map(move |client| {
                        client.error(SdCardError::InitializationFailure as u32);
                    });
                }
            }

            SpiState::InitSetBlocksize => {
                // check response
                let (r1, _, _) = self.get_response(SDResponse::R1_Status, read_buffer);

                if r1 == SUCCESS_STATUS {
                    // Read CSD register
                    // Note that the receive length needs to be increased here
                    //  to capture the 16-byte register (plus some slack)
                    self.state.set(SpiState::InitComplete);
                    self.send_command(SDCmd::CMD9_ReadCSD, 0x0, write_buffer, read_buffer, 28);
                } else {
                    // error, send callback and quit
                    self.txbuffer.replace(write_buffer);
                    self.rxbuffer.replace(read_buffer);
                    self.state.set(SpiState::Idle);
                    self.alarm_state.set(AlarmState::Idle);
                    self.alarm_count.set(0);
                    self.client.map(move |client| {
                        client.error(SdCardError::InitializationFailure as u32);
                    });
                }
            }

            SpiState::InitComplete => {
                // check response
                let (r1, _, _) = self.get_response(SDResponse::R1_Status, read_buffer);

                if r1 == SUCCESS_STATUS {
                    let mut total_size: u64 = 0;

                    // find CSD register value
                    // Slide through 12-byte windows searching for beginning of
                    // the CSD register
                    for buf in read_buffer.windows(12) {
                        if buf[0] == DATA_TOKEN {
                            // get total size from CSD
                            if (buf[1] & 0xC0) == 0x00 {
                                // CSD version 1.0
                                let c_size = (((buf[7] & 0x03) as u32) << 10)
                                    | (((buf[8] & 0xFF) as u32) << 2)
                                    | (((buf[9] & 0xC0) as u32) >> 6);
                                let c_size_mult = (((buf[10] & 0x03) as u32) << 1)
                                    | (((buf[11] & 0x80) as u32) >> 7);
                                let read_bl_len = (buf[6] & 0x0F) as u32;

                                let block_count = (c_size + 1) * (1 << (c_size_mult + 2));
                                let block_len = 1 << read_bl_len;
                                total_size = block_count as u64 * block_len as u64;
                            } else {
                                // CSD version 2.0
                                let c_size = (((buf[8] & 0x3F) as u32) << 16)
                                    | (((buf[9] & 0xFF) as u32) << 8)
                                    | ((buf[10] & 0xFF) as u32);
                                total_size = ((c_size as u64) + 1) * 512 * 1024;
                            }

                            break;
                        }
                    }

                    // replace buffers
                    self.txbuffer.replace(write_buffer);
                    self.rxbuffer.replace(read_buffer);

                    // initialization complete
                    self.state.set(SpiState::Idle);
                    self.is_initialized.set(true);

                    // perform callback
                    self.client.map(move |client| {
                        client.init_done(512, total_size);
                    });
                } else {
                    // error, send callback and quit
                    self.txbuffer.replace(write_buffer);
                    self.rxbuffer.replace(read_buffer);
                    self.state.set(SpiState::Idle);
                    self.alarm_state.set(AlarmState::Idle);
                    self.alarm_count.set(0);
                    self.client.map(move |client| {
                        client.error(SdCardError::InitializationFailure as u32);
                    });
                }
            }

            SpiState::StartReadBlocks { count } => {
                // check response
                let (r1, _, _) = self.get_response(SDResponse::R1_Status, read_buffer);

                if r1 == SUCCESS_STATUS {
                    if count <= 1 {
                        // check for data block to be ready
                        self.state.set(SpiState::WaitReadBlock);
                        self.read_bytes(write_buffer, read_buffer, 1);
                    } else {
                        // check for data block to be ready
                        self.state.set(SpiState::WaitReadBlocks { count });
                        self.read_bytes(write_buffer, read_buffer, 1);
                    }
                } else {
                    // error, send callback and quit
                    self.txbuffer.replace(write_buffer);
                    self.rxbuffer.replace(read_buffer);
                    self.state.set(SpiState::Idle);
                    self.alarm_state.set(AlarmState::Idle);
                    self.alarm_count.set(0);
                    self.client.map(move |client| {
                        client.error(SdCardError::ReadFailure as u32);
                    });
                }
            }

            SpiState::WaitReadBlock => {
                if read_buffer[0] == DATA_TOKEN {
                    // data ready to read. Read block plus CRC
                    self.alarm_count.set(0);
                    self.state.set(SpiState::ReadBlockComplete);
                    self.read_bytes(write_buffer, read_buffer, 512 + 2);
                } else if read_buffer[0] == 0xFF {
                    // line is idling high, data is not ready

                    // replace buffers
                    self.txbuffer.replace(write_buffer);
                    self.rxbuffer.replace(read_buffer);

                    // try again after 1 ms
                    self.alarm_state.set(AlarmState::WaitForDataBlock);
                    let delay = self.alarm.ticks_from_ms(1);
                    self.alarm.set_alarm(self.alarm.now(), delay);
                } else {
                    // error, send callback and quit
                    self.txbuffer.replace(write_buffer);
                    self.rxbuffer.replace(read_buffer);
                    self.state.set(SpiState::Idle);
                    self.alarm_state.set(AlarmState::Idle);
                    self.alarm_count.set(0);
                    self.client.map(move |client| {
                        client.error(SdCardError::ReadFailure as u32);
                    });
                }
            }

            SpiState::ReadBlockComplete => {
                // replace buffers
                self.txbuffer.replace(write_buffer);
                self.rxbuffer.replace(read_buffer);

                // read finished, perform callback
                self.state.set(SpiState::Idle);
                self.rxbuffer.map(|read_buffer| {
                    self.client_buffer.take().map(move |buffer| {
                        // copy data to user buffer
                        // Limit to minimum length between buffer, read_buffer,
                        // and 512 (block size)
                        for (client_byte, &read_byte) in
                            buffer.iter_mut().zip(read_buffer.iter()).take(512)
                        {
                            *client_byte = read_byte;
                        }

                        // callback
                        let read_len = cmp::min(read_buffer.len(), cmp::min(buffer.len(), 512));
                        self.client.map(move |client| {
                            client.read_done(buffer, read_len);
                        });
                    });
                });
            }

            SpiState::WaitReadBlocks { count } => {
                if read_buffer[0] == DATA_TOKEN {
                    // data ready to read. Read block plus CRC
                    self.alarm_count.set(0);
                    self.state.set(SpiState::ReceivedBlock { count });
                    self.read_bytes(write_buffer, read_buffer, 512 + 2);
                } else if read_buffer[0] == 0xFF {
                    // line is idling high, data is not ready

                    // replace buffers
                    self.txbuffer.replace(write_buffer);
                    self.rxbuffer.replace(read_buffer);

                    // try again after 1 ms
                    self.alarm_state
                        .set(AlarmState::WaitForDataBlocks { count });
                    let delay = self.alarm.ticks_from_ms(1);
                    self.alarm.set_alarm(self.alarm.now(), delay);
                } else {
                    // error, send callback and quit
                    self.txbuffer.replace(write_buffer);
                    self.rxbuffer.replace(read_buffer);
                    self.state.set(SpiState::Idle);
                    self.alarm_state.set(AlarmState::Idle);
                    self.alarm_count.set(0);
                    self.client.map(move |client| {
                        client.error(SdCardError::ReadFailure as u32);
                    });
                }
            }

            SpiState::ReceivedBlock { count } => {
                // copy block over to client buffer
                self.client_buffer.map(|buffer| {
                    // copy block into client buffer
                    // Limit to minimum length between buffer, read_buffer, and
                    // 512 (block size)
                    let offset = self.client_offset.get();
                    for (client_byte, &read_byte) in buffer
                        .iter_mut()
                        .skip(offset)
                        .zip(read_buffer.iter())
                        .take(512)
                    {
                        *client_byte = read_byte;
                    }

                    // update offset
                    let read_len = cmp::min(read_buffer.len(), cmp::min(buffer.len(), 512));
                    self.client_offset.set(offset + read_len);
                });

                if count <= 1 {
                    // all blocks received. Terminate multiple read
                    self.state.set(SpiState::ReadBlocksComplete);
                    self.send_command(SDCmd::CMD12_StopRead, 0x0, write_buffer, read_buffer, 10);
                } else {
                    // check for next data block to be ready
                    self.state
                        .set(SpiState::WaitReadBlocks { count: count - 1 });
                    self.read_bytes(write_buffer, read_buffer, 1);
                }
            }

            SpiState::ReadBlocksComplete => {
                // check response
                let (r1, _, _) = self.get_response(SDResponse::R1_Status, read_buffer);

                if r1 == SUCCESS_STATUS {
                    // replace buffers
                    self.txbuffer.replace(write_buffer);
                    self.rxbuffer.replace(read_buffer);
                    self.state.set(SpiState::Idle);

                    // read finished, perform callback
                    self.client_buffer.take().map(move |buffer| {
                        self.client.map(move |client| {
                            client.read_done(buffer, self.client_offset.get());
                        });
                    });
                } else {
                    // error, send callback and quit
                    self.txbuffer.replace(write_buffer);
                    self.rxbuffer.replace(read_buffer);
                    self.state.set(SpiState::Idle);
                    self.alarm_state.set(AlarmState::Idle);
                    self.alarm_count.set(0);
                    self.client.map(move |client| {
                        client.error(SdCardError::ReadFailure as u32);
                    });
                }
            }

            SpiState::StartWriteBlocks { count } => {
                // check response
                let (r1, _, _) = self.get_response(SDResponse::R1_Status, read_buffer);

                if r1 == SUCCESS_STATUS {
                    if count <= 1 {
                        let bytes_written = self.client_buffer.map_or(0, |buffer| {
                            // copy over data from client buffer
                            // Limit to minimum length between write_buffer,
                            // buffer, and 512 (block size)
                            for (write_byte, &client_byte) in
                                write_buffer.iter_mut().skip(1).zip(buffer.iter()).take(512)
                            {
                                *write_byte = client_byte;
                            }

                            // calculate number of bytes written
                            cmp::min(write_buffer.len(), cmp::min(buffer.len(), 512))
                        });

                        // set a known value for remaining bytes
                        for write_byte in write_buffer
                            .iter_mut()
                            .skip(1)
                            .skip(bytes_written)
                            .take(512)
                        {
                            *write_byte = 0xFF;
                        }

                        // set up remainder of data packet
                        write_buffer[0] = DATA_TOKEN; // Data token
                        write_buffer[513] = 0xFF; // dummy CRC
                        write_buffer[514] = 0xFF; // dummy CRC

                        // write data packet
                        self.state.set(SpiState::WriteBlockResponse);
                        self.write_bytes(write_buffer, read_buffer, 515);
                    } else {
                        // multi-block SD card writes are unimplemented
                        // This should have returned an error already, but if
                        // we got here somehow, return an error and quit now
                        self.txbuffer.replace(write_buffer);
                        self.rxbuffer.replace(read_buffer);
                        self.state.set(SpiState::Idle);
                        self.alarm_state.set(AlarmState::Idle);
                        self.alarm_count.set(0);
                        self.client.map(move |client| {
                            client.error(SdCardError::WriteFailure as u32);
                        });
                    }
                } else {
                    // error, send callback and quit
                    self.txbuffer.replace(write_buffer);
                    self.rxbuffer.replace(read_buffer);
                    self.state.set(SpiState::Idle);
                    self.alarm_state.set(AlarmState::Idle);
                    self.alarm_count.set(0);
                    self.client.map(move |client| {
                        client.error(SdCardError::WriteFailure as u32);
                    });
                }
            }

            SpiState::WriteBlockResponse => {
                // Get data packet
                self.state.set(SpiState::WriteBlockBusy);
                self.read_bytes(write_buffer, read_buffer, 1);
            }

            SpiState::WriteBlockBusy => {
                if (read_buffer[0] & 0x1F) == 0x05 {
                    // check if sd card is busy
                    self.state.set(SpiState::WaitWriteBlockBusy);
                    self.read_bytes(write_buffer, read_buffer, 1);
                } else {
                    // error, send callback and quit
                    self.txbuffer.replace(write_buffer);
                    self.rxbuffer.replace(read_buffer);
                    self.state.set(SpiState::Idle);
                    self.alarm_state.set(AlarmState::Idle);
                    self.alarm_count.set(0);
                    self.client.map(move |client| {
                        client.error(SdCardError::WriteFailure as u32);
                    });
                }
            }

            SpiState::WaitWriteBlockBusy => {
                // check if line is still held low (busy state)
                if read_buffer[0] != 0x00 {
                    // replace buffers
                    self.txbuffer.replace(write_buffer);
                    self.rxbuffer.replace(read_buffer);

                    // read finished, perform callback
                    self.state.set(SpiState::Idle);
                    self.alarm_count.set(0);
                    self.client_buffer.take().map(move |buffer| {
                        self.client.map(move |client| {
                            client.write_done(buffer);
                        });
                    });
                } else {
                    // replace buffers
                    self.txbuffer.replace(write_buffer);
                    self.rxbuffer.replace(read_buffer);

                    // try again after 1 ms
                    self.alarm_state.set(AlarmState::WaitForWriteBusy);
                    let delay = self.alarm.ticks_from_ms(1);
                    self.alarm.set_alarm(self.alarm.now(), delay);
                }
            }

            SpiState::Idle => {
                // receiving an event from Idle means something was killed

                // replace buffers
                self.txbuffer.replace(write_buffer);
                self.rxbuffer.replace(read_buffer);
            }
        }
    }

    /// updates SD card state upon timer alarm fired
    fn process_alarm_states(&self) {
        // keep track of how many times the alarm has been called in a row
        let repeats = self.alarm_count.get();
        if repeats > 100 {
            // error, send callback and quit
            self.state.set(SpiState::Idle);
            self.alarm_state.set(AlarmState::Idle);
            self.alarm_count.set(0);
            self.client.map(move |client| {
                client.error(SdCardError::TimeoutFailure as u32);
            });
        } else {
            self.alarm_count.set(repeats + 1);
        }

        match self.alarm_state.get() {
            AlarmState::DetectionChange => {
                // perform callback
                self.client.map(move |client| {
                    client.card_detection_changed(self.is_installed());
                });

                // re-enable interrupts
                self.detect_changes();
                self.alarm_count.set(0);
                self.alarm_state.set(AlarmState::Idle);
            }

            AlarmState::RepeatHCSInit => {
                // check card initialization again
                self.txbuffer.take().map(|write_buffer| {
                    self.rxbuffer.take().map(move |read_buffer| {
                        // send application-specific initialization in high capcity mode (HCS)
                        self.state.set(SpiState::SendManufSpecificCmd {
                            cmd: SDCmd::ACMD41_ManufSpecificInit,
                            arg: 0x40000000,
                        });
                        self.after_state.set(SpiState::InitRepeatHCSInit);
                        self.send_command(
                            SDCmd::CMD55_ManufSpecificCommand,
                            0x0,
                            write_buffer,
                            read_buffer,
                            10,
                        );
                    });
                });

                self.alarm_state.set(AlarmState::Idle);
            }

            AlarmState::RepeatAppSpecificInit => {
                // check card initialization again
                self.txbuffer.take().map(|write_buffer| {
                    self.rxbuffer.take().map(move |read_buffer| {
                        // send application-specific initialization
                        self.state.set(SpiState::SendManufSpecificCmd {
                            cmd: SDCmd::ACMD41_ManufSpecificInit,
                            arg: 0x0,
                        });
                        self.after_state.set(SpiState::InitRepeatAppSpecificInit);
                        self.send_command(
                            SDCmd::CMD55_ManufSpecificCommand,
                            0x0,
                            write_buffer,
                            read_buffer,
                            10,
                        );
                    });
                });

                self.alarm_state.set(AlarmState::Idle);
            }

            AlarmState::RepeatGenericInit => {
                // check card initialization again
                self.txbuffer.take().map(|write_buffer| {
                    self.rxbuffer.take().map(move |read_buffer| {
                        // send generic initialization
                        self.state.set(SpiState::InitRepeatGenericInit);
                        self.send_command(SDCmd::CMD1_Init, 0x0, write_buffer, read_buffer, 10);
                    });
                });

                self.alarm_state.set(AlarmState::Idle);
            }

            AlarmState::WaitForDataBlock => {
                // check card initialization again
                self.txbuffer.take().map(|write_buffer| {
                    self.rxbuffer.take().map(move |read_buffer| {
                        // wait until ready and then read data block, then done
                        self.state.set(SpiState::WaitReadBlock);
                        self.read_bytes(write_buffer, read_buffer, 1);
                    });
                });

                self.alarm_state.set(AlarmState::Idle);
            }

            AlarmState::WaitForDataBlocks { count } => {
                // check card initialization again
                self.txbuffer.take().map(|write_buffer| {
                    self.rxbuffer.take().map(move |read_buffer| {
                        // wait until ready and then read data block, then done
                        self.state.set(SpiState::WaitReadBlocks { count });
                        self.read_bytes(write_buffer, read_buffer, 1);
                    });
                });

                self.alarm_state.set(AlarmState::Idle);
            }

            AlarmState::WaitForWriteBusy => {
                // check card initialization again
                self.txbuffer.take().map(|write_buffer| {
                    self.rxbuffer.take().map(move |read_buffer| {
                        // check if sd card is busy
                        self.state.set(SpiState::WaitWriteBlockBusy);
                        self.read_bytes(write_buffer, read_buffer, 1);
                    });
                });

                self.alarm_state.set(AlarmState::Idle);
            }

            AlarmState::Idle => {
                // receiving an event from Idle means something was killed
                // do nothing
            }
        }
    }

    pub fn set_client<C: SDCardClient>(&self, client: &'static C) {
        self.client.set(client);
    }

    pub fn is_installed(&self) -> bool {
        // if there is no detect pin, assume an sd card is installed
        self.detect_pin.get().is_none_or(|pin| {
            // sd card detection pin is active low
            !pin.read()
        })
    }

    pub fn is_initialized(&self) -> bool {
        self.is_initialized.get()
    }

    /// watches SD card detect pin for changes, sends callback on change
    pub fn detect_changes(&self) {
        self.detect_pin.get().map(|pin| {
            pin.enable_interrupts(hil::gpio::InterruptEdge::EitherEdge);
        });
    }

    pub fn initialize(&self) -> Result<(), ErrorCode> {
        // if not already, set card to uninitialized again
        self.is_initialized.set(false);

        // no point in initializing if the card is not installed
        if self.is_installed() {
            // reset the SD card in order to start initializing it
            self.txbuffer
                .take()
                .map_or(Err(ErrorCode::NOMEM), |txbuffer| {
                    self.rxbuffer
                        .take()
                        .map_or(Err(ErrorCode::NOMEM), move |rxbuffer| {
                            self.state.set(SpiState::InitReset);
                            self.send_command(SDCmd::CMD0_Reset, 0x0, txbuffer, rxbuffer, 10);

                            // command started successfully
                            Ok(())
                        })
                })
        } else {
            // no sd card installed
            Err(ErrorCode::UNINSTALLED)
        }
    }

    pub fn read_blocks(
        &self,
        buffer: &'static mut [u8],
        sector: u32,
        count: u32,
    ) -> Result<(), ErrorCode> {
        // only if initialized and installed
        if self.is_installed() {
            if self.is_initialized() {
                self.txbuffer
                    .take()
                    .map_or(Err(ErrorCode::NOMEM), |txbuffer| {
                        self.rxbuffer
                            .take()
                            .map_or(Err(ErrorCode::NOMEM), move |rxbuffer| {
                                // save the user buffer for later
                                self.client_buffer.replace(buffer);
                                self.client_offset.set(0);

                                // convert block address to byte address for non-block
                                //  access cards
                                let mut address = sector;
                                if self.card_type.get() != SDCardType::SDv2BlockAddressable {
                                    address *= 512;
                                }

                                self.state.set(SpiState::StartReadBlocks { count });
                                if count == 1 {
                                    self.send_command(
                                        SDCmd::CMD17_ReadSingle,
                                        address,
                                        txbuffer,
                                        rxbuffer,
                                        10,
                                    );
                                } else {
                                    self.send_command(
                                        SDCmd::CMD18_ReadMultiple,
                                        address,
                                        txbuffer,
                                        rxbuffer,
                                        10,
                                    );
                                }

                                // command started successfully
                                Ok(())
                            })
                    })
            } else {
                // sd card not initialized
                Err(ErrorCode::RESERVE)
            }
        } else {
            // sd card not installed
            Err(ErrorCode::UNINSTALLED)
        }
    }

    pub fn write_blocks(
        &self,
        buffer: &'static mut [u8],
        sector: u32,
        count: u32,
    ) -> Result<(), ErrorCode> {
        // only if initialized and installed
        if self.is_installed() {
            if self.is_initialized() {
                self.txbuffer
                    .take()
                    .map_or(Err(ErrorCode::NOMEM), |txbuffer| {
                        self.rxbuffer
                            .take()
                            .map_or(Err(ErrorCode::NOMEM), move |rxbuffer| {
                                // save the user buffer for later
                                self.client_buffer.replace(buffer);
                                self.client_offset.set(0);

                                // convert block address to byte address for non-block
                                //  access cards
                                let mut address = sector;
                                if self.card_type.get() != SDCardType::SDv2BlockAddressable {
                                    address *= 512;
                                }

                                self.state.set(SpiState::StartWriteBlocks { count });
                                if count == 1 {
                                    self.send_command(
                                        SDCmd::CMD24_WriteSingle,
                                        address,
                                        txbuffer,
                                        rxbuffer,
                                        10,
                                    );

                                    // command started successfully
                                    Ok(())
                                } else {
                                    // can't write multiple blocks yet
                                    Err(ErrorCode::NOSUPPORT)
                                }
                            })
                    })
            } else {
                // sd card not initialized
                Err(ErrorCode::RESERVE)
            }
        } else {
            // sd card not installed
            Err(ErrorCode::UNINSTALLED)
        }
    }
}

/// Handle callbacks from the SPI peripheral
impl<'a, A: hil::time::Alarm<'a>> hil::spi::SpiMasterClient for SDCard<'a, A> {
    fn read_write_done(
        &self,
        write_buffer: SubSliceMut<'static, u8>,
        read_buffer: Option<SubSliceMut<'static, u8>>,
        status: Result<usize, ErrorCode>,
    ) {
        // unwrap so we don't have to deal with options everywhere
        read_buffer.map(move |read_buffer| {
            self.process_spi_states(write_buffer.take(), read_buffer.take(), status.unwrap_or(0));
        });
    }
}

/// Handle callbacks from the timer
impl<'a, A: hil::time::Alarm<'a>> hil::time::AlarmClient for SDCard<'a, A> {
    fn alarm(&self) {
        self.process_alarm_states();
    }
}

/// Handle callbacks from the card detection pin
impl<'a, A: hil::time::Alarm<'a>> hil::gpio::Client for SDCard<'a, A> {
    fn fired(&self) {
        // check if there was an open transaction with the sd card
        if self.alarm_state.get() != AlarmState::Idle || self.state.get() != SpiState::Idle {
            // something was running when this occurred. Kill the transaction and
            //  send an error callback
            self.state.set(SpiState::Idle);
            self.alarm_state.set(AlarmState::Idle);
            self.client.map(move |client| {
                client.error(SdCardError::CardStateChanged as u32);
            });
        }

        // either the card is new or gone, in either case it isn't initialized
        self.is_initialized.set(false);

        // disable additional interrupts
        self.detect_pin.get().map(|pin| {
            pin.disable_interrupts();
        });

        // run a timer for 500 ms in order to let the sd card settle
        self.alarm_state.set(AlarmState::DetectionChange);
        let delay = self.alarm.ticks_from_ms(500);
        self.alarm.set_alarm(self.alarm.now(), delay);
    }
}

/// Application driver for SD Card capsule.
///
/// This is used if the SDCard is going to be attached directly to userspace
/// syscalls. SDCardDriver can be ignored if another capsule is going to build
/// off of the SDCard instead
pub struct SDCardDriver<'a, A: hil::time::Alarm<'a>> {
    sdcard: &'a SDCard<'a, A>,
    kernel_buf: TakeCell<'static, [u8]>,
    grants: Grant<
        App,
        UpcallCount<1>,
        AllowRoCount<{ ro_allow::COUNT }>,
        AllowRwCount<{ rw_allow::COUNT }>,
    >,
    current_process: OptionalCell<ProcessId>,
}

/// Holds buffers and whatnot that the application has passed us.
#[derive(Default)]
pub struct App;

/// Buffer for SD card driver, assigned in board `main.rs` files
pub const KERNEL_BUFFER_LENGTH: usize = 512;

/// Functions for SDCardDriver
impl<'a, A: hil::time::Alarm<'a>> SDCardDriver<'a, A> {
    /// Create new SD card userland interface
    ///
    /// sdcard - SDCard interface to provide application access to
    /// kernel_buf - buffer used to hold SD card blocks, must be at least 512
    ///     bytes in length
    pub fn new(
        sdcard: &'a SDCard<'a, A>,
        kernel_buf: &'static mut [u8; 512],
        grants: Grant<
            App,
            UpcallCount<1>,
            AllowRoCount<{ ro_allow::COUNT }>,
            AllowRwCount<{ rw_allow::COUNT }>,
        >,
    ) -> SDCardDriver<'a, A> {
        // return new SDCardDriver
        SDCardDriver {
            sdcard,
            kernel_buf: TakeCell::new(kernel_buf),
            grants,
            current_process: OptionalCell::empty(),
        }
    }
}

/// Handle callbacks from SDCard
impl<'a, A: hil::time::Alarm<'a>> SDCardClient for SDCardDriver<'a, A> {
    fn card_detection_changed(&self, installed: bool) {
        self.current_process.map(|process_id| {
            let _ = self.grants.enter(process_id, |_app, kernel_data| {
                kernel_data
                    .schedule_upcall(0, (0, installed as usize, 0))
                    .ok();
            });
        });
    }

    fn init_done(&self, block_size: u32, total_size: u64) {
        self.current_process.map(|process_id| {
            let _ = self.grants.enter(process_id, |_app, kernel_data| {
                let size_in_kb = ((total_size >> 10) & 0xFFFFFFFF) as usize;
                kernel_data
                    .schedule_upcall(0, (1, block_size as usize, size_in_kb))
                    .ok();
            });
        });
    }

    fn read_done(&self, data: &'static mut [u8], len: usize) {
        self.kernel_buf.replace(data);

        self.current_process.map(|process_id| {
            let _ = self.grants.enter(process_id, |_, kernel_data| {
                let mut read_len = 0;
                self.kernel_buf.map(|data| {
                    kernel_data
                        .get_readwrite_processbuffer(rw_allow::READ)
                        .and_then(|read| {
                            read.mut_enter(|read_buffer| {
                                // copy bytes to user buffer
                                // Limit to minimum length between read_buffer, data, and
                                // len field
                                for (read_byte, &data_byte) in
                                    read_buffer.iter().zip(data.iter()).take(len)
                                {
                                    read_byte.set(data_byte);
                                }
                                read_len = cmp::min(read_buffer.len(), cmp::min(data.len(), len));
                                read_len
                            })
                        })
                        .unwrap_or(0);
                });

                // perform callback
                // Note that we are explicitly performing the callback even if no
                // data was read or if the app's read_buffer doesn't exist
                kernel_data.schedule_upcall(0, (2, read_len, 0)).ok();
            });
        });
    }

    fn write_done(&self, buffer: &'static mut [u8]) {
        self.kernel_buf.replace(buffer);

        self.current_process.map(|process_id| {
            let _ = self.grants.enter(process_id, |_app, kernel_data| {
                kernel_data.schedule_upcall(0, (3, 0, 0)).ok();
            });
        });
    }

    fn error(&self, error: u32) {
        self.current_process.map(|process_id| {
            let _ = self.grants.enter(process_id, |_app, kernel_data| {
                kernel_data.schedule_upcall(0, (4, error as usize, 0)).ok();
            });
        });
    }
}

/// Connections to userspace syscalls
impl<'a, A: hil::time::Alarm<'a>> SyscallDriver for SDCardDriver<'a, A> {
    fn command(
        &self,
        command_num: usize,
        data: usize,
        _: usize,
        process_id: ProcessId,
    ) -> CommandReturn {
        if command_num == 0 {
            // Handle unconditional driver existence check.
            return CommandReturn::success();
        }

        // Check if this driver is free, or already dedicated to this process.
        let match_or_empty_or_nonexistant = self.current_process.map_or(true, |current_process| {
            self.grants
                .enter(current_process, |_, _| current_process == process_id)
                .unwrap_or(true)
        });
        if match_or_empty_or_nonexistant {
            self.current_process.set(process_id);
        } else {
            return CommandReturn::failure(ErrorCode::NOMEM);
        }

        match command_num {
            // is_installed
            1 => {
                let value = self.sdcard.is_installed() as u32;
                CommandReturn::success_u32(value)
            }

            // initialize
            2 => match self.sdcard.initialize() {
                Ok(()) => CommandReturn::success(),
                Err(e) => CommandReturn::failure(e),
            },

            // read_block
            3 => self.kernel_buf.take().map_or(
                CommandReturn::failure(ErrorCode::BUSY),
                |kernel_buf| {
                    CommandReturn::from(self.sdcard.read_blocks(kernel_buf, data as u32, 1))
                },
            ),

            // write_block
            4 => {
                let result: Result<(), ErrorCode> = self
                    .grants
                    .enter(process_id, |_, kernel_data| {
                        kernel_data
                            .get_readonly_processbuffer(ro_allow::WRITE)
                            .and_then(|write| {
                                write.enter(|write_buffer| {
                                    self.kernel_buf.take().map_or(
                                        Err(ErrorCode::BUSY),
                                        |kernel_buf| {
                                            // copy over write data from application
                                            // Limit to minimum length between kernel_buf,
                                            // write_buffer, and 512 (block size)
                                            for (kernel_byte, write_byte) in kernel_buf
                                                .iter_mut()
                                                .zip(write_buffer.iter())
                                                .take(512)
                                            {
                                                *kernel_byte = write_byte.get();
                                            }

                                            // begin writing
                                            self.sdcard.write_blocks(kernel_buf, data as u32, 1)
                                        },
                                    )
                                })
                            })
                            .unwrap_or(Err(ErrorCode::NOMEM))
                    })
                    .unwrap_or(Err(ErrorCode::NOMEM));
                CommandReturn::from(result)
            }

            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::process::Error> {
        self.grants.enter(processid, |_, _| {})
    }
}

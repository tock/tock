// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Interfaces for SPI controller (master) and peripheral (slave)
//! communication. We use the terms master/slave in some situations
//! because the term peripheral can also refer to a hardware peripheral
//! (e.g., memory-mapped I/O devices in ARM are called peripherals).

// Author: Alexandru Radovici <msg4alex@gmail.com>
// Author: Philip Levis <pal@cs.stanford.edu>
// Author: Hubert Teo <hubert.teo.hk@gmail.com>
// Author: Brad Campbell <bradjc5@gmail.com>
// Author: Amit Aryeh Levy <amit@amitlevy.com>

use crate::ErrorCode;
use core::option::Option;

/// Data order defines the order of bits sent over the wire: most
/// significant first, or least significant first.
#[derive(Copy, Clone, Debug)]
pub enum DataOrder {
    MSBFirst,
    LSBFirst,
}

/// Clock polarity (CPOL) defines whether the SPI clock is high
/// or low when idle.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ClockPolarity {
    IdleLow,
    IdleHigh,
}

/// Clock phase (CPHA) defines whether to sample and send data on
/// a leading or trailing clock edge; consult a SPI reference
/// on how CPHA interacts with CPOL.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ClockPhase {
    SampleLeading,
    SampleTrailing,
}

/// Trait for clients of a SPI bus in master mode.
pub trait SpiMasterClient {
    /// Callback when a read/write operation finishes: `read_buffer`
    /// is an `Option` because the call passes an `Option` (with
    /// `None` if it's a write-only operation.
    fn read_write_done(
        &self,
        write_buffer: &'static mut [u8],
        read_buffer: Option<&'static mut [u8]>,
        len: usize,
        status: Result<(), ErrorCode>,
    );
}
/// The `SpiMaster` trait for interacting with SPI slave
/// devices at a byte or buffer level.
///
/// Using SpiMaster normally involves three steps:
///
/// 1. Configure the SPI bus for a peripheral
///    1a. Call set_chip_select to select which peripheral and
///        turn on SPI
///    1b. Call set operations as needed to configure bus
///    NOTE: You MUST select the chip select BEFORE configuring
///           SPI settings.
/// 2. Invoke read, write, read_write on SpiMaster
/// 3a. Call clear_chip_select to turn off bus, or
/// 3b. Call set_chip_select to choose another peripheral,
///     go to step 1b or 2.
///
/// The SPI configuration for a particular peripheral persists across
/// changes to the chip select. For example, this set of calls
///
///   specify_chip_select(1);
///   set_phase(SampleLeading);
///   specify_chip_select(2);
///   set_phase(SampleTrailing);
///   specify_chip_select(1);
///   write_byte(0); // Uses SampleLeading
///
/// will have a SampleLeading phase in the final `write_byte` call,
/// because the configuration of chip select 1 is saved, and restored
/// when chip select is set back to 1.
///
/// If additional chip selects are needed, they can be performed
/// with GPIO and manual re-initialization of settings. Note that
/// a SPI chip select (CS) line is usually active low.
///
///   specify_chip_select(0);
///   set_phase(SampleLeading);
///   pin_a.clear(); // Select A
///   write_byte(0xaa); // Uses SampleLeading
///   pin_a.set(); // Unselect A
///   set_phase(SampleTrailing);
///   pin_b.clear(); // Select B
///   write_byte(0xaa); // Uses SampleTrailing
///
pub trait SpiMaster {
    /// Chip select is an associated type because different SPI
    /// buses may have different numbers of chip selects. This
    /// allows peripheral implementations to define their own type.
    type ChipSelect: Copy;

    /// Initialize this SPI interface. Call this once before
    /// invoking any other operations. Return values are:
    ///   - Ok(()): initialized correctly
    ///   - Err(OFF): not currently powered so can't be initialized
    ///   - Err(RESERVE): no clock is configured yet
    ///   - Err(FAIL): other failure condition
    fn init(&self) -> Result<(), ErrorCode>;

    /// Change the callback handler for `read_write_bytes`
    /// calls.
    fn set_client(&self, client: &'static dyn SpiMasterClient);

    /// Return whether the SPI peripheral is busy with `read_write_bytes`
    /// call.
    fn is_busy(&self) -> bool;

    /// Perform an asynchronous read/write operation, whose
    /// completion is signaled by invoking SpiMasterClient on
    /// the client. Write-only operations may pass `None` for
    /// `read_buffer`, while read-write operations pass `Some`
    /// for `read_buffer`.
    ///
    /// If `read_buffer` is `None`, the
    /// number of bytes written will be the mimumum of the length of
    /// `write_buffer` and the `len` argument. If `read_buffer`
    /// is `Some`, the number of bytes read/written will be the
    /// minimum of the `len` argument, the length of `write_buffer`,
    /// and the length of `read_buffer`.
    ///
    /// If `read_write_bytes` returns `Ok(())`, the operation will be
    /// attempted and a callback will be called. If it returns `Err`,
    /// no callback will be called and the buffers are returned.
    ///   - Ok(()): the operation will be attempted and the callback will
    ///     be called.
    ///   - Err(OFF): the SPI bus is powered down.
    ///   - Err(INVAL): length is 0
    ///   - Err(BUSY): the SPI bus is busy with a prior `read_write_bytes`
    ///     operation whose callback hasn't been called yet.
    fn read_write_bytes(
        &self,
        write_buffer: &'static mut [u8],
        read_buffer: Option<&'static mut [u8]>,
        len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8], Option<&'static mut [u8]>)>;

    /// Synchronously write a single byte on the bus. Not for general
    /// use because it is blocking: intended for debugging.
    /// Return values:
    ///   - Ok(()): the byte was written
    ///   - Err(OFF): the SPI bus is powered down
    ///   - Err(BUSY): the SPI bus is busy with a `read_write_bytes`
    ///     operation whose callback hasn't been called yet.
    ///   - Err(FAIL): other failure
    fn write_byte(&self, val: u8) -> Result<(), ErrorCode>;

    /// Synchronously write a 0 and read a single byte from the bus.
    /// Not for general use because it is blocking: intended for debugging.
    /// Return values:
    ///   - Ok((u8)): the read byte
    ///   - Err(OFF): the SPI bus is powered down
    ///   - Err(BUSY): the SPI bus is busy with a `read_write_bytes`
    ///     operation whose callback hasn't been called yet.
    ///   - Err(FAIL): other failure
    fn read_byte(&self) -> Result<u8, ErrorCode>;

    /// Synchronously write and read a single byte.
    /// Not for general use because it is blocking: intended for debugging.
    /// Return values:
    ///   - Ok((u8)): the read byte
    ///   - Err(OFF): the SPI bus is powered down
    ///   - Err(BUSY): the SPI bus is busy with a `read_write_bytes`
    ///     operation whose callback hasn't been called yet.
    ///   - Err(FAIL): other failure
    fn read_write_byte(&self, val: u8) -> Result<u8, ErrorCode>;

    /// Specify which chip select to use. Configuration settings
    /// (rate, polarity, phase) are chip-select specific and are
    /// stored for that chip select.
    fn specify_chip_select(&self, cs: Self::ChipSelect) -> Result<(), ErrorCode>;

    /// Set the clock/data rate for the current chip select. Return values:
    ///   - Ok(u32): the actual data rate set (limited by clock precision)
    ///   - Err(INVAL): a rate outside the bounds of the bus was passed
    ///   - Err(BUSY): the SPI bus is busy with a read_write_bytes
    ///     operation whose callback hasn't been called yet.
    ///   - Err(FAIL): other failure
    fn set_rate(&self, rate: u32) -> Result<u32, ErrorCode>;
    /// Return the current chip select's clock rate.
    fn get_rate(&self) -> u32;

    /// Set the bus polarity (whether idle is high or low) for the
    /// current chip select. Return values:
    ///   - Ok(()): the polarity was set.
    ///   - Err(BUSY): the SPI bus is busy with a `read_write_bytes`
    ///     operation whose callback hasn't been called yet.
    ///   - Err(FAIL): other failure
    fn set_polarity(&self, polarity: ClockPolarity) -> Result<(), ErrorCode>;
    /// Return the current bus polarity.
    fn get_polarity(&self) -> ClockPolarity;

    /// Set the bus phase for the current chip select (whether data is
    /// sent/received on leading or trailing edges).
    ///   - Ok(()): the phase was set.
    ///   - Err(BUSY): the SPI bus is busy with a `read_write_bytes`
    ///     operation whose callback hasn't been called yet.
    ///   - Err(FAIL): other failure
    fn set_phase(&self, phase: ClockPhase) -> Result<(), ErrorCode>;

    /// Get the current bus phase for the current chip select.
    fn get_phase(&self) -> ClockPhase;

    // These two functions determine what happens to the chip
    // select line between transfers. If hold_low() is called,
    // then the chip select line is held low after transfers
    // complete. If release_low() is called, then the chip select
    // line is brought high after a transfer completes. A "transfer"
    // is any of the read/read_write calls. These functions
    // allow an application to manually control when the
    // CS line is high or low, such that it can issue longer
    // read/writes with multiple read_write_bytes calls.

    /// Hold the chip select line low after a read_write_bytes completes.
    /// This allows a client to make one long SPI read/write with
    /// multiple calls to `read_write_bytes`.
    fn hold_low(&self);
    /// Raise the chip select line after a read_write_bytes completes.
    /// This will complete the SPI operation.
    fn release_low(&self);
}

/// SPIMasterDevice provides a chip-select-specific interface to the SPI
/// Master hardware, such that a client cannot changethe chip select line.
pub trait SpiMasterDevice {
    /// Set the callback for read_write operations.
    fn set_client(&self, client: &'static dyn SpiMasterClient);

    /// Configure the bus for this chip select.
    fn configure(&self, cpol: ClockPolarity, cpal: ClockPhase, rate: u32) -> Result<(), ErrorCode>;

    /// Perform an asynchronous read/write operation, whose
    /// completion is signaled by invoking SpiMasterClient on
    /// the client. Write-only operations may pass `None` for
    /// `read_buffer`, while read-write operations pass `Some`
    /// for `read_buffer`.
    ///
    /// If `read_buffer` is `None`, the
    /// number of bytes written will be the mimumum of the length of
    /// `write_buffer` and the `len` argument. If `read_buffer`
    /// is `Some`, the number of bytes read/written will be the
    /// minimum of the `len` argument, the length of `write_buffer`,
    /// and the length of `read_buffer`.
    ///
    /// If `read_write_bytes` returns `Ok(())`, the operation will be
    /// attempted and a callback will be called. If it returns `Err`,
    /// no callback will be called and the buffers are returned.
    ///   - Ok(()): the operation will be attempted and the callback will
    ///     be called.
    ///   - Err(OFF): the SPI bus is powered down.
    ///   - Err(INVAL): length is 0
    ///   - Err(BUSY): the SPI bus is busy with a prior `read_write_bytes`
    ///     operation whose callback hasn't been called yet.
    fn read_write_bytes(
        &self,
        write_buffer: &'static mut [u8],
        read_buffer: Option<&'static mut [u8]>,
        len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8], Option<&'static mut [u8]>)>;

    /// Set the clock/data rate for this chip select. Return values:
    ///   - Ok(): set successfully. Note actual rate may differ, check with get_rate.
    ///   - Err(INVAL): a rate outside the bounds of the bus was passed
    ///   - Err(BUSY): the SPI bus is busy with a read_write_bytes
    ///     operation whose callback hasn't been called yet.
    ///   - Err(FAIL): other failure
    fn set_rate(&self, rate: u32) -> Result<(), ErrorCode>;
    /// Return the current chip select's clock rate.
    fn get_rate(&self) -> u32;

    /// Set the bus polarity (whether idle is high or low) for this
    /// chip select. Return values:
    ///   - Ok(()): the polarity was set.
    ///   - Err(BUSY): the SPI bus is busy with a `read_write_bytes`
    ///     operation whose callback hasn't been called yet.
    ///   - Err(FAIL): other failure
    fn set_polarity(&self, polarity: ClockPolarity) -> Result<(), ErrorCode>;
    /// Return the current bus polarity.
    fn get_polarity(&self) -> ClockPolarity;

    /// Set the bus phase for this chip select (whether data is
    /// sent/received on leading or trailing edges).
    ///   - Ok(()): the phase was set.
    ///   - Err(BUSY): the SPI bus is busy with a `read_write_bytes`
    ///     operation whose callback hasn't been called yet.
    ///   - Err(FAIL): other failure
    fn set_phase(&self, phase: ClockPhase) -> Result<(), ErrorCode>;

    /// Get the current bus phase for the current chip select.
    fn get_phase(&self) -> ClockPhase;
}

/// Trait for SPI peripherals (slaves) to receive callbacks when the
/// corresponding controller (master) issues operations. A SPI operation
/// begins with a callback of `chip_selected`. If the client has
/// provided buffers with `SpiSlave::read_write_bytes`, these buffers
/// are written from and read into until the operation completes or one
/// of them fills, at which point a `SpiSlaveClient::read_write_done`
/// callback is called. If the client needs to read/write more it
/// can call `SpiSlave::read_write_bytes` again. Note that there is
/// no notification when the chip select line goes high.
pub trait SpiSlaveClient {
    /// Notification that the chip select has been brought low.
    fn chip_selected(&self);

    /// Callback issued when the controller completes an SPI operation
    /// to this peripheral. `write_buffer` and `read_buffer` are
    /// the values passed in the previous call to
    /// `SpiSlave::read_write_bytes`. The `len` parameter specifies
    /// how many bytes were written from/read into `Some` values of
    /// these buffers. `len` may be shorter than the size of these
    /// buffers if the operation did not fill them.
    fn read_write_done(
        &self,
        write_buffer: Option<&'static mut [u8]>,
        read_buffer: Option<&'static mut [u8]>,
        len: usize,
        status: Result<(), ErrorCode>,
    );
}

/// Trait for SPI peripherals (slaves) to exchange data with a contoller
/// (master). This is a low-level trait typically implemented by hardware:
/// higher level software typically uses the `SpiSlaveDevice` trait,
/// which is provided by a virtualizing/multiplexing layer.
pub trait SpiSlave {
    /// Initialize the SPI device to be in peripheral mode.
    /// Return values:
    ///   - Ok(()): the device is in peripheral mode
    ///   - Err(BUSY): the device is busy with an operation and cannot
    ///     be initialized
    ///   - Err(FAIL): other failure condition
    fn init(&self) -> Result<(), ErrorCode>;

    /// Returns true if there is a client. Useful for verifying that
    /// two software drivers do not both try to take control of the
    /// device.
    fn has_client(&self) -> bool;

    /// Set the callback for slave operations, passing `None` to
    /// disable peripheral mode.
    fn set_client(&self, client: Option<&'static dyn SpiSlaveClient>);

    /// Set a single byte to write in response to a read/write
    /// operation from a controller. Useful for devices that always
    /// send a status code in their first byte.
    fn set_write_byte(&self, write_byte: u8);

    /// Provide buffers for the peripheral to write from and read
    /// into when a controller performs a `read_write_bytes` operation.
    /// The device will issue a callback when one of four things occurs:
    ///   - The controller completes the operation by bringing the chip
    ///     select high.
    ///   - A `Some` write buffer is written.
    ///   - A `Some` read buffer is filled.
    ///   - `len` bytes are read/written
    /// Return values:
    ///   - Ok(()): the SPI bus will read/write the provided buffers on
    ///     the next SPI operation requested by the controller.
    ///   - Err(BUSY): the device is busy with an existing
    ///     `read_write_bytes` operation.
    ///   - Err(INVAL): the `len` parameter is 0
    ///
    /// `Err` return values return the passed buffer `Option`s.
    fn read_write_bytes(
        &self,
        write_buffer: Option<&'static mut [u8]>,
        read_buffer: Option<&'static mut [u8]>,
        len: usize,
    ) -> Result<
        (),
        (
            ErrorCode,
            Option<&'static mut [u8]>,
            Option<&'static mut [u8]>,
        ),
    >;
    /// Set the bus polarity (whether idle is high or low). Return values:
    ///   - Ok(()): the polarity was set.
    ///   - Err(BUSY): the SPI bus is busy with a `read_write_bytes`
    ///     operation whose callback hasn't been called yet.
    ///   - Err(FAIL): other failure
    fn set_polarity(&self, polarity: ClockPolarity) -> Result<(), ErrorCode>;
    /// Return the current bus polarity.
    fn get_polarity(&self) -> ClockPolarity;

    /// Set the bus phase (whether data is sent/received on leading or
    /// trailing edges).
    ///   - Ok(()): the phase was set.
    ///   - Err(BUSY): the SPI bus is busy with a `read_write_bytes`
    ///     operation whose callback hasn't been called yet.
    ///   - Err(FAIL): other failure
    fn set_phase(&self, phase: ClockPhase) -> Result<(), ErrorCode>;

    /// Return the current bus phase.
    fn get_phase(&self) -> ClockPhase;
}

/// SPISlaveDevice is an interface to a SPI bus in peripheral mode.
/// It is the standard trait used by services within the kernel:
/// `SpiSlave` is for lower-level access responsible for initializing
/// hardware.
pub trait SpiSlaveDevice {
    /// Specify the callback of `read_write_bytes` operations:
    fn set_client(&self, client: &'static dyn SpiSlaveClient);

    /// Setup the SPI settings and speed of the bus.
    fn configure(&self, cpol: ClockPolarity, cpal: ClockPhase) -> Result<(), ErrorCode>;

    /// Provide buffers for the peripheral to write from and read
    /// into when a controller performs a `read_write_bytes` operation.
    /// The device will issue a callback when one of four things occurs:
    ///   - The controller completes the operation by bringing the chip
    ///     select high.
    ///   - A `Some` write buffer is written.
    ///   - A `Some` read buffer is filled.
    ///   - `len` bytes are read/written
    /// Return values:
    ///   - Ok(()): the SPI bus will read/write the provided buffers on
    ///     the next SPI operation requested by the controller.
    ///   - Err(BUSY): the device is busy with an existing
    ///     `read_write_bytes` operation.
    ///   - Err(INVAL): the `len` parameter is 0
    ///
    /// `Err` return values return the passed buffer `Option`s.
    fn read_write_bytes(
        &self,
        write_buffer: Option<&'static mut [u8]>,
        read_buffer: Option<&'static mut [u8]>,
        len: usize,
    ) -> Result<
        (),
        (
            ErrorCode,
            Option<&'static mut [u8]>,
            Option<&'static mut [u8]>,
        ),
    >;
    /// Set the bus polarity (whether idle is high or low). Return values:
    ///   - Ok(()): the polarity was set.
    ///   - Err(BUSY): the SPI bus is busy with a `read_write_bytes`
    ///     operation whose callback hasn't been called yet.
    ///   - Err(FAIL): other failure
    fn set_polarity(&self, polarity: ClockPolarity) -> Result<(), ErrorCode>;
    /// Return the current bus polarity.
    fn get_polarity(&self) -> ClockPolarity;

    /// Set the bus phase (whether data is sent/received on leading or
    /// trailing edges).
    ///   - Ok(()): the phase was set.
    ///   - Err(BUSY): the SPI bus is busy with a `read_write_bytes`
    ///     operation whose callback hasn't been called yet.
    ///   - Err(FAIL): other failure
    fn set_phase(&self, phase: ClockPhase) -> Result<(), ErrorCode>;

    /// Return the current bus phase.
    fn get_phase(&self) -> ClockPhase;
}

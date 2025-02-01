// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Interfaces for SPI controller (master) and peripheral (slave) communication.
//!
//! We use the terms master/slave in some situations because the term peripheral
//! can also refer to a hardware peripheral (e.g., memory-mapped I/O devices in
//! ARM are called peripherals).

// Author: Alexandru Radovici <msg4alex@gmail.com>
// Author: Philip Levis <pal@cs.stanford.edu>
// Author: Hubert Teo <hubert.teo.hk@gmail.com>
// Author: Brad Campbell <bradjc5@gmail.com>
// Author: Amit Aryeh Levy <amit@amitlevy.com>

use crate::{utilities::leasable_buffer::SubSliceMut, ErrorCode};

/// Data order defines the order of bits sent over the wire: most significant
/// first, or least significant first.
#[derive(Copy, Clone, Debug)]
pub enum DataOrder {
    /// Send the most significant byte first.
    MSBFirst,
    /// Send the least significant byte first.
    LSBFirst,
}

/// Clock polarity (CPOL) defines whether the SPI clock is high or low when
/// idle.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ClockPolarity {
    /// The clock is low when the SPI bus is not active. This is CPOL = 0.
    IdleLow,
    /// The clock is high when the SPI bus is not active. This is CPOL = 1.
    IdleHigh,
}

/// Clock phase (CPHA) defines whether to sample and send data on a leading or
/// trailing clock edge.
///
/// Consult a SPI reference on how CPHA interacts with CPOL.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ClockPhase {
    /// Sample on the leading clock edge. This is CPHA = 0. If CPOL is 0, then
    /// this samples on the rising edge of the clock. If CPOL is 1, then this
    /// samples on the falling edge of the clock.
    SampleLeading,
    /// Sample on the trailing clock edge. This is CPHA = 1. If CPOL is 0, then
    /// this samples on the falling edge of the clock. If CPOL is 1, then this
    /// samples on the rising edge of the clock.
    SampleTrailing,
}

/// Utility types for modeling chip select pins in a [`SpiMaster`]
/// implementation.
pub mod cs {

    /// Represents the Polarity of a chip-select pin (i.e. whether high or low
    /// indicates the peripheral is active).
    #[derive(Copy, Clone)]
    pub enum Polarity {
        /// Chip select is active high.
        High,
        /// Chip select is active low.
        Low,
    }

    mod private {
        pub trait Sealed {}
    }

    /// Marker trait indicating whether a peripheral requires active high or low
    /// polarity as well as whether a [`SpiMaster`](super::SpiMaster)
    /// implementation can support either or both polarities.
    ///
    /// This trait is sealed and only implemented for [`ActiveLow`] and
    /// [`ActiveHigh`].
    pub trait ChipSelectActivePolarity: private::Sealed {
        const POLARITY: Polarity;
    }

    /// Marks a peripheral as requiring or controller as supporting active low
    /// chip select pins.
    pub enum ActiveLow {}
    /// Marks a peripheral as requiring or controller as supporting active high
    /// chip select pins.
    pub enum ActiveHigh {}

    impl private::Sealed for ActiveLow {}
    impl private::Sealed for ActiveHigh {}

    impl ChipSelectActivePolarity for ActiveLow {
        const POLARITY: Polarity = Polarity::Low;
    }

    impl ChipSelectActivePolarity for ActiveHigh {
        const POLARITY: Polarity = Polarity::High;
    }

    /// A type that can be converted to the appropriate type for
    /// [`SpiMaster::ChipSelect`](super::SpiMaster::ChipSelect) for a particular
    /// `POLARITY`.
    ///
    /// Instantiating a driver for any SPI peripheral should require a type that
    /// implements [`IntoChipSelect`]. That enforces that whatever object is
    /// used as the chip select can support the correct polarity for the
    /// particular SPI peripheral. This is mostly commonly handled by the
    /// component for the peripheral, which requires an object with type
    /// [`IntoChipSelect`] and then converts the object to the
    /// [`SpiMaster::ChipSelect`](super::SpiMaster::ChipSelect) type.
    ///
    /// # Examples:
    ///
    /// Some SPI host controllers only support active low or active high chip
    /// select pins. Such a controller might provide a unit implementation of
    /// this trait _only_ for the [`ActiveLow`] marker.
    ///
    /// ```rust
    /// use kernel::hil::spi::cs::*;
    ///
    /// #[derive(Copy, Clone)]
    /// enum PeripheralSelect {
    ///     Peripheral0,
    ///     Peripheral1,
    /// }
    ///
    /// impl IntoChipSelect<PeripheralSelect, ActiveLow> for PeripheralSelect {
    ///     fn into_cs(self) -> Self { self }
    /// }
    /// ```
    ///
    /// Many other controllers can handle both active low and active high chip
    /// select pins, in which case, they should implement both the [`ActiveLow`]
    /// and [`ActiveHigh`] variants, for example, using the [`ChipSelectPolar`]
    /// wrapper struct (which implements both).
    pub trait IntoChipSelect<T, POLARITY> {
        fn into_cs(self) -> T;
    }

    /// A convenience wrapper type around [`Output`](crate::hil::gpio::Output)
    /// GPIO pins that implements [`IntoChipSelect`] for both [`ActiveLow`] and
    /// [`ActiveHigh`].
    pub struct ChipSelectPolar<'a, P: crate::hil::gpio::Output> {
        /// The underlying chip select "pin".
        pub pin: &'a P,
        /// The polarity from which this wrapper was derived using
        /// [`IntoChipSelect`].
        pub polarity: Polarity,
    }

    impl<P: crate::hil::gpio::Output> Clone for ChipSelectPolar<'_, P> {
        fn clone(&self) -> Self {
            *self
        }
    }

    impl<P: crate::hil::gpio::Output> Copy for ChipSelectPolar<'_, P> {}

    impl<'a, P: crate::hil::gpio::Output, A: ChipSelectActivePolarity>
        IntoChipSelect<ChipSelectPolar<'a, P>, A> for &'a P
    {
        fn into_cs(self) -> ChipSelectPolar<'a, P> {
            ChipSelectPolar {
                pin: self,
                polarity: A::POLARITY,
            }
        }
    }

    /// When wrapping a GPIO pin that implements
    /// [`gpio::Output`](crate::hil::gpio::Output), users can use the
    /// [`activate`](ChipSelectPolar::activate) and
    /// [`deactivate`](ChipSelectPolar::deactivate) methods to automatically set
    /// or clear the chip select pin based on the stored polarity.
    impl<P: crate::hil::gpio::Output> ChipSelectPolar<'_, P> {
        /// Deactivate the chip select pin.
        ///
        /// High if active low, low if active high.
        pub fn deactivate(&self) {
            match self.polarity {
                Polarity::Low => self.pin.set(),
                Polarity::High => self.pin.clear(),
            }
        }

        /// Active the chip select pin.
        ///
        /// Low if active low, high if active high.
        pub fn activate(&self) {
            match self.polarity {
                Polarity::Low => self.pin.clear(),
                Polarity::High => self.pin.set(),
            }
        }
    }
}

/// Trait for clients of a SPI bus in master mode.
pub trait SpiMasterClient {
    /// Callback issued when a read/write operation finishes.
    ///
    /// `write_buffer` and `read_buffer` always contain the buffers
    /// passed to the [SpiMaster::read_write_bytes]
    /// down-call, with `read_buffer` as an `Option` because the
    /// down-call passes an `Option`. The contents of `write_buffer`
    /// is unmodified, while `read_buffer` contains the bytes read
    /// over SPI. Each buffer's bounds are unmodified from their state
    /// when `read_write_bytes` is called.
    ///
    /// `status` signals if the operation was successful, and if so,
    /// the length of the operation, or an appropriate `ErrorCode`.
    fn read_write_done(
        &self,
        write_buffer: SubSliceMut<'static, u8>,
        read_buffer: Option<SubSliceMut<'static, u8>>,
        status: Result<usize, ErrorCode>,
    );
}
/// Trait for interacting with SPI peripheral devices at a byte or buffer level.
///
/// Using [`SpiMaster`] normally involves three steps:
///
/// 1. Configure the SPI bus for a peripheral.
///
///    1. Call [`SpiMaster::specify_chip_select`] to select which peripheral and
///       turn on SPI.
///
///    2. Call set operations as needed to configure the bus. **NOTE**: You MUST
///       select the chip select BEFORE configuring SPI settings.
///
/// 2. Invoke [`SpiMaster::read_write_bytes`] on [`SpiMaster`].
///
/// 3. Go back to step 2 to complete another transaction, or call
///    [`SpiMaster::specify_chip_select`] to choose another peripheral and go to
///    step 1.2 or 2.
///
/// The SPI configuration for a particular peripheral persists across changes to
/// the chip select. For example, this set of calls:
///
/// ```rust,ignore
/// SpiMaster::specify_chip_select(1);
/// SpiMaster::set_phase(ClockPhase::SampleLeading);
/// SpiMaster::specify_chip_select(2);
/// SpiMaster::set_phase(ClockPhase::SampleTrailing);
/// SpiMaster::specify_chip_select(1);
/// SpiMaster::write_byte(0); // Uses SampleLeading
/// ```
///
/// will have a [`ClockPhase::SampleLeading`] phase in the final
/// [`SpiMaster::write_byte`] call, because the configuration of chip select 1
/// is saved, and restored when chip select is set back to 1.
///
/// If additional chip selects are needed, they can be performed with GPIO and
/// manual re-initialization of settings. Note that a SPI chip select (CS) line
/// is usually active low.
///
/// ```rust,ignore
/// specify_chip_select(0);
/// set_phase(ClockPhase::SampleLeading);
/// pin_a.clear(); // Select A
/// write_byte(0xaa); // Uses SampleLeading
/// pin_a.set(); // Unselect A
/// set_phase(ClockPhase::SampleTrailing);
/// pin_b.clear(); // Select B
/// write_byte(0xaa); // Uses SampleTrailing
/// ```
pub trait SpiMaster<'a> {
    /// Chip select is an associated type because different SPI buses may have
    /// different numbers of chip selects. This allows peripheral
    /// implementations to define their own type.
    type ChipSelect: Copy;

    /// Initialize this SPI interface. Call this once before invoking any other
    /// operations.
    ///
    /// ### Return values
    ///
    /// - `Ok(())`: initialized correctly
    /// - `Err(OFF)`: not currently powered so can't be initialized
    /// - `Err(RESERVE)`: no clock is configured yet
    /// - `Err(FAIL)`: other failure condition
    fn init(&self) -> Result<(), ErrorCode>;

    /// Change the callback handler for [`SpiMaster::read_write_bytes`] calls.
    fn set_client(&self, client: &'a dyn SpiMasterClient);

    /// Return whether the SPI peripheral is busy with a
    /// [`SpiMaster::read_write_bytes`] operation.
    fn is_busy(&self) -> bool;

    /// Perform an asynchronous read/write operation, whose completion is
    /// signaled by invoking [`SpiMasterClient`] on the client. Write-only
    /// operations may pass `None` for `read_buffer`, while read-write
    /// operations pass `Some` for `read_buffer`.
    ///
    /// If `read_buffer` is `None`, the number of bytes written will
    /// be the the length of `write_buffer`. If `read_buffer` is
    /// `Some`, the number of bytes read/written will be the minimum
    /// of the length of `write_buffer` and the length of
    /// `read_buffer`.
    ///
    /// ### Return values
    ///
    /// If `read_write_bytes` returns `Ok(())`, the operation will be
    /// attempted and a callback will be called. If it returns `Err`,
    /// no callback will be called and the buffers are returned.
    /// - `Ok(())`: the operation will be attempted and the callback will be
    ///   called.
    /// - `Err(OFF)`: the SPI bus is powered down.
    /// - `Err(INVAL)`: length is 0
    /// - `Err(BUSY)`: the SPI bus is busy with a prior
    ///   [`SpiMaster::read_write_bytes`] operation whose callback hasn't been
    ///   called yet.
    fn read_write_bytes(
        &self,
        write_buffer: SubSliceMut<'static, u8>,
        read_buffer: Option<SubSliceMut<'static, u8>>,
    ) -> Result<
        (),
        (
            ErrorCode,
            SubSliceMut<'static, u8>,
            Option<SubSliceMut<'static, u8>>,
        ),
    >;

    /// Synchronously write a single byte on the bus. Not for general use
    /// because it is blocking: intended for debugging.
    ///
    /// ### Return values
    ///
    /// - `Ok(())`: the byte was written
    /// - `Err(OFF)`: the SPI bus is powered down
    /// - `Err(BUSY)`: the SPI bus is busy with a
    ///   [`SpiMaster::read_write_bytes`] operation whose callback hasn't been
    ///   called yet.
    /// - `Err(FAIL)`: other failure
    fn write_byte(&self, val: u8) -> Result<(), ErrorCode>;

    /// Synchronously write a 0 and read a single byte from the bus. Not for
    /// general use because it is blocking: intended for debugging.
    ///
    /// ### Return values
    ///
    /// - `Ok(u8)`: the read byte
    /// - `Err(OFF)`: the SPI bus is powered down
    /// - `Err(BUSY)`: the SPI bus is busy with a
    ///   [`SpiMaster::read_write_bytes`] operation whose callback hasn't been
    ///   called yet.
    /// - `Err(FAIL)`: other failure
    fn read_byte(&self) -> Result<u8, ErrorCode>;

    /// Synchronously write and read a single byte. Not for general use because
    /// it is blocking: intended for debugging.
    ///
    /// ### Return values
    ///
    /// - `Ok(u8)`: the read byte
    /// - `Err(OFF)`: the SPI bus is powered down
    /// - `Err(BUSY)`: the SPI bus is busy with a
    ///   [`SpiMaster::read_write_bytes`] operation whose callback hasn't been
    ///   called yet.
    /// - `Err(FAIL)`: other failure
    fn read_write_byte(&self, val: u8) -> Result<u8, ErrorCode>;

    /// Specify which chip select to use. Configuration settings (rate,
    /// polarity, phase) are chip-select specific and are stored for that chip
    /// select.
    fn specify_chip_select(&self, cs: Self::ChipSelect) -> Result<(), ErrorCode>;

    /// Set the clock/data rate for the current chip select.
    ///
    /// ### Return values
    ///
    /// - `Ok(u32)`: the actual data rate set (limited by clock precision)
    /// - `Err(INVAL)`: a rate outside the bounds of the bus was passed
    /// - `Err(BUSY)`: the SPI bus is busy with a
    ///   [`SpiMaster::read_write_bytes`] operation whose callback hasn't been
    ///   called yet.
    /// - `Err(FAIL)`: other failure
    fn set_rate(&self, rate: u32) -> Result<u32, ErrorCode>;

    /// Return the current chip select's clock rate.
    fn get_rate(&self) -> u32;

    /// Set the bus polarity (whether idle is high or low) for the
    /// current chip select.
    ///
    /// ### Return values
    ///
    /// - `Ok(())`: the polarity was set.
    /// - `Err(BUSY)`: the SPI bus is busy with a
    ///   [`SpiMaster::read_write_bytes`] operation whose callback hasn't been
    ///   called yet.
    /// - `Err(FAIL)`: other failure
    fn set_polarity(&self, polarity: ClockPolarity) -> Result<(), ErrorCode>;

    /// Return the current bus polarity.
    fn get_polarity(&self) -> ClockPolarity;

    /// Set the bus phase for the current chip select (whether data is
    /// sent/received on leading or trailing edges).
    ///
    /// ### Return values
    ///
    /// - `Ok(())`: the phase was set.
    /// - `Err(BUSY)`: the SPI bus is busy with a
    ///   [`SpiMaster::read_write_bytes`] operation whose callback hasn't been
    ///   called yet.
    /// - `Err(FAIL)`: other failure
    fn set_phase(&self, phase: ClockPhase) -> Result<(), ErrorCode>;

    /// Get the current bus phase for the current chip select.
    fn get_phase(&self) -> ClockPhase;

    // These two functions determine what happens to the chip select line
    // between transfers. If hold_low() is called, then the chip select line is
    // held low after transfers complete. If release_low() is called, then the
    // chip select line is brought high after a transfer completes. A "transfer"
    // is any of the read/read_write calls. These functions allow an application
    // to manually control when the CS line is high or low, such that it can
    // issue longer read/writes with multiple read_write_bytes calls.

    /// Hold the chip select line low after a [`SpiMaster::read_write_bytes`]
    /// completes. This allows a client to make one long SPI read/write with
    /// multiple calls to `read_write_bytes`.
    fn hold_low(&self);

    /// Raise the chip select line after a [`SpiMaster::read_write_bytes`]
    /// completes. This will complete the SPI operation.
    fn release_low(&self);
}

/// A chip-select-specific interface to the SPI Controller hardware, such that a
/// client cannot change the chip select line.
///
/// This restricts the SPI peripherals the client can access to a specific
/// peripheral.
pub trait SpiMasterDevice<'a> {
    /// Set the callback for read_write operations.
    fn set_client(&self, client: &'a dyn SpiMasterClient);

    /// Configure the bus for this chip select.
    fn configure(&self, cpol: ClockPolarity, cpal: ClockPhase, rate: u32) -> Result<(), ErrorCode>;

    /// Same as [`SpiMaster::read_write_bytes`].
    fn read_write_bytes(
        &self,
        write_buffer: SubSliceMut<'static, u8>,
        read_buffer: Option<SubSliceMut<'static, u8>>,
    ) -> Result<
        (),
        (
            ErrorCode,
            SubSliceMut<'static, u8>,
            Option<SubSliceMut<'static, u8>>,
        ),
    >;

    /// Same as [`SpiMaster::set_rate`].
    fn set_rate(&self, rate: u32) -> Result<(), ErrorCode>;

    /// Return the current chip select's clock rate.
    fn get_rate(&self) -> u32;

    /// Same as [`SpiMaster::set_polarity`].
    fn set_polarity(&self, polarity: ClockPolarity) -> Result<(), ErrorCode>;

    /// Return the current bus polarity.
    fn get_polarity(&self) -> ClockPolarity;

    /// Same as [`SpiMaster::set_phase`].
    fn set_phase(&self, phase: ClockPhase) -> Result<(), ErrorCode>;

    /// Get the current bus phase for the current chip select.
    fn get_phase(&self) -> ClockPhase;
}

/// Trait for SPI peripherals (slaves) to receive callbacks when the
/// corresponding controller (master) issues operations.
///
/// A SPI operation begins with a callback of [`SpiSlaveClient::chip_selected`].
/// If the client has provided buffers with [`SpiSlave::read_write_bytes`],
/// these buffers are written from and read into until the operation completes
/// or one of them fills, at which point a [`SpiSlaveClient::read_write_done`]
/// callback is called. If the client needs to read/write more it can call
/// [`SpiSlave::read_write_bytes`] again.
///
/// Note that there is no notification when the chip select line goes high.
pub trait SpiSlaveClient {
    /// Notification that the chip select has been brought low.
    fn chip_selected(&self);

    /// Callback issued when the controller completes an SPI operation to this
    /// peripheral.
    ///
    /// `write_buffer` and `read_buffer` are the values passed in the previous
    /// call to [`SpiSlave::read_write_bytes`]. The `len` parameter specifies
    /// how many bytes were written from/read into `Some` values of these
    /// buffers. `len` may be shorter than the size of these buffers if the
    /// operation did not fill them.
    fn read_write_done(
        &self,
        write_buffer: Option<&'static mut [u8]>,
        read_buffer: Option<&'static mut [u8]>,
        len: usize,
        status: Result<(), ErrorCode>,
    );
}

/// Trait for SPI peripherals (slaves) to exchange data with a contoller
/// (master).
///
/// This is a low-level trait typically implemented by hardware: higher level
/// software typically uses the [`SpiSlaveDevice`] trait, which is provided by a
/// virtualizing/multiplexing layer.
pub trait SpiSlave<'a> {
    /// Initialize the SPI device to be in peripheral mode.
    ///
    /// ### Return values
    ///
    /// - `Ok(())`: the device is in peripheral mode
    /// - `Err(BUSY)`: the device is busy with an operation and cannot be
    ///   initialized
    /// - `Err(FAIL)`: other failure condition
    fn init(&self) -> Result<(), ErrorCode>;

    /// Returns true if there is a client. Useful for verifying that two
    /// software drivers do not both try to take control of the device.
    fn has_client(&self) -> bool;

    /// Set the callback for slave operations, passing `None` to disable
    /// peripheral mode.
    fn set_client(&self, client: Option<&'a dyn SpiSlaveClient>);

    /// Set a single byte to write in response to a read/write operation from a
    /// controller. Useful for devices that always send a status code in their
    /// first byte.
    fn set_write_byte(&self, write_byte: u8);

    /// Provide buffers for the peripheral to write from and read into when a
    /// controller performs a `read_write_bytes` operation.
    ///
    /// The device will issue a callback when one of four things occurs:
    ///
    /// - The controller completes the operation by bringing the chip select
    ///   high.
    /// - A `Some` write buffer is written.
    /// - A `Some` read buffer is filled.
    /// - `len` bytes are read/written
    ///
    /// ### Return values
    ///
    /// - `Ok(())`: the SPI bus will read/write the provided buffers on the next
    ///   SPI operation requested by the controller.
    /// - `Err(BUSY)`: the device is busy with an existing `read_write_bytes`
    ///   operation.
    /// - `Err(INVAL)`: the `len` parameter is 0
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

    /// Set the bus polarity (whether idle is high or low).
    ///
    /// ### Return values
    ///
    /// - `Ok(())`: the polarity was set.
    /// - `Err(BUSY)`: the SPI bus is busy with a [`SpiSlave::read_write_bytes`]
    ///   operation whose callback hasn't been called yet.
    /// - `Err(FAIL)`: other failure
    fn set_polarity(&self, polarity: ClockPolarity) -> Result<(), ErrorCode>;

    /// Return the current bus polarity.
    fn get_polarity(&self) -> ClockPolarity;

    /// Set the bus phase (whether data is sent/received on leading or
    /// trailing edges).
    ///
    /// ### Return values
    ///
    /// - `Ok(())`: the phase was set.
    /// - `Err(BUSY)`: the SPI bus is busy with a [`SpiSlave::read_write_bytes`]
    ///   operation whose callback hasn't been called yet.
    /// - `Err(FAIL)`: other failure
    fn set_phase(&self, phase: ClockPhase) -> Result<(), ErrorCode>;

    /// Return the current bus phase.
    fn get_phase(&self) -> ClockPhase;
}

/// An interface to a SPI bus in peripheral mode.
///
/// It is the standard trait used by services within the kernel: [`SpiSlave`] is
/// for lower-level access responsible for initializing hardware.
pub trait SpiSlaveDevice<'a> {
    /// Specify the callback of [`SpiSlaveDevice::read_write_bytes`] operations.
    fn set_client(&self, client: &'a dyn SpiSlaveClient);

    /// Setup the SPI settings and speed of the bus.
    fn configure(&self, cpol: ClockPolarity, cpal: ClockPhase) -> Result<(), ErrorCode>;

    /// Same as [`SpiSlave::read_write_bytes`].
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

    /// Same as [`SpiSlave::set_polarity`].
    fn set_polarity(&self, polarity: ClockPolarity) -> Result<(), ErrorCode>;

    /// Return the current bus polarity.
    fn get_polarity(&self) -> ClockPolarity;

    /// Same as [`SpiSlave::set_phase`].
    fn set_phase(&self, phase: ClockPhase) -> Result<(), ErrorCode>;

    /// Return the current bus phase.
    fn get_phase(&self) -> ClockPhase;
}

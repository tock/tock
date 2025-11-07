// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2025.

use kernel::utilities::leasable_buffer::SubSliceMut;

mod common;
pub mod spi;

/// Addresses (F0/F1/F2) are 32 bits
pub type RegAddr = u32;

/// Function of the incoming operation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Function {
    /// Register access
    Bus = 0b00,
    /// Inner address space access
    Backplane = 0b01,
    /// WLAN packets
    Wlan = 0b10,
}

/// Supported transfer types
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Type {
    Read = 0b0,
    Write = 0b1,
}

/// Length of bytes to read/write in a register
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum RegLen {
    Byte = 1,
    HalfWord = 2,
    Word = 4,
}

/// Current state of the bus
#[derive(Clone, Copy, Debug)]
pub enum State {
    /// No incoming packet available
    Idle,
    /// CLient should wait for a `packet_available` call
    Incoming,
    /// A packet is now available to be read
    Available(usize),
}

/// Trait for a bus that is used to communicate with CYW43xx device
pub trait CYW4343xBus<'a> {
    /// Set the client to be used for callbacks of the `Cyw43Bus` implementation
    fn set_client(&self, client: &'a dyn CYW4343xBusClient);

    /// Initialise the bus
    fn init(&self) -> Result<(), kernel::ErrorCode>;

    /// Write a WLAN (F2) packet
    fn write_bytes(
        &self,
        buffer: SubSliceMut<'static, u8>,
    ) -> Result<(), (kernel::ErrorCode, SubSliceMut<'static, u8>)>;

    /// Read a WLAN (F2) packet if available.
    fn read_bytes(
        &self,
        buffer: SubSliceMut<'static, u8>,
        len: usize,
    ) -> Result<(), (kernel::ErrorCode, SubSliceMut<'static, u8>)>;

    /// Get current bus state
    fn state(&self) -> Result<State, kernel::ErrorCode>;
}

/// Client trait for defining callbacks on initialisation, transfer
/// and F2 packet interrupts.
pub trait CYW4343xBusClient {
    /// Initialisation process is done
    fn init_done(&self, rval: Result<(), kernel::ErrorCode>);

    /// WLAN write done
    fn write_bytes_done(
        &self,
        buffer: SubSliceMut<'static, u8>,
        rval: Result<(), kernel::ErrorCode>,
    );

    /// WLAN read done
    fn read_bytes_done(
        &self,
        buffer: SubSliceMut<'static, u8>,
        rval: Result<(), kernel::ErrorCode>,
    );

    /// An F2 (WLAN) packet is done read
    fn packet_available(&self, len: usize);
}

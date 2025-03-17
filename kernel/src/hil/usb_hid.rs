// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Interface for USB HID (Human Interface Device) class

use crate::ErrorCode;

/// The 'types' of USB HID, this should define the size of send/received packets
pub trait UsbHidType: Copy + Clone + Sized {}

impl UsbHidType for [u8; 64] {}
impl UsbHidType for [u8; 32] {}
impl UsbHidType for [u8; 16] {}
impl UsbHidType for [u8; 8] {}

/// Implement this trait and use `set_client()` in order to receive callbacks.
pub trait Client<'a, T: UsbHidType> {
    /// Called when a packet is received.
    /// This will return the buffer passed into `receive_buffer()` as well as
    /// the endpoint where the data was received. If the buffer length is smaller
    /// then the data length the buffer will only contain part of the packet and
    /// `result` will contain indicate an `SIZE` error. Result will indicate
    /// `CANCEL` if a receive was cancelled by `receive_cancel()` but the
    /// callback still occurred. See `receive_cancel()` for more details.
    fn packet_received(
        &'a self,
        result: Result<(), ErrorCode>,
        buffer: &'static mut T,
        endpoint: usize,
    );

    /// Called when a packet has been finished transmitting.
    /// This will return the buffer passed into `send_buffer()` as well as
    /// the endpoint where the data was sent. If not all of the data could
    /// be sent the `result` will contain the `SIZE` error. Result will
    /// indicate `CANCEL` if a send was cancelled by `send_cancel()`
    /// but the callback still occurred. See `send_cancel()` for more
    /// details.
    fn packet_transmitted(
        &'a self,
        result: Result<(), ErrorCode>,
        buffer: &'static mut T,
        endpoint: usize,
    );
}

pub trait UsbHid<'a, T: UsbHidType> {
    /// Sets the buffer to be sent and starts a send transaction.
    /// Once the packet is sent the `packet_transmitted()` callback will
    /// be triggered and no more data will be sent until
    /// this is called again.
    ///
    /// Once this is called, the implementation will wait until either
    /// the packet is sent or `send_cancel()` is called.
    ///
    /// Calling `send_buffer()` while there is an outstanding
    /// `send_buffer()` operation will return BUSY.
    ///
    /// On success returns the length of data to be sent.
    /// On failure returns an error code and the buffer passed in.
    fn send_buffer(&'a self, send: &'static mut T) -> Result<usize, (ErrorCode, &'static mut T)>;

    /// Cancels a send called by `send_buffer()`.
    /// If `send_cancel()` successfully cancels a send transaction
    /// before the transaction has been acted upon this function will
    /// return the buffer passed via `send_buffer()` and no callback
    /// will occur.
    /// If there is currently no send transaction (`send_buffer()`
    /// hasn't been called) this will return `Err(INVAL)`.
    /// If the transaction can't be cancelled cleanly, either because
    /// the send has already occured, a partial send has occured or the
    /// send can not be cancelled by the hardware this will return
    /// `Err(BUSY)` and the callback will still occur.
    /// Note that unless the transaction completes the callback will
    /// indicate a result of `CANCEL`.
    fn send_cancel(&'a self) -> Result<&'static mut T, ErrorCode>;

    /// Sets the buffer for received data to be stored and enables receive
    /// transactions. Once this is called the implementation will enable
    /// receiving via USB. Once a packet is received the `packet_received()`
    /// callback will be triggered and no more data will be received until
    /// this is called again.
    ///
    /// Once this is called, the implementation will wait until either
    /// a packet is received or `receive_cancel()` is called.
    ///
    /// Calling `receive_buffer()` while there is an outstanding
    /// `receive_buffer()` operation will return BUSY.
    ///
    /// On success returns nothing.
    /// On failure returns an error code and the buffer passed in.
    fn receive_buffer(&'a self, recv: &'static mut T) -> Result<(), (ErrorCode, &'static mut T)>;

    /// Cancels a receive called by `receive_buffer()`.
    /// If `receive_cancel()` successfully cancels a receive transaction
    /// before the transaction has been acted upon this function will
    /// return the buffer passed via `receive_buffer()` and no callback
    /// will occur.
    /// If there is currently no receive transaction (`receive_buffer()`
    /// hasn't been called) this will return `Err(INVAL)`.
    /// If the transaction can't be cancelled cleanly, either because
    /// the receive has already occured, a partial receive has occured or the
    /// receive can not be cancelled by the hardware this will return
    /// `Err(BUSY)` and the callback will still occur.
    /// Note that unless the transaction completes the callback will
    /// indicate a result of `CANCEL`.
    fn receive_cancel(&'a self) -> Result<&'static mut T, ErrorCode>;
}

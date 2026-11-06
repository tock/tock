// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! BLE advertising across the two nRF5340 cores
//!
//! Only the network core has a radio. [`BleServer`] runs there and drives it
//! for [`BleProxy`], which runs on the application core and implements the BLE
//! advertising HIL. They exchange one request and one response at a time
//! through a [`BleMailboxRegisters`] block in RAM that both cores can access,
//! and signal each other on IPC channels [`REQUEST_CHANNEL`] and
//! [`RESPONSE_CHANNEL`]. The server announces itself when it starts, so the
//! proxy can fail a request that a network core reset lost.

use core::cell::Cell;
use kernel::ErrorCode;
use kernel::errorcode::into_statuscode;
use kernel::hil::ble_advertising::{self, BleAdvertisementDriver, BleConfig, RadioChannel};
use kernel::utilities::StaticRef;
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::utilities::registers::interfaces::{Readable, Writeable};
use kernel::utilities::registers::{FieldValue, ReadWrite, register_bitfields, register_structs};

use crate::constants::TxPower;
use crate::ipc::{Ipc, IpcClient};

/// IPC channel for requests from the application core.
pub const REQUEST_CHANNEL: usize = 0;
/// IPC channel for responses from the network core.
pub const RESPONSE_CHANNEL: usize = 1;

/// Largest packet the mailbox carries, the most an 8-bit LEN field holds.
const MAX_PACKET_LEN: usize = 255;

register_structs! {
    pub BleMailboxRegisters {
        (0x000 => request: ReadWrite<u32, Request::Register>),
        (0x004 => request_data: [ReadWrite<u8>; MAX_PACKET_LEN + 1]),
        (0x104 => response: ReadWrite<u32, Response::Register>),
        (0x108 => response_data: [ReadWrite<u8>; MAX_PACKET_LEN + 1]),
        (0x208 => @END),
    }
}

register_bitfields! [u32,
    Request [
        OP OFFSET(0) NUMBITS(8) [
            Transmit = 1,
            Receive = 2
        ],
        /// BLE channel index, 37 to 39.
        CHANNEL OFFSET(8) NUMBITS(8) [],
        LEN OFFSET(16) NUMBITS(8) [],
        TX_POWER OFFSET(24) NUMBITS(8) []
    ],
    Response [
        OP OFFSET(0) NUMBITS(8) [
            Transmitted = 1,
            Received = 2,
            /// The server started.
            Ready = 3
        ],
        /// 0 for success, otherwise the `ErrorCode`.
        RESULT OFFSET(8) NUMBITS(8) [],
        LEN OFFSET(16) NUMBITS(8) []
    ]
];

/// The result that `into_statuscode` encoded as `code`.
fn result_from_code(code: u32) -> Result<(), ErrorCode> {
    match code {
        0 => Ok(()),
        2 => Err(ErrorCode::BUSY),
        3 => Err(ErrorCode::ALREADY),
        4 => Err(ErrorCode::OFF),
        5 => Err(ErrorCode::RESERVE),
        6 => Err(ErrorCode::INVAL),
        7 => Err(ErrorCode::SIZE),
        8 => Err(ErrorCode::CANCEL),
        9 => Err(ErrorCode::NOMEM),
        10 => Err(ErrorCode::NOSUPPORT),
        11 => Err(ErrorCode::NODEVICE),
        12 => Err(ErrorCode::UNINSTALLED),
        13 => Err(ErrorCode::NOACK),
        _ => Err(ErrorCode::FAIL),
    }
}

/// Application-core side: the BLE advertising HIL, served by the network core.
pub struct BleProxy<'a> {
    mailbox: StaticRef<BleMailboxRegisters>,
    ipc: &'a Ipc<'a>,
    tx_power: Cell<u8>,
    tx_buffer: TakeCell<'static, [u8]>,
    rx_buffer: TakeCell<'static, [u8]>,
    tx_client: OptionalCell<&'a dyn ble_advertising::TxClient>,
    rx_client: OptionalCell<&'a dyn ble_advertising::RxClient>,
}

impl<'a> BleProxy<'a> {
    pub fn new(mailbox: StaticRef<BleMailboxRegisters>, ipc: &'a Ipc<'a>) -> Self {
        Self {
            mailbox,
            ipc,
            tx_power: Cell::new(TxPower::ZerodBm as u8),
            tx_buffer: TakeCell::empty(),
            rx_buffer: TakeCell::empty(),
            tx_client: OptionalCell::empty(),
            rx_client: OptionalCell::empty(),
        }
    }

    /// Take the IPC client role and listen for responses.
    pub fn init(&'a self) {
        self.ipc.set_client(self);
        self.ipc.listen(RESPONSE_CHANNEL);
    }

    fn request(&self, op: FieldValue<u32, Request::Register>, channel: RadioChannel, len: usize) {
        self.mailbox.request.write(
            op + Request::CHANNEL.val(channel.get_channel_index())
                + Request::LEN.val(len as u32)
                + Request::TX_POWER.val(self.tx_power.get().into()),
        );
        self.ipc.send(REQUEST_CHANNEL);
    }
}

impl<'a> BleAdvertisementDriver<'a> for BleProxy<'a> {
    fn transmit_advertisement(&self, buf: &'static mut [u8], len: usize, channel: RadioChannel) {
        let len = len.min(buf.len()).min(MAX_PACKET_LEN);
        for (reg, byte) in self.mailbox.request_data.iter().zip(&buf[..len]) {
            reg.set(*byte);
        }
        self.tx_buffer.replace(buf);
        self.request(Request::OP::Transmit, channel, len);
    }

    fn receive_advertisement(&self, buf: &'static mut [u8], channel: RadioChannel) {
        self.rx_buffer.replace(buf);
        self.request(Request::OP::Receive, channel, 0);
    }

    fn set_receive_client(&self, client: &'a dyn ble_advertising::RxClient) {
        self.rx_client.set(client);
    }

    fn set_transmit_client(&self, client: &'a dyn ble_advertising::TxClient) {
        self.tx_client.set(client);
    }
}

impl BleConfig for BleProxy<'_> {
    // Checked here because the answer is synchronous. The network core radio
    // tops out at 0 dBm.
    fn set_tx_power(&self, tx_power: u8) -> Result<(), ErrorCode> {
        match TxPower::try_from(tx_power) {
            Ok(TxPower::Positive4dBM | TxPower::Positive3dBM) | Err(()) => {
                Err(ErrorCode::NOSUPPORT)
            }
            Ok(_) => {
                self.tx_power.set(tx_power);
                Ok(())
            }
        }
    }
}

impl IpcClient for BleProxy<'_> {
    fn received(&self, channel: usize) {
        if channel != RESPONSE_CHANNEL {
            return;
        }
        let response = self.mailbox.response.extract();
        let result = result_from_code(response.read(Response::RESULT));
        match response.read_as_enum(Response::OP) {
            Some(Response::OP::Value::Transmitted) => {
                self.tx_buffer.take().map(|buf| {
                    self.tx_client
                        .map(|client| client.transmit_event(buf, result));
                });
            }
            Some(Response::OP::Value::Received) => {
                let len = response.read(Response::LEN) as u8;
                self.rx_buffer.take().map(|buf| {
                    let data = self.mailbox.response_data.iter();
                    for (byte, reg) in buf.iter_mut().zip(data).take(len.into()) {
                        *byte = reg.get();
                    }
                    self.rx_client
                        .map(|client| client.receive_event(buf, len, result));
                });
            }
            // A request still pending was lost when the network core restarted.
            Some(Response::OP::Value::Ready) => {
                self.tx_buffer.take().map(|buf| {
                    self.tx_client
                        .map(|client| client.transmit_event(buf, Err(ErrorCode::FAIL)));
                });
                self.rx_buffer.take().map(|buf| {
                    self.rx_client
                        .map(|client| client.receive_event(buf, 0, Err(ErrorCode::FAIL)));
                });
            }
            None => (),
        }
    }
}

/// Network-core side: drives the radio for a [`BleProxy`].
pub struct BleServer<'a, R: BleAdvertisementDriver<'a> + BleConfig> {
    mailbox: StaticRef<BleMailboxRegisters>,
    ipc: &'a Ipc<'a>,
    radio: &'a R,
    tx_buffer: TakeCell<'static, [u8]>,
    rx_buffer: TakeCell<'static, [u8]>,
}

impl<'a, R: BleAdvertisementDriver<'a> + BleConfig> BleServer<'a, R> {
    pub fn new(
        mailbox: StaticRef<BleMailboxRegisters>,
        ipc: &'a Ipc<'a>,
        radio: &'a R,
        tx_buffer: &'static mut [u8],
        rx_buffer: &'static mut [u8],
    ) -> Self {
        Self {
            mailbox,
            ipc,
            radio,
            tx_buffer: TakeCell::new(tx_buffer),
            rx_buffer: TakeCell::new(rx_buffer),
        }
    }

    /// Take the IPC and radio client roles, listen for requests, and announce
    /// the start to the proxy.
    pub fn init(&'a self) {
        self.ipc.set_client(self);
        self.radio.set_transmit_client(self);
        self.radio.set_receive_client(self);
        self.ipc.listen(REQUEST_CHANNEL);
        self.respond(Response::OP::Ready, Ok(()), 0);
    }

    fn respond(
        &self,
        op: FieldValue<u32, Response::Register>,
        result: Result<(), ErrorCode>,
        len: u8,
    ) {
        self.mailbox.response.write(
            op + Response::RESULT.val(into_statuscode(result) as u32)
                + Response::LEN.val(len.into()),
        );
        self.ipc.send(RESPONSE_CHANNEL);
    }
}

impl<'a, R: BleAdvertisementDriver<'a> + BleConfig> IpcClient for BleServer<'a, R> {
    fn received(&self, channel: usize) {
        if channel != REQUEST_CHANNEL {
            return;
        }
        let request = self.mailbox.request.extract();
        let channel = match request.read(Request::CHANNEL) {
            37 => Some(RadioChannel::AdvertisingChannel37),
            38 => Some(RadioChannel::AdvertisingChannel38),
            39 => Some(RadioChannel::AdvertisingChannel39),
            _ => None,
        };
        match request.read_as_enum(Request::OP) {
            Some(Request::OP::Value::Transmit) => match (channel, self.tx_buffer.take()) {
                (Some(channel), Some(buf)) => {
                    let len = (request.read(Request::LEN) as usize).min(buf.len());
                    let data = self.mailbox.request_data.iter();
                    for (byte, reg) in buf.iter_mut().zip(data).take(len) {
                        *byte = reg.get();
                    }
                    let _ = self
                        .radio
                        .set_tx_power(request.read(Request::TX_POWER) as u8);
                    self.radio.transmit_advertisement(buf, len, channel);
                }
                (_, buf) => {
                    buf.map(|buf| self.tx_buffer.replace(buf));
                    self.respond(Response::OP::Transmitted, Err(ErrorCode::INVAL), 0);
                }
            },
            Some(Request::OP::Value::Receive) => match (channel, self.rx_buffer.take()) {
                (Some(channel), Some(buf)) => self.radio.receive_advertisement(buf, channel),
                (_, buf) => {
                    buf.map(|buf| self.rx_buffer.replace(buf));
                    self.respond(Response::OP::Received, Err(ErrorCode::INVAL), 0);
                }
            },
            None => (),
        }
    }
}

impl<'a, R: BleAdvertisementDriver<'a> + BleConfig> ble_advertising::TxClient for BleServer<'a, R> {
    fn transmit_event(&self, buf: &'static mut [u8], result: Result<(), ErrorCode>) {
        self.tx_buffer.replace(buf);
        self.respond(Response::OP::Transmitted, result, 0);
    }
}

impl<'a, R: BleAdvertisementDriver<'a> + BleConfig> ble_advertising::RxClient for BleServer<'a, R> {
    fn receive_event(&self, buf: &'static mut [u8], len: u8, result: Result<(), ErrorCode>) {
        for (reg, byte) in self
            .mailbox
            .response_data
            .iter()
            .zip(buf.iter())
            .take(len.into())
        {
            reg.set(*byte);
        }
        self.rx_buffer.replace(buf);
        self.respond(Response::OP::Received, result, len);
    }
}

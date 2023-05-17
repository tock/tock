// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! IEEE 802.15.4 userspace interface for configuration and transmit/receive.
//!
//! Implements a userspace interface for sending and receiving IEEE 802.15.4
//! frames. Also provides a minimal list-based interface for managing keys and
//! known link neighbors, which is needed for 802.15.4 security.

use crate::ieee802154::framer::Framer;
use crate::ieee802154::mac::AwakeMac;
use crate::net::ieee802154::{AddressMode, Header, KeyId, MacAddress, PanID, SecurityLevel};
use crate::net::stream::{decode_bytes, decode_u8, encode_bytes, encode_u32, encode_u8, SResult};
use crate::net::thread::{device, framer, mac};
use crate::net::udp::udp_recv::{MuxUdpReceiver, UDPReceiver};
use capsules_core::button::SubscribeMap;
use capsules_core::virtualizers::virtual_aes_ccm::VirtualAES128CCM;

use core::cell::Cell;
use kernel::hil::symmetric_encryption::{CCMClient, AES128CCM};

use kernel::deferred_call::{DeferredCall, DeferredCallClient};
use kernel::grant::{AllowRoCount, AllowRwCount, Grant, UpcallCount};
use kernel::processbuffer::{ReadableProcessBuffer, WriteableProcessBuffer};
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::utilities::cells::{MapCell, OptionalCell, TakeCell};
use kernel::{ErrorCode, ProcessId};

const MAX_NEIGHBORS: usize = 4;
const MAX_KEYS: usize = 4;

/// Ids for read-only allow buffers
mod ro_allow {
    pub const WRITE: usize = 0;
    /// The number of allow buffers the kernel stores for this grant
    pub const COUNT: u8 = 1;
}

/// Ids for read-write allow buffers
mod rw_allow {
    pub const READ: usize = 0;
    pub const CFG: usize = 1;
    /// The number of allow buffers the kernel stores for this grant
    pub const COUNT: u8 = 2;
}

use capsules_core::driver;
pub const DRIVER_NUM: usize = driver::NUM::Thread as usize;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
struct DeviceDescriptor {
    short_addr: u16,
    long_addr: [u8; 8],
}

impl Default for DeviceDescriptor {
    fn default() -> Self {
        DeviceDescriptor {
            short_addr: 0,
            long_addr: [0; 8],
        }
    }
}

/// The Key ID mode mapping expected by the userland driver
#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum KeyIdModeUserland {
    Implicit = 0,
    Index = 1,
    Source4Index = 2,
    Source8Index = 3,
}

impl KeyIdModeUserland {
    pub fn from_u8(byte: u8) -> Option<KeyIdModeUserland> {
        match byte {
            0 => Some(KeyIdModeUserland::Implicit),
            1 => Some(KeyIdModeUserland::Index),
            2 => Some(KeyIdModeUserland::Source4Index),
            3 => Some(KeyIdModeUserland::Source8Index),
            _ => None,
        }
    }
}

#[derive(Default)]
pub struct App {
    pending_tx: Option<(u16, Option<(SecurityLevel, KeyId)>)>,
}

pub struct ThreadNetworkDriver<'a> {
    /// Underlying MAC device, possibly multiplexed
    encry: &'a dyn AES128CCM<'a>,
    udp_recv_client: MuxUdpReceiver<'a>,
    /// Grant of apps that use this radio driver.
    apps: Grant<
        App,
        UpcallCount<2>,
        AllowRoCount<{ ro_allow::COUNT }>,
        AllowRwCount<{ rw_allow::COUNT }>,
    >,
}

impl<'a> ThreadNetworkDriver<'a> {
    pub fn new(
        encry: &'a dyn AES128CCM<'a>,
        udp_recv_client: MuxUdpReceiver<'a>,
        grant: Grant<
            App,
            UpcallCount<2>,
            AllowRoCount<{ ro_allow::COUNT }>,
            AllowRwCount<{ rw_allow::COUNT }>,
        >,
    ) -> Self {
        Self {
            encry,
            udp_recv_client,
            apps: grant,
        }
    }
}

impl SyscallDriver for ThreadNetworkDriver<'_> {
    /// Setup buffers to read/write from.
    ///
    /// ### `allow_num`
    ///

    fn command(
        &self,
        command_number: usize,
        arg1: usize,
        _: usize,
        processid: ProcessId,
    ) -> CommandReturn {
        match command_number {
            0 => {
                kernel::debug!("We have successfully called our first tock capsule.");
                // let mut rcvr: &'static UDPReceiver = &UDPReceiver::new();
                // self.udp_recv_client.add_client();

                CommandReturn::success()
            }
            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::process::Error> {
        self.apps.enter(processid, |_, _| {})
    }
}

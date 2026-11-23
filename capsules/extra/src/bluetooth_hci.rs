// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Userspace driver for a Bluetooth HCI transport.
//!
//! This capsule provides a single process access to a Bluetooth controller
//! through an [`HciTransport`] (i.e., it is not virtualized). It exchanges H4
//! packets and uses the [`kernel::hil::bluetooth_hci`] HIL.
//!
//! The driver initializes the controller when a process first uses the driver.
//!
//! A process should allow its receive slice (read-write allow buffer 0) before
//! its first command. Otherwise, it must issue command 2 after allowing the
//! slice to start reception.
//!
//! # Receiving packets
//!
//! The driver appends received packets from the controller to a
//! [`StreamingProcessSlice`] shared with a process. It writes the packet length
//! in front of each packet as a native-endian `u16`. The driver schedules
//! upcall 0 when it appends data to an empty slice. The userspace process is
//! expected to then swap out this buffer and re-allow an empty one. This way,
//! we don't unnecessarily notify userspace for repeated receive operations.
//!
//! The driver stops reception from the controller when the slice has less free
//! space than the longest packet needs (implicitly rate-limiting the
//! controller). The process swaps in an empty slice and issues command 2 to
//! resume reception. This capsule does not drop packets because of a full
//! slice, which can be important to avoid dropping BLE connections.
//!
//! # Controller failures
//!
//! The driver restarts the controller when it fails for whatever reason (e.g.,
//! hardware error, or a violation caught by an Omniglot-wrapped BLE stack). It
//! accepts and discards all packets that a process (BLE host) sends while the
//! controller restarts. After the restart, it appends an HCI Hardware Error
//! event to the RX streaming process slice. The process is expected to answer
//! with an HCI Reset command. The driver discards all other packets from the
//! host until it receives this command (which a BLE host like NimBLE will issue
//! after receiving an HCI Hardware Error event). This ensures that the BLE host
//! state is reset appropriately.
//!
//! If the controller does not initialize (either initially or after a failure),
//! the driver also appends an HCI Hardware Error event, and command 1 fails
//! with `OFF` from then on.
//!
//! # System call interface
//!
//! ## Command system calls
//!
//! - Command 0: Check whether the driver exists.
//! - Command 1: Send the first `arg1` bytes of read-only allow buffer 0 as
//!   one H4 packet. The packet type must be command or ACL data.
//!
//!   Command 1 returns these errors:
//!   - `BUSY`: The driver cannot accept a packet now (for instance, on the
//!     first command, while the controller initializes). The driver schedules
//!     upcall 1 when it can accept a packet.
//!   - `RESERVE`: Another process owns the driver.
//!   - `OFF`: The controller did not start / initialize.
//!   - `SIZE`: `arg1` is zero, larger than the allow buffer, or larger than
//!     [`MAX_PACKET_LEN`].
//!   - `INVAL`: The packet is not a well-formed command or ACL data packet.
//! - Command 2: Resume reception after the process swapped in an empty receive
//!   slice. Like command 1, the first command 2 claims the driver and starts
//!   the controller.
//!
//!   Command 2 returns these errors:
//!   - `RESERVE`: Another process owns the driver.
//!
//! ## Subscribe system calls
//!
//! - Upcall 0: The driver appended data to an empty receive slice.
//! - Upcall 1: The driver can accept a packet with command 1.
//!
//! ## Read-only allow system calls
//!
//! - Buffer 0: The packet to send with command 1.
//!
//! ## Read-write allow system calls
//!
//! - Buffer 0: A [`StreamingProcessSlice`] that receives packets from the
//!   controller.

use core::cell::Cell;

use kernel::grant::{AllowRoCount, AllowRwCount, Grant, UpcallCount};
use kernel::hil::bluetooth_hci::{HciTransport, HciTransportClient, MAX_PACKET_LEN, PacketType};
use kernel::processbuffer::{ReadableProcessBuffer, WriteableProcessBuffer};
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::utilities::streaming_process_slice::StreamingProcessSlice;
use kernel::{ErrorCode, ProcessId};

/// Syscall driver number.
pub const DRIVER_NUM: usize = capsules_core::driver::NUM::BluetoothHci as usize;

/// Length of the length prefix in front of every packet in the receive slice.
const PREFIX_LEN: usize = 2;

/// Space in the receive slice that the longest packet needs.
const CHUNK_MAX: usize = PREFIX_LEN + MAX_PACKET_LEN;

/// Opcode of the HCI Reset command.
const HCI_RESET_OPCODE: u16 = 0x0c03;

/// Hardware code in the Hardware Error event that this driver sends.
///
/// The Bluetooth Core Specification leaves the meaning of the hardware code to
/// the implementation. We choose an arbitrary one to identify this driver.
const HARDWARE_CODE: u8 = 0x5b;

/// HCI Hardware Error event that the driver sends after a controller restart.
const HARDWARE_ERROR_EVENT: [u8; 4] = [PacketType::Event as u8, 0x10, 0x01, HARDWARE_CODE];

mod upcall {
    pub const RX: usize = 0;
    pub const TX_READY: usize = 1;
    pub const COUNT: u8 = 2;
}

mod ro_allow {
    pub const TX: usize = 0;
    pub const COUNT: u8 = 1;
}

mod rw_allow {
    pub const RX: usize = 0;
    pub const COUNT: u8 = 1;
}

#[derive(Default)]
pub struct App;

type HciGrant = Grant<
    App,
    UpcallCount<{ upcall::COUNT }>,
    AllowRoCount<{ ro_allow::COUNT }>,
    AllowRwCount<{ rw_allow::COUNT }>,
>;

#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    /// The transport is stopped, and no reset is in progress.
    Stopped,
    /// A reset is in progress. Packets from the host wait for it.
    Resetting,
    /// A reset after a controller failure is in progress. The driver discards
    /// packets from the host.
    Recovering,
    /// The controller restarted after a failure. The driver discards packets
    /// from the host until the host sends HCI Reset.
    AwaitingHostReset,
    /// The transport is ready.
    Ready,
    /// The controller did not start. Only a new owner restarts it.
    Failed,
}

pub struct BluetoothHci<'a, T: HciTransport<'a>> {
    transport: &'a T,
    apps: HciGrant,
    owner: OptionalCell<ProcessId>,
    /// The buffer for packets to the controller, while no packet is
    /// outstanding.
    tx_buffer: TakeCell<'static, [u8]>,
    state: Cell<State>,
    /// Whether reception from the transport is enabled.
    receiving: Cell<bool>,
    /// The Hardware Error event is due to the owner.
    hardware_error_pending: Cell<bool>,
}

impl<'a, T: HciTransport<'a>> BluetoothHci<'a, T> {
    pub fn new(transport: &'a T, apps: HciGrant, tx_buffer: &'static mut [u8]) -> Self {
        Self {
            transport,
            apps,
            owner: OptionalCell::empty(),
            tx_buffer: TakeCell::new(tx_buffer),
            state: Cell::new(State::Stopped),
            receiving: Cell::new(false),
            hardware_error_pending: Cell::new(false),
        }
    }

    /// Returns the owner if it still exists.
    ///
    /// Releases the ownership if the owner process exited or restarted. The
    /// driver then resets the controller. The reset ends the connections and
    /// the advertising that the controller ran for that process.
    fn owner(&self) -> Option<ProcessId> {
        let owner = self.owner.get()?;

        // Check if the owner process is alive:
        if self.apps.enter(owner, |_, _| ()).is_ok() {
            return Some(owner);
        }

        // Otherwise, reset the owner and reset the state and controller:
        self.owner.clear();
        self.hardware_error_pending.set(false);
        self.set_receiving(false);
        if self.state.get() != State::Failed {
            self.start_reset(State::Resetting);
        }
        None
    }

    /// Makes `processid` the owner if the driver has no owner.
    fn claim(&self, processid: ProcessId) -> Result<(), ErrorCode> {
        match self.owner() {
            Some(owner) if owner == processid => Ok(()),
            Some(_) => Err(ErrorCode::RESERVE),
            None => {
                self.owner.set(processid);
                self.set_receiving(false);
                self.start_reset(State::Resetting);
                Ok(())
            }
        }
    }

    // `state` should be either `Resetting` or `Recovering`
    fn start_reset(&self, state: State) {
        self.state.set(state);
        // We ignore the error here, as `BUSY` means that a reset is already in
        // progress. The transport will then call `reset_done`, which ends this
        // reset request either way.
        let _ = self.transport.reset();
    }

    fn set_receiving(&self, enabled: bool) {
        if self.receiving.get() == enabled {
            return;
        }
        self.receiving.set(enabled);
        if enabled {
            self.transport.enable_receive();
        } else {
            self.transport.disable_receive();
        }
    }

    /// Returns the free space in the receive slice of `processid`.
    fn rx_space(&self, processid: ProcessId) -> usize {
        self.apps
            .enter(processid, |_, kernel_data| {
                kernel_data
                    .get_readwrite_processbuffer(rw_allow::RX)
                    .and_then(|buffer| {
                        buffer.mut_enter(|slice| {
                            StreamingProcessSlice::new(slice).remaining_capacity()
                        })
                    })
                    .unwrap_or(Err(ErrorCode::NOMEM))
                    .unwrap_or(0)
            })
            .unwrap_or(0)
    }

    /// Enables reception if the owner has space for the longest packet.
    fn update_receiving(&self) {
        let enabled = match self.state.get() {
            State::Ready | State::AwaitingHostReset => self.owner().is_some_and(|owner| {
                !self.hardware_error_pending.get() && self.rx_space(owner) >= CHUNK_MAX
            }),
            _ => false,
        };
        self.set_receiving(enabled);
    }

    /// Appends `packet` to the receive slice of `processid`.
    ///
    /// Returns `false` if the packet did not fit. The slice then records that
    /// data was lost.
    fn deliver(&self, processid: ProcessId, packet: &[u8]) -> bool {
        let mut chunk = [0u8; CHUNK_MAX];
        let Some(chunk) = chunk.get_mut(..PREFIX_LEN + packet.len()) else {
            return false;
        };
        chunk[..PREFIX_LEN].copy_from_slice(&(packet.len() as u16).to_ne_bytes());
        chunk[PREFIX_LEN..].copy_from_slice(packet);
        self.apps
            .enter(processid, |_, kernel_data| {
                let appended = kernel_data
                    .get_readwrite_processbuffer(rw_allow::RX)
                    .and_then(|buffer| {
                        buffer.mut_enter(|slice| {
                            StreamingProcessSlice::new(slice).append_chunk(chunk)
                        })
                    })
                    .unwrap_or(Err(ErrorCode::NOMEM));
                if let Ok((first, _)) = appended {
                    if first {
                        let _ = kernel_data.schedule_upcall(upcall::RX, (0, 0, 0));
                    }
                }
                appended.is_ok()
            })
            .unwrap_or(false)
    }

    /// Appends the H4 HCI Hardware Error event to the owner's receive slice, if
    /// one is pending.
    ///
    /// We must ensure that we always deliver a Hardware Error event, even if
    /// the controller fails while the process didn't have a buffer allowed with
    /// sufficient capacity. Therefore, we store it in a variable and try to
    /// append it here. On success, this clears the pending error indicator.
    fn deliver_hardware_error(&self) {
        if !self.hardware_error_pending.get() {
            return;
        }
        if let Some(owner) = self.owner() {
            if self.rx_space(owner) >= PREFIX_LEN + HARDWARE_ERROR_EVENT.len()
                && self.deliver(owner, &HARDWARE_ERROR_EVENT)
            {
                self.hardware_error_pending.set(false);
            }
        }
    }

    fn notify_tx_ready(&self) {
        if let Some(owner) = self.owner() {
            let _ = self.apps.enter(owner, |_, kernel_data| {
                let _ = kernel_data.schedule_upcall(upcall::TX_READY, (0, 0, 0));
            });
        }
    }

    /// Copies the packet from the process into the transmit buffer and returns
    /// the buffer.
    fn copy_packet(
        &self,
        processid: ProcessId,
        len: usize,
    ) -> Result<&'static mut [u8], ErrorCode> {
        if len == 0 || len > MAX_PACKET_LEN {
            return Err(ErrorCode::SIZE);
        }

        let buffer = self.tx_buffer.take().ok_or(ErrorCode::BUSY)?;
        let copied = self
            .apps
            .enter(processid, |_, kernel_data| {
                kernel_data
                    .get_readonly_processbuffer(ro_allow::TX)
                    .and_then(|src| {
                        src.enter(|src| {
                            let dst = buffer.get_mut(..len).ok_or(ErrorCode::SIZE)?;
                            let src = src.get(..len).ok_or(ErrorCode::SIZE)?;
                            src.copy_to_slice(dst);
                            Ok(())
                        })
                    })
                    .unwrap_or(Err(ErrorCode::NOMEM))
            })
            .unwrap_or_else(|err| Err(err.into()));

        match copied {
            Ok(()) => Ok(buffer),
            Err(e) => {
                self.tx_buffer.replace(buffer);
                Err(e)
            }
        }
    }

    fn send(&self, processid: ProcessId, len: usize) -> Result<(), ErrorCode> {
        match self.state.get() {
            State::Stopped | State::Resetting => return Err(ErrorCode::BUSY),
            State::Failed => return Err(ErrorCode::OFF),
            // Controller is down; we'll send a Hardware Error event.
            State::Recovering => return Ok(()),
            State::AwaitingHostReset | State::Ready => {}
        }

        let buffer = self.copy_packet(processid, len)?;
        let awaiting_host_reset = self.state.get() == State::AwaitingHostReset;
        if awaiting_host_reset {
            // HCI Reset has no parameters, so its parameter length is 0.
            let is_reset = len == 4
                && buffer[0] == PacketType::Command as u8
                && u16::from_le_bytes([buffer[1], buffer[2]]) == HCI_RESET_OPCODE
                && buffer[3] == 0;
            if !is_reset {
                self.tx_buffer.replace(buffer);
                return Ok(());
            }
        }

        match self.transport.transmit(buffer, len) {
            Ok(()) => {
                // Only leave `AwaitingHostReset` once the transport accepted
                // the HCI Reset.
                if awaiting_host_reset {
                    self.state.set(State::Ready);
                }
                Ok(())
            }
            Err((e, buffer)) => {
                self.tx_buffer.replace(buffer);
                match e {
                    // The transport stopped, and its `stopped` callback is
                    // still due. The packet is discarded as in `Recovering`.
                    ErrorCode::OFF => Ok(()),
                    e => Err(e),
                }
            }
        }
    }
}

impl<'a, T: HciTransport<'a>> HciTransportClient for BluetoothHci<'a, T> {
    fn transmit_done(&self, buffer: &'static mut [u8], _result: Result<(), ErrorCode>) {
        self.tx_buffer.replace(buffer);
        if matches!(self.state.get(), State::Ready | State::AwaitingHostReset) {
            self.notify_tx_ready();
        }
    }

    fn packet_received(&self, packet: &[u8]) {
        let Some(owner) = self.owner() else {
            return;
        };
        // A packet that does not fit is lost. This only happens if the process
        // replaced its slice with a smaller one.
        let _ = self.deliver(owner, packet);
        self.update_receiving();
    }

    fn reset_done(&self, result: Result<(), ErrorCode>) {
        match (result, self.state.get()) {
            (Err(_), _) => {
                self.state.set(State::Failed);
                // Tell the owner that the controller is gone, even if it is
                // only waiting for received packets and would otherwise never
                // notice. Command 1 then fails with `OFF`.
                if self.owner.is_some() {
                    self.hardware_error_pending.set(true);
                }
            }
            (Ok(()), State::Recovering) => {
                self.state.set(State::AwaitingHostReset);
                self.hardware_error_pending.set(true);
            }
            (Ok(()), _) => self.state.set(State::Ready),
        }
        self.deliver_hardware_error();
        self.update_receiving();
        self.notify_tx_ready();
    }

    fn stopped(&self) {
        if self.owner().is_some() {
            self.start_reset(State::Recovering);
        } else {
            self.state.set(State::Stopped);
        }
    }
}

impl<'a, T: HciTransport<'a>> SyscallDriver for BluetoothHci<'a, T> {
    fn command(
        &self,
        command_num: usize,
        arg1: usize,
        _arg2: usize,
        processid: ProcessId,
    ) -> CommandReturn {
        match command_num {
            0 => CommandReturn::success(),
            1 => self
                .claim(processid)
                .and_then(|()| self.send(processid, arg1))
                .into(),
            2 => self
                .claim(processid)
                .map(|()| {
                    self.deliver_hardware_error();
                    self.update_receiving();
                })
                .into(),
            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::process::Error> {
        self.apps.enter(processid, |_, _| {})
    }
}

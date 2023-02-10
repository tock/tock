// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022
// Copyright OxidOS Automotive SRL 2022
//
// Author: Teona Severin <teona.severin@oxidos.io>

//! Syscall driver capsule for CAN communication.
//!
//! This module has a CAN syscall driver capsule implementation.
//!
//! This capsule sends commands from the userspace to a driver that
//! implements the Can trait.
//!
//! The capsule shares 2 buffers with the userspace: one RO that is used
//! for transmitting messages and one RW that is used for receiving
//! messages.
//!
//! The RO buffer uses the first 4 bytes as a counter of how many messages
//! the userspace must read, at the time the upcall was sent. If the
//! userspace is slower and in the meantime there were other messages
//! that were received, the userspace reads them all and sends to the
//! capsule a new buffer that has the counter on the first 4 bytes 0.
//! Because of that, when receiving a callback from the driver regarding
//! a received message, the capsule checks the counter:
//! - if it's 0, the message will be copied to the RW buffer, the counter
//!   will be incremented and an upcall will be sent
//! - if it's greater the 0, the message will be copied to the RW buffer
//!   but no upcall will be done
//!
//! Usage
//! -----
//!
//! You need a driver that implements the Can trait.
//! ```rust
//! let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);
//! let grant_can = self.board_kernel.create_grant(
//!     capsules::can::CanCapsule::DRIVER_NUM, &grant_cap);
//! let can = capsules::can::CanCapsule::new(
//!    can_peripheral,
//!    grant_can,
//!    tx_buffer,
//!    rx_buffer,
//! );
//!
//! kernel::hil::can::Controller::set_client(can_peripheral, Some(can));
//! kernel::hil::can::Transmit::set_client(can_peripheral, Some(can));
//! kernel::hil::can::Receive::set_client(can_peripheral, Some(can));
//! ```
//!

use core::mem::size_of;

use kernel::grant::{AllowRoCount, AllowRwCount, Grant, UpcallCount};
use kernel::hil::can;
use kernel::processbuffer::{ReadableProcessBuffer, WriteableProcessBuffer};
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::ErrorCode;
use kernel::ProcessId;

use core_capsules::driver;
pub const DRIVER_NUM: usize = driver::NUM::Can as usize;
pub const BYTE4_MASK: usize = 0xff000000;
pub const BYTE3_MASK: usize = 0xff0000;
pub const BYTE2_MASK: usize = 0xff00;
pub const BYTE1_MASK: usize = 0xff;

mod error_upcalls {
    pub const ERROR_TX: usize = 100;
    pub const ERROR_RX: usize = 101;
}

mod up_calls {
    pub const UPCALL_ENABLE: usize = 0;
    pub const UPCALL_DISABLE: usize = 1;
    pub const UPCALL_MESSAGE_SENT: usize = 2;
    pub const UPCALL_MESSAGE_RECEIVED: usize = 3;
    pub const UPCALL_RECEIVED_STOPPED: usize = 4;
    pub const UPCALL_TRANSMISSION_ERROR: usize = 5;
    pub const COUNT: u8 = 6;
}

mod ro_allow {
    pub const RO_ALLOW_BUFFER: usize = 0;
    pub const COUNT: u8 = 1;
}

mod rw_allow {
    pub const RW_ALLOW_BUFFER: usize = 0;
    pub const COUNT: u8 = 1;
}

pub struct CanCapsule<'a, Can: can::Can> {
    // CAN driver
    can: &'a Can,

    // CAN buffers
    can_tx: TakeCell<'static, [u8; can::STANDARD_CAN_PACKET_SIZE]>,
    can_rx: TakeCell<'static, [u8; can::STANDARD_CAN_PACKET_SIZE]>,

    // Process
    processes: Grant<
        App,
        UpcallCount<{ up_calls::COUNT }>,
        AllowRoCount<{ ro_allow::COUNT }>,
        AllowRwCount<{ rw_allow::COUNT }>,
    >,
    processid: OptionalCell<ProcessId>,

    // Variable used to store the current state of the CAN peripheral
    // during an `enable` or `disable` command.
    peripheral_state: OptionalCell<can::State>,
}

pub struct App {
    receive_index: usize,
    lost_messages: u32,
}

impl Default for App {
    fn default() -> Self {
        App {
            receive_index: 0,
            lost_messages: 0,
        }
    }
}

impl<'a, Can: can::Can> CanCapsule<'a, Can> {
    pub fn new(
        can: &'a Can,
        grant: Grant<
            App,
            UpcallCount<{ up_calls::COUNT }>,
            AllowRoCount<{ ro_allow::COUNT }>,
            AllowRwCount<{ rw_allow::COUNT }>,
        >,
        can_tx: &'static mut [u8; can::STANDARD_CAN_PACKET_SIZE],
        can_rx: &'static mut [u8; can::STANDARD_CAN_PACKET_SIZE],
    ) -> CanCapsule<'a, Can> {
        CanCapsule {
            can,
            can_tx: TakeCell::new(can_tx),
            can_rx: TakeCell::new(can_rx),
            processes: grant,
            peripheral_state: OptionalCell::empty(),
            processid: OptionalCell::empty(),
        }
    }

    fn schedule_callback(&self, callback_number: usize, data: (usize, usize, usize)) {
        self.processid.map(|processid| {
            let _ = self.processes.enter(*processid, |_app, kernel_data| {
                kernel_data
                    .schedule_upcall(callback_number, (data.0, data.1, data.2))
                    .ok();
            });
        });
    }

    /// This function makes a copy of the buffer in the grant and sends it
    /// to the low-level hardware, in order for it to be sent on the bus.
    pub fn process_send_command(
        &self,
        processid: &mut ProcessId,
        id: can::Id,
        length: usize,
    ) -> Result<(), ErrorCode> {
        self.processes
            .enter(*processid, |_, kernel_data| {
                kernel_data
                    .get_readonly_processbuffer(ro_allow::RO_ALLOW_BUFFER)
                    .map_or_else(
                        |err| err.into(),
                        |buffer_ref| {
                            buffer_ref
                                .enter(|buffer| {
                                    self.can_tx.take().map_or(
                                        Err(ErrorCode::NOMEM),
                                        |dest_buffer| {
                                            for i in 0..length {
                                                dest_buffer[i] = buffer[i].get();
                                            }
                                            match self.can.send(id, dest_buffer, length) {
                                                Ok(_) => Ok(()),
                                                Err((err, buf)) => {
                                                    self.can_tx.replace(buf);
                                                    Err(err)
                                                }
                                            }
                                        },
                                    )
                                })
                                .unwrap_or_else(|err| err.into())
                        },
                    )
            })
            .unwrap_or_else(|err| err.into())
    }

    pub fn is_valid_process(&self, processid: ProcessId) -> bool {
        self.processid.map_or(true, |owning_process| {
            self.processes
                .enter(*owning_process, |_, _| owning_process == &processid)
                .unwrap_or(true)
        })
    }
}

impl<'a, Can: can::Can> SyscallDriver for CanCapsule<'a, Can> {
    fn command(
        &self,
        command_num: usize,
        arg1: usize,
        arg2: usize,
        processid: ProcessId,
    ) -> CommandReturn {
        // This driver exists.
        if command_num == 0 {
            return CommandReturn::success();
        }

        // Check to see if the process or no process at all
        // owns the capsule. Only one application can use the
        // capsule at a time.
        if !self.is_valid_process(processid) {
            return CommandReturn::failure(ErrorCode::RESERVE);
        } else {
            self.processid.set(processid);
        }

        match command_num {
            // Set the bitrate
            1 => match self.can.set_bitrate(arg1 as u32) {
                Ok(_) => CommandReturn::success(),
                Err(err) => CommandReturn::failure(err),
            },

            // Set the operation mode (Loopback, Monitoring, etc)
            2 => {
                match self.can.set_operation_mode(match arg1 {
                    0 => can::OperationMode::Loopback,
                    1 => can::OperationMode::Monitoring,
                    2 => can::OperationMode::Freeze,
                    _ => can::OperationMode::Normal,
                }) {
                    Ok(_) => CommandReturn::success(),
                    Err(err) => CommandReturn::failure(err),
                }
            }

            // Enable the peripheral
            3 => match self.can.enable() {
                Ok(_) => CommandReturn::success(),
                Err(err) => CommandReturn::failure(err),
            },

            // Disable the peripheral
            4 => match self.can.disable() {
                Ok(_) => CommandReturn::success(),
                Err(err) => CommandReturn::failure(err),
            },

            // Send a message with a 16-bit identifier
            5 => {
                let id = can::Id::Standard(arg1 as u16);
                self.processid
                    .map_or(
                        CommandReturn::failure(ErrorCode::BUSY),
                        |processid| match self.process_send_command(processid, id, arg2) {
                            Ok(_) => CommandReturn::success(),
                            Err(err) => CommandReturn::failure(err),
                        },
                    )
            }

            // Send a message with a 32-bit identifier
            6 => {
                let id = can::Id::Extended(arg1 as u32);
                self.processid
                    .map_or(
                        CommandReturn::failure(ErrorCode::BUSY),
                        |processid| match self.process_send_command(processid, id, arg2) {
                            Ok(_) => CommandReturn::success(),
                            Err(err) => CommandReturn::failure(err),
                        },
                    )
            }

            // Start receiving messages
            7 => {
                self.can_rx
                    .take()
                    .map(|dest_buffer| {
                        self.processes
                            .enter(processid, |_, kernel| {
                                match kernel.get_readwrite_processbuffer(0).map_or_else(
                                    |err| err.into(),
                                    |buffer_ref| {
                                        buffer_ref
                                            .enter(|buffer| {
                                                // make sure that the receiving buffer can have at least
                                                // 2 messages of 8 bytes each and 4 another bytes for the counter
                                                if buffer.len()
                                                    >= 2 * can::STANDARD_CAN_PACKET_SIZE
                                                        + size_of::<u32>()
                                                {
                                                    Ok(())
                                                } else {
                                                    Err(ErrorCode::SIZE)
                                                }
                                            })
                                            .unwrap_or_else(|err| err.into())
                                    },
                                ) {
                                    Ok(_) => match self.can.start_receive_process(dest_buffer) {
                                        Ok(_) => CommandReturn::success(),
                                        Err((err, _)) => CommandReturn::failure(err),
                                    },
                                    Err(err) => CommandReturn::failure(err.into()),
                                }
                            })
                            .unwrap_or_else(|err| err.into())
                    })
                    .unwrap_or(CommandReturn::failure(ErrorCode::NOMEM))
            }

            // Stop receiving messages
            8 => match self.can.stop_receive() {
                Ok(_) => CommandReturn::success(),
                Err(err) => CommandReturn::failure(err),
            },

            // Set the timing parameters
            9 => {
                match self.can.set_bit_timing(can::BitTiming {
                    segment1: ((arg1 & BYTE4_MASK) >> 24) as u8,
                    segment2: ((arg1 & BYTE3_MASK) >> 16) as u8,
                    propagation: arg2 as u8,
                    sync_jump_width: ((arg1 & BYTE2_MASK) >> 8) as u32,
                    baud_rate_prescaler: (arg1 & BYTE1_MASK) as u32,
                }) {
                    Ok(_) => CommandReturn::success(),
                    Err(err) => CommandReturn::failure(err),
                }
            }

            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, process_id: ProcessId) -> Result<(), kernel::process::Error> {
        self.processes.enter(process_id, |_, _| {})
    }
}

impl<'a, Can: can::Can> can::ControllerClient for CanCapsule<'a, Can> {
    // This callback must be called after an `enable` or `disable` command was sent.
    // It stores the new state of the peripheral.
    fn state_changed(&self, state: can::State) {
        self.peripheral_state.replace(state);
    }

    // This callback must be called after an `enable` command was sent and after a
    // `state_changed` callback was called. If there is no error and the state of
    // the peripheral is Running, send to the userspace a success callback.
    // If the state is different or the status is an error, send to the userspace an
    // error callback.
    fn enabled(&self, status: Result<(), ErrorCode>) {
        match status {
            Ok(_) => match self.peripheral_state.take() {
                Some(can::State::Running) => {
                    self.schedule_callback(up_calls::UPCALL_ENABLE, (0, 0, 0));
                }
                Some(can::State::Error(err)) => {
                    self.schedule_callback(up_calls::UPCALL_ENABLE, (err as usize, 0, 0));
                }
                Some(can::State::Disabled) | None => {
                    self.schedule_callback(
                        up_calls::UPCALL_ENABLE,
                        (ErrorCode::OFF as usize, 0, 0),
                    );
                }
            },
            Err(err) => {
                self.peripheral_state.take();
                self.schedule_callback(up_calls::UPCALL_ENABLE, (err as usize, 0, 0));
            }
        }
    }

    // This callback must be called after an `disable` command was sent and after a
    // `state_changed` callback was called. If there is no error and  the state of
    // the peripheral is Disabled, send to the userspace a success callback.
    // If the state is different or the status is an error, send to the userspace an
    // error callback.
    fn disabled(&self, status: Result<(), ErrorCode>) {
        match status {
            Ok(_) => match self.peripheral_state.take() {
                Some(can::State::Disabled) => {
                    self.schedule_callback(up_calls::UPCALL_DISABLE, (0, 0, 0));
                }
                Some(can::State::Error(err)) => {
                    self.schedule_callback(up_calls::UPCALL_DISABLE, (err as usize, 0, 0));
                }
                Some(can::State::Running) | None => {
                    self.schedule_callback(
                        up_calls::UPCALL_DISABLE,
                        (ErrorCode::FAIL as usize, 0, 0),
                    );
                }
            },
            Err(err) => {
                self.peripheral_state.take();
                self.schedule_callback(up_calls::UPCALL_ENABLE, (err as usize, 0, 0));
            }
        }
        self.processid.clear();
    }
}

impl<'a, Can: can::Can> can::TransmitClient<{ can::STANDARD_CAN_PACKET_SIZE }>
    for CanCapsule<'a, Can>
{
    // This callback is called when the hardware acknowledges that a message
    // was sent. This callback also makes an upcall to the userspace.
    fn transmit_complete(
        &self,
        status: Result<(), can::Error>,
        buffer: &'static mut [u8; can::STANDARD_CAN_PACKET_SIZE],
    ) {
        self.can_tx.replace(buffer);
        match status {
            Ok(()) => self.schedule_callback(up_calls::UPCALL_MESSAGE_SENT, (0, 0, 0)),
            Err(err) => {
                self.schedule_callback(
                    up_calls::UPCALL_TRANSMISSION_ERROR,
                    (error_upcalls::ERROR_TX, err as usize, 0),
                );
            }
        }
    }
}

impl<'a, Can: can::Can> can::ReceiveClient<{ can::STANDARD_CAN_PACKET_SIZE }>
    for CanCapsule<'a, Can>
{
    // This callback is called when a new message is received on any receiving
    // fifo.
    fn message_received(
        &self,
        id: can::Id,
        buffer: &mut [u8; can::STANDARD_CAN_PACKET_SIZE],
        len: usize,
        status: Result<(), can::Error>,
    ) {
        let mut new_buffer = false;
        let mut shared_len = 0;
        match status {
            Ok(_) => {
                match self.processid.map_or(Err(ErrorCode::NOMEM), |processid| {
                    self.processes
                        .enter(*processid, |app_data, kernel_data| {
                            kernel_data
                                .get_readwrite_processbuffer(rw_allow::RW_ALLOW_BUFFER)
                                .map_or_else(
                                    |err| err.into(),
                                    |buffer_ref| {
                                        buffer_ref
                                            .mut_enter(|user_buffer| {
                                                shared_len = user_buffer.len();
                                                // For now, the first 4 bytes (the size of u32) represent the number
                                                // of messages that the user has not read yet, represented as Little Endian.
                                                // When the userspace reads the buffer, the counter will be set
                                                // to 0 so that the capsule knows. This will be changed after
                                                // https://github.com/tock/tock/pull/3252 and
                                                // https://github.com/tock/tock/pull/3258 are merged.
                                                let mut tmp_buf: [u8; size_of::<u32>()] =
                                                    [0; size_of::<u32>()];
                                                user_buffer[0..size_of::<u32>()]
                                                    .copy_to_slice(&mut tmp_buf);
                                                let contor = u32::from_le_bytes(tmp_buf);
                                                if contor == 0 {
                                                    new_buffer = true;
                                                    app_data.receive_index = size_of::<u32>();
                                                }
                                                user_buffer[0..size_of::<u32>()]
                                                    .copy_from_slice(&(contor + 1).to_le_bytes());
                                                if app_data.receive_index + len > user_buffer.len()
                                                {
                                                    app_data.lost_messages =
                                                        app_data.lost_messages + 1;
                                                    Err(ErrorCode::SIZE)
                                                } else {
                                                    let r = user_buffer[app_data.receive_index
                                                        ..app_data.receive_index + len]
                                                        .copy_from_slice_or_err(&buffer[0..len]);
                                                    if r.is_ok() {
                                                        app_data.receive_index =
                                                            app_data.receive_index + len;
                                                    }
                                                    r
                                                }
                                            })
                                            .unwrap_or_else(|err| err.into())
                                    },
                                )
                        })
                        .unwrap_or_else(|err| err.into())
                }) {
                    Err(err) => self.schedule_callback(
                        up_calls::UPCALL_TRANSMISSION_ERROR,
                        (error_upcalls::ERROR_RX, err as usize, 0),
                    ),
                    Ok(_) => {
                        if new_buffer {
                            self.schedule_callback(
                                up_calls::UPCALL_MESSAGE_RECEIVED,
                                (
                                    0,
                                    shared_len as usize,
                                    match id {
                                        can::Id::Standard(u16) => u16 as usize,
                                        can::Id::Extended(u32) => u32 as usize,
                                    },
                                ),
                            )
                        }
                    }
                }
            }
            Err(err) => {
                let kernel_err: ErrorCode = err.into();
                self.schedule_callback(
                    up_calls::UPCALL_TRANSMISSION_ERROR,
                    (error_upcalls::ERROR_RX, kernel_err.into(), 0),
                )
            }
        };
    }

    fn stopped(&self, buffer: &'static mut [u8; can::STANDARD_CAN_PACKET_SIZE]) {
        self.can_rx.replace(buffer);
        self.schedule_callback(up_calls::UPCALL_RECEIVED_STOPPED, (0, 0, 0));
    }
}

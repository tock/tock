// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! IEEE 802.15.4 userspace interface for configuration and transmit/receive.
//!
//! Implements a userspace interface for sending and receiving raw IEEE 802.15.4
//! frames.
//!
//! Sending - Userspace fully forms the 15.4 frame and passes it to the driver.
//!
//! Receiving - The driver receives 15.4 frames and passes them to the process.
//! To accomplish this, the process must first `allow` a read/write ring buffer
//! to the kernel. The kernel will then fill this buffer with received frames
//! and schedule an upcall upon receipt of the first packet.
//!
//! The ring buffer provided by the process must be of the form:
//!
//! ```text
//! | read index | write index | user_frame 0 | user_frame 1 | ... | user_frame n |
//! ```
//!
//! `user_frame` denotes the 15.4 frame in addition to the relevant 3 bytes of
//! metadata (offset to data payload, length of data payload, and the MIC len).
//! The capsule assumes that this is the form of the buffer. Errors or deviation
//! in the form of the provided buffer will likely result in incomplete or
//! dropped packets.
//!
//! Because the scheduled receive upcall must be handled by the process, there
//! is no guarantee as to when this will occur and if additional packets will be
//! received prior to the upcall being handled. Without a ring buffer (or some
//! equivalent data structure), the original packet will be lost. The ring
//! buffer allows for the upcall to be scheduled and for all received packets to
//! be passed to the process. The ring buffer is designed to overwrite old
//! packets if the buffer becomes full. If the process notices a high number of
//! "dropped" packets, this may be the cause. The process can mitigate this
//! issue by increasing the size of the ring buffer provided to the capsule.

use kernel::grant::{AllowRoCount, AllowRwCount, Grant, UpcallCount};
use kernel::hil;
use kernel::processbuffer::{ReadableProcessBuffer, WriteableProcessBuffer};
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::{ErrorCode, ProcessId};

/// IDs for subscribed upcalls.
mod upcall {
    /// Frame is received
    pub const FRAME_RECEIVED: usize = 0;
    /// Frame is transmitted
    pub const FRAME_TRANSMITTED: usize = 1;
    /// Number of upcalls.
    pub const COUNT: u8 = 2;
}

/// Ids for read-only allow buffers
mod ro_allow {
    /// Write buffer. Contains the frame payload to be transmitted.
    pub const WRITE: usize = 0;
    /// The number of allow buffers the kernel stores for this grant
    pub const COUNT: u8 = 1;
}

/// Ids for read-write allow buffers
mod rw_allow {
    /// Read buffer. Will contain the received frame.
    pub const READ: usize = 0;
    /// The number of allow buffers the kernel stores for this grant
    pub const COUNT: u8 = 1;
}

use capsules_core::driver;
pub const DRIVER_NUM: usize = driver::NUM::Ieee802154 as usize;

#[derive(Default)]
pub struct App {
    pending_tx: bool,
}

pub struct RadioDriver<'a, R: hil::radio::Radio<'a>> {
    /// Underlying radio.
    radio: &'a R,

    /// Grant of apps that use this radio driver.
    apps: Grant<
        App,
        UpcallCount<{ upcall::COUNT }>,
        AllowRoCount<{ ro_allow::COUNT }>,
        AllowRwCount<{ rw_allow::COUNT }>,
    >,
    /// ID of app whose transmission request is being processed.
    current_app: OptionalCell<ProcessId>,

    /// Buffer that stores the IEEE 802.15.4 frame to be transmitted.
    kernel_tx: TakeCell<'static, [u8]>,
}

impl<'a, R: hil::radio::Radio<'a>> RadioDriver<'a, R> {
    pub fn new(
        radio: &'a R,
        grant: Grant<
            App,
            UpcallCount<{ upcall::COUNT }>,
            AllowRoCount<{ ro_allow::COUNT }>,
            AllowRwCount<{ rw_allow::COUNT }>,
        >,
        kernel_tx: &'static mut [u8],
    ) -> Self {
        Self {
            radio,
            apps: grant,
            current_app: OptionalCell::empty(),
            kernel_tx: TakeCell::new(kernel_tx),
        }
    }

    /// Performs `processid`'s pending transmission. Assumes that the driver is
    /// currently idle and the app has a pending transmission.
    fn perform_tx(&self, processid: ProcessId) -> Result<(), ErrorCode> {
        self.apps.enter(processid, |app, kernel_data| {
            app.pending_tx = false;

            self.kernel_tx.take().map_or(Err(ErrorCode::NOMEM), |kbuf| {
                kernel_data
                    .get_readonly_processbuffer(ro_allow::WRITE)
                    .and_then(|write| {
                        write.enter(|payload| {
                            let frame_len = payload.len();
                            let dst_start = hil::radio::PSDU_OFFSET;
                            let dst_end = dst_start + frame_len;
                            payload.copy_to_slice(&mut kbuf[dst_start..dst_end]);

                            self.radio.transmit(kbuf, frame_len).map_or_else(
                                |(errorcode, error_buf)| {
                                    self.kernel_tx.replace(error_buf);
                                    Err(errorcode)
                                },
                                |()| {
                                    self.current_app.set(processid);
                                    Ok(())
                                },
                            )
                        })
                    })?
            })
        })?
    }

    /// If the driver is currently idle and there are pending transmissions,
    /// pick an app with a pending transmission and return its `ProcessId`.
    fn get_next_tx_if_idle(&self) -> Option<ProcessId> {
        if self.current_app.is_some() {
            return None;
        }
        let mut pending_app = None;
        for app in self.apps.iter() {
            let processid = app.processid();
            app.enter(|app, _| {
                if app.pending_tx {
                    pending_app = Some(processid);
                }
            });
            if pending_app.is_some() {
                break;
            }
        }
        pending_app
    }

    /// Schedule the next transmission if there is one pending.
    fn do_next_tx(&self) {
        self.get_next_tx_if_idle()
            .map(|processid| match self.perform_tx(processid) {
                Ok(()) => {}
                Err(e) => {
                    let _ = self.apps.enter(processid, |_app, upcalls| {
                        let _ = upcalls.schedule_upcall(
                            upcall::FRAME_TRANSMITTED,
                            (kernel::errorcode::into_statuscode(Err(e)), 0, 0),
                        );
                    });
                }
            });
    }
}

impl<'a, R: hil::radio::Radio<'a>> SyscallDriver for RadioDriver<'a, R> {
    /// IEEE 802.15.4 low-level control.
    ///
    /// ### `command_num`
    ///
    /// - `0`: Driver existence check.
    /// - `1`: Return radio status. Ok(())/OFF = on/off.
    /// - `2`: Set short address.
    /// - `4`: Set PAN ID.
    /// - `5`: Set channel.
    /// - `6`: Set transmission power.
    /// - `7`: Commit any configuration changes.
    /// - `8`: Get the short MAC address.
    /// - `10`: Get the PAN ID.
    /// - `11`: Get the channel.
    /// - `12`: Get the transmission power.
    /// - `27`: Transmit a frame. The frame must be stored in the write RO allow
    ///   buffer 0. The allowed buffer must be the length of the frame. The
    ///   frame includes the PDSU (i.e., the MAC payload) _without_ the MFR
    ///   (i.e., CRC) bytes.
    /// - `28`: Set long address.
    /// - `29`: Get the long MAC address.
    /// - `30`: Turn the radio on.
    /// - `31`: Turn the radio off.
    fn command(
        &self,
        command_number: usize,
        arg1: usize,
        arg2: usize,
        processid: ProcessId,
    ) -> CommandReturn {
        match command_number {
            0 => CommandReturn::success(),
            1 => {
                if self.radio.is_on() {
                    CommandReturn::success()
                } else {
                    CommandReturn::failure(ErrorCode::OFF)
                }
            }
            2 => {
                self.radio.set_address(arg1 as u16);
                CommandReturn::success()
            }
            4 => {
                self.radio.set_pan(arg1 as u16);
                CommandReturn::success()
            }
            5 => {
                let channel = (arg1 as u8).try_into();
                channel.map_or(CommandReturn::failure(ErrorCode::INVAL), |chan| {
                    self.radio.set_channel(chan);
                    CommandReturn::success()
                })
            }
            6 => self.radio.set_tx_power(arg1 as i8).into(),
            7 => {
                self.radio.config_commit();
                CommandReturn::success()
            }
            8 => {
                // Guarantee that address is positive by adding 1
                let addr = self.radio.get_address();
                CommandReturn::success_u32(addr as u32 + 1)
            }
            10 => {
                // Guarantee that the PAN is positive by adding 1
                let pan = self.radio.get_pan();
                CommandReturn::success_u32(pan as u32 + 1)
            }
            11 => {
                let channel = self.radio.get_channel();
                CommandReturn::success_u32(channel as u32)
            }
            12 => {
                let txpower = self.radio.get_tx_power();
                CommandReturn::success_u32(txpower as u32)
            }
            27 => {
                self.apps
                    .enter(processid, |app, _| {
                        if app.pending_tx {
                            // Cannot support more than one pending TX per process.
                            return Err(ErrorCode::BUSY);
                        }
                        app.pending_tx = true;
                        Ok(())
                    })
                    .map_or_else(
                        |err| CommandReturn::failure(err.into()),
                        |_| {
                            self.do_next_tx();
                            CommandReturn::success()
                        },
                    )
            }
            28 => {
                let addr_upper: u64 = arg2 as u64;
                let addr_lower: u64 = arg1 as u64;
                let addr = addr_upper << 32 | addr_lower;
                self.radio.set_address_long(addr.to_be_bytes());
                CommandReturn::success()
            }
            29 => {
                let addr = u64::from_be_bytes(self.radio.get_address_long());
                CommandReturn::success_u64(addr)
            }
            30 => self.radio.start().into(),
            31 => self.radio.stop().into(),
            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::process::Error> {
        self.apps.enter(processid, |_, _| {})
    }
}

impl<'a, R: hil::radio::Radio<'a>> hil::radio::TxClient for RadioDriver<'a, R> {
    fn send_done(&self, spi_buf: &'static mut [u8], acked: bool, result: Result<(), ErrorCode>) {
        self.kernel_tx.replace(spi_buf);
        self.current_app.take().map(|processid| {
            let _ = self.apps.enter(processid, |_app, upcalls| {
                let _ = upcalls.schedule_upcall(
                    upcall::FRAME_TRANSMITTED,
                    (kernel::errorcode::into_statuscode(result), acked.into(), 0),
                );
            });
        });
        self.do_next_tx();
    }
}

impl<'a, R: hil::radio::Radio<'a>> hil::radio::RxClient for RadioDriver<'a, R> {
    fn receive<'b>(
        &self,
        buf: &'static mut [u8],
        frame_len: usize,
        lqi: u8,
        crc_valid: bool,
        result: Result<(), ErrorCode>,
    ) {
        // Drop invalid packets or packets that had errors during reception.
        if !crc_valid || result.is_err() {
            // Replace the RX buffer and drop the packet.
            self.radio.set_receive_buffer(buf);
            return;
        }

        self.apps.each(|_, _, kernel_data| {
            let read_present = kernel_data
                .get_readwrite_processbuffer(rw_allow::READ)
                .and_then(|read| {
                    read.mut_enter(|rbuf| {
                        ////////////////////////////////////////////////////////
                        // NOTE: context for the ring buffer and assumptions
                        // regarding the ring buffer format and usage can be
                        // found in the detailed comment at the top of this
                        // file.
                        //
                        // Ring buffer format:
                        //  | read  | write | user_frame | user_frame |...| user_frame |
                        //  | index | index | 0          | 1          |   | n          |
                        //
                        // user_frame format:
                        //  | header_len | payload_len | mic_len | 15.4 frame |
                        //
                        ////////////////////////////////////////////////////////

                        // 2 bytes for the readwrite buffer metadata (read and
                        // write index).
                        const RING_BUF_METADATA_SIZE: usize = 2;

                        /// 3 byte metadata (offset, len, mic_len)
                        const USER_FRAME_METADATA_SIZE: usize = 3;

                        /// 3 byte metadata + 127 byte max payload
                        const USER_FRAME_MAX_SIZE: usize =
                            USER_FRAME_METADATA_SIZE + hil::radio::MAX_FRAME_SIZE;

                        // Confirm the availability of the buffer. A buffer of
                        // len 0 is indicative of the userprocess not allocating
                        // a readwrite buffer. We must also confirm that the
                        // userprocess correctly formatted the buffer to be of
                        // length 2 + n * USER_FRAME_MAX_SIZE, where n is the
                        // number of user frames that the buffer can store. We
                        // combine checking the buffer's non-zero length and the
                        // case of the buffer being shorter than the
                        // `RING_BUF_METADATA_SIZE` as an invalid buffer (e.g.
                        // of length 1) may otherwise errantly pass the second
                        // conditional check (due to unsigned integer
                        // arithmetic).
                        if rbuf.len() <= RING_BUF_METADATA_SIZE
                            || (rbuf.len() - RING_BUF_METADATA_SIZE) % USER_FRAME_MAX_SIZE != 0
                        {
                            return false;
                        }

                        let mut read_index = rbuf[0].get() as usize;
                        let mut write_index = rbuf[1].get() as usize;

                        let max_pending_rx =
                            (rbuf.len() - RING_BUF_METADATA_SIZE) / USER_FRAME_MAX_SIZE;

                        // Confirm user modifiable metadata is valid (i.e.
                        // within bounds of the provided buffer).
                        if read_index >= max_pending_rx || write_index >= max_pending_rx {
                            return false;
                        }

                        // We don't parse the received packet, so we don't know
                        // how long all of the pieces are.
                        let mic_len = 0;
                        let header_len = 0;

                        // Start in the buffer where we are going to write this
                        // incoming packet.
                        let offset = RING_BUF_METADATA_SIZE + (write_index * USER_FRAME_MAX_SIZE);

                        // Copy the entire frame over to userland, preceded by
                        // three metadata bytes: the header length, the data
                        // length, and the MIC length.
                        let dst_start = offset + USER_FRAME_METADATA_SIZE;
                        let dst_end = dst_start + frame_len;
                        let src_start = hil::radio::PSDU_OFFSET;
                        let src_end = src_start + frame_len;
                        rbuf[dst_start..dst_end].copy_from_slice(&buf[src_start..src_end]);

                        rbuf[offset].set(header_len as u8);
                        rbuf[offset + 1].set(frame_len as u8);
                        rbuf[offset + 2].set(mic_len as u8);

                        // Prepare the ring buffer for the next write. The
                        // current design favors newness; newly received packets
                        // will begin to overwrite the oldest data in the event
                        // of the buffer becoming full. The read index must
                        // always point to the "oldest" data. If we have
                        // overwritten the oldest data, the next oldest data is
                        // now at the read index + 1. We must update the read
                        // index to reflect this.
                        write_index = (write_index + 1) % max_pending_rx;
                        if write_index == read_index {
                            read_index = (read_index + 1) % max_pending_rx;
                            rbuf[0].set(read_index as u8);
                        }

                        // Update write index metadata since we have added a
                        // frame.
                        rbuf[1].set(write_index as u8);
                        true
                    })
                })
                .unwrap_or(false);
            if read_present {
                // Place lqi as argument to be included in upcall.
                let _ = kernel_data.schedule_upcall(upcall::FRAME_RECEIVED, (lqi as usize, 0, 0));
            }
        });

        self.radio.set_receive_buffer(buf);
    }
}

impl<'a, R: hil::radio::Radio<'a>> hil::radio::ConfigClient for RadioDriver<'a, R> {
    fn config_done(&self, _result: Result<(), ErrorCode>) {}
}

impl<'a, R: hil::radio::Radio<'a>> hil::radio::PowerClient for RadioDriver<'a, R> {
    fn changed(&self, _on: bool) {}
}

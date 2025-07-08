// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Leon Schuermann <leon@is.currently.online> 2025.
// Copyright Tock Contributors 2025.

//! Userspace application TAP-like network driver
//!
//! This capsule provides a userspace syscall driver designed to expose a raw
//! IEEE 802.3 (Ethernet) compatible network interface (resembling a
//! Linux/FreeBSD TAP virtual network device).
//!
//! The driver multiplexes the underlying network interfaces across all
//! applications. Applications can allow a receive buffer to the kernel, and all
//! incoming frames will be placed into all applications' receive buffers. Every
//! application may transmit frames, which are sent on the underlying Ethernet
//! link.
//!
//! Incoming frames are sent to all processes (effectively mimicking the
//! behavior of a MacVTap device in Virtual Ethernet Port Aggregator (VEPA)
//! mode). This means that all processes can communicate over the Ethernet link,
//! but processes will generally not able to talk to each other, _unless_ the
//! connected far-end Ethernet device supports hairpin mode and is able to send
//! frames back on the link they came from. If processes should be able to talk
//! to each other, we can instantiate a hairpinning bridge in the Tock kernel,
//! or use a regular bridge and provide each process its own, dedicated tap
//! driver.
//!
//! ## Interface description
//!
//! This driver can both deliver incoming IEEE 802.3 Ethernet frames from the
//! backing [`EthernetAdapterDatapath`] interface to userspace applications, as
//! well as accept frames from the application for transmission over the
//! [`EthernetAdapterDatapath`] interface.
//!
//! The driver is built on top of the [`StreamingProcessSlice`] abstraction,
//! which allows for lossless streaming of data from the kernel to a userspace
//! application. It places incoming Ethernet frames into the
//! [`StreamingProcessSlice`], prefixed by a driver-specific header per
//! frame. This header contains each frames' length, allowing userspace to
//! efficiently find frame delimitors without parsing their content.
//!
//! The per-frame header has the following format:
//! - Bytes 0 to 1 (`u16`, native endian): flags
//!   - Bit 0: receive timestamp valid
//! - Bytes 2 to 3 (`u16`, native endian): frame length in bytes, excluding this header
//! - Bytes 4 to 11 (`u64`, native endian): receive timestamp
//! - Bytes 12 to (12 + frame length): frame contents, starting with the
//!   Ethernet header, up to and exluding the Frame Check Sequence (FCS)
//!
//! When one or more frames have been placed into the receive buffer, the driver
//! will schedule an upcall to the application. The application must follow the
//! regular [`StreamingProcessSlice`] semantics to swap the buffer for another
//! buffer atomically, to ensure lossless reception of incoming frames.
//!
//! Futhermore, userspace can transmit a frame by providing a buffer and
//! issuing a command-style system call. The interface can transmit at most a
//! single frame at any given time. Copying the to-be transmitted frame into
//! kernel memory is a synchronous operation, while the actual transmission may
//! be asynchronous. Thus an upcall will be issued to the application once the
//! transmission has commenced. Userspace must not issue a transmission while a
//! previous one has not been acknowledged by means of an upcall.
//!
//! To convey further information about transmitted frames, an application can
//! allow a "TX info" buffer into the kernel, which the driver can use to feed
//! additional metadata back to the transmitting application, such as the frame
//! transmission timestamp (if supported by the
//! [`EthernetAdapterDatapath`]). The layout of this buffer is specified below.
//!
//! ## System call interface
//!
//! ### Command-type system calls
//!
//! Implemented through [`EthernetTapDriver::command`].
//!
//! - **Command system call `0`**: Check if the driver is installed.
//!
//!   Returns the [`CommandReturn::success`] variant if the driver is installed,
//!   or [`CommandReturn::failure`] with associated ENOSUPPORT otherwise.
//!
//! - **Command system call `1`**: Query the interface RX statistics.
//!
//!   Returns [`CommandReturn::success_u32_u32_u32`] with a tuple of 32-bit
//!   counters local to the process: `(rx_frames, rx_bytes,
//!   rx_frames_dropped)`. Neither `rx_frames` not `rx_bytes` include any
//!   dropped frames. The counters will wrap at `u32::MAX`.
//!
//!   These counters are local to each process.
//!
//! - **Command system call `2`**: Query the interface TX statistics.
//!
//!   Returns [`CommandReturn::success_u32_u32`] with a tuple of 32-bit counters
//!   local to the process: `(tx_frames, tx_bytes)`. The counters will wrap at
//!   `u32::MAX`.
//!
//!   These counters are local to each process.
//!
//! - **Command system call `3`**: Transmit a frame located in the
//!   `frame_transmit_buffer`.
//!
//!   Arguments:
//!   1. transmit at most `n` bytes of the buffer (starting at `0`, limited by
//!      `MAX_MTU`, at most `u16::MAX` bytes). Supplied frames must start with
//!      the Ethernet header, and must not include the Frame Check Sequence
//!      (FCS).
//!   2. frame transmission identifier (`u32`): identifier passed back in the
//!      frame transmission upcall (upcall `2`).
//!
//!   Returns:
//!   - [`CommandReturn::success`] if the frame was queued for transmission
//!   - [`CommandReturn::failure`] with [`ErrorCode::BUSY`] if a frame is
//!     currently being transmitted (wait for upcall)
//!   - [`CommandReturn::failure`] with [`ErrorCode::SIZE`] if the TX buffer is
//!     to small to possibly contain the entire frame, or the passed frame
//!     exceeds the interface MTU.
//!
//! ## Subscribe-type system calls
//!
//! - **Upcall `0`**: _(currently not supported)_ Register an upcall to be
//!   informed when the driver was released by another process.
//!
//! - **Upcall `1`**: Register an upcall to be called when one or more frames
//!   have been placed into the receive [`StreamingProcessSlice`].
//!
//! - **Upcall `2`**: Register an upcall to be called when a frame transmission
//!   has been completed.
//!
//!   Upcall arguments:
//!   1. `statuscode` indicating 0 or ErrorCode
//!
//!   2. If `statuscode == 0`, flags and length (`u32`, native endian):
//!
//!      - bits `16..=30`: flags
//!        - bit 16: TX timestamp valid
//!
//!      - bits `0..=15`: transmitted bytes
//!
//!   3. frame transmission identifier (`u32`, native endian): identifier to
//!      associate an invocation of _command system call `3`_ to its
//!      corresponding upcall.
//!
//! ## Read-write allow type system calls:
//!
//! - **Read-write allow buffer `0`**: Allow a [`StreamingProcessSlice`] for
//!   received frames to be written by the driver.
//!
//! - **Read-write allow buffer `1`**: Allow a buffer for transmitted frame
//!   metadata to be written by the driver.
//!
//!   The allowed buffer must be at least 8 bytes in size. Individual fields may
//!   be valid or invalid, depending on the flags of the frame transmission
//!   upcall (upcall `2`) flags.
//!
//!   Layout:
//!   - bytes `0..=7`: frame transmission timestamp (u64; big endian)
//!
//!     Transmission timestamp of a frame provided by the underlying
//!     [`EthernetAdapterDatapath`] implementation.
//!
//! ## Read-only allow type system calls:
//!
//! - **Read-only allow buffer `0`**: Allow a buffer containing a frame to
//!   transmit by the driver. The frame transmission must still be scheduled by
//!   issuing the appropriate command system call (*command system call `6`*).

use kernel::errorcode::into_statuscode;
use kernel::grant::{AllowRoCount, AllowRwCount, Grant, GrantKernelData, UpcallCount};
use kernel::hil::ethernet::{EthernetAdapterDatapath, EthernetAdapterDatapathClient};
use kernel::processbuffer::{ReadableProcessBuffer, WriteableProcessBuffer};
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::utilities::cells::MapCell;
use kernel::utilities::streaming_process_slice::StreamingProcessSlice;
use kernel::ErrorCode;
use kernel::ProcessId;

/// Syscall driver number.
pub const DRIVER_NUM: usize = capsules_core::driver::NUM::EthernetTap as usize;

/// Maximum size of a frame which can be transmitted over the underlying
/// [`EthernetAdapterDatapath`] device.
///
/// Currently hard-coded to `1522 - 4` bytes, for an Ethernet frame with an
/// 802.1q VLAN tag with a 1500 byte payload MTU, excluding the 4-byte FCS.
pub const MAX_MTU: usize = 1518;

mod upcall {
    pub const RX_FRAME: usize = 0;
    pub const TX_FRAME: usize = 1;
    pub const COUNT: u8 = 2;
}

mod ro_allow {
    pub const TX_FRAME: usize = 0;
    pub const COUNT: u8 = 1;
}

mod rw_allow {
    pub const RX_FRAMES: usize = 0;
    pub const TX_FRAME_INFO: usize = 1;
    pub const COUNT: u8 = 2;
}

/// Receive streaming packet buffer frame header constants:
mod rx_frame_header {
    use core::ops::Range;

    pub const LENGTH: usize = 12;
    pub const FLAGS_BYTES: Range<usize> = 0_usize..2;
    pub const FRAME_LENGTH_BYTES: Range<usize> = 2_usize..4;
    pub const RECEIVE_TIMESTAMP_BYTES: Range<usize> = 4_usize..12;
}

#[derive(Default)]
pub struct App {
    // Per-process interface statistics:
    tx_frames: u32,
    tx_bytes: u32,

    rx_frames: u32,
    rx_bytes: u32,
    rx_frames_dropped: u32,
    rx_bytes_dropped: u32,

    // Pending transmission state:
    tx_pending: Option<(u16, u32)>,
}

enum TxState {
    Active { process: ProcessId },
    Inactive { buffer: &'static mut [u8] },
}

pub struct EthernetTapDriver<'a, E: EthernetAdapterDatapath<'a>> {
    /// The underlying [`EthernetAdapterDatapath`] network device
    iface: &'a E,

    /// Per-process state
    apps: Grant<
        App,
        UpcallCount<{ upcall::COUNT }>,
        AllowRoCount<{ ro_allow::COUNT }>,
        AllowRwCount<{ rw_allow::COUNT }>,
    >,

    /// Transmission buffer for frames to transmit from the application, or
    /// indicator which application is currently transmitting.
    tx_state: MapCell<TxState>,
}

impl<'a, E: EthernetAdapterDatapath<'a>> EthernetTapDriver<'a, E> {
    pub fn new(
        iface: &'a E,
        grant: Grant<
            App,
            UpcallCount<{ upcall::COUNT }>,
            AllowRoCount<{ ro_allow::COUNT }>,
            AllowRwCount<{ rw_allow::COUNT }>,
        >,
        tx_buffer: &'static mut [u8],
    ) -> Self {
        EthernetTapDriver {
            iface,
            apps: grant,
            tx_state: MapCell::new(TxState::Inactive { buffer: tx_buffer }),
        }
    }

    pub fn initialize(&self) {
        self.iface.enable_receive();
    }

    // This method must only be called when `tx_state` is currently `Inactive`
    // and the `grant` indicates that there is a frame to transmit
    // (`tx_pending.is_some()`).
    fn transmit_frame_from_grant(
        &self,
        tx_state: &mut TxState,
        process_id: ProcessId,
        grant: &App,
        kernel_data: &GrantKernelData<'_>,
    ) -> Result<(), ErrorCode> {
        // Mark this process as having an active transmission:
        let prev_state = core::mem::replace(
            tx_state,
            TxState::Active {
                process: process_id,
            },
        );

        let transmit_buffer = match prev_state {
            // This method must never be called during an active
            // transmission. If this code-path is reached, this represents a
            // driver-internal invariant violation.
            TxState::Active { .. } => unreachable!(),
            TxState::Inactive { buffer } => buffer,
        };

        // This method must never be called when a process does not have a
        // pending transmission:
        let (frame_len, transmission_identifier) = grant.tx_pending.unwrap();

        // Try to copy the frame from the allowed slice into the transmission
        // buffer. Return `ErrorCode::SIZE` if the `frame_len` exceeds the
        // allowed slice, the interface MTU, or the transmission buffer:
        let res = kernel_data
            .get_readonly_processbuffer(ro_allow::TX_FRAME)
            .and_then(|tx_frame| {
                tx_frame.enter(|data| {
                    if frame_len as usize
                        > core::cmp::min(data.len(), core::cmp::min(MAX_MTU, transmit_buffer.len()))
                    {
                        return Err(ErrorCode::SIZE);
                    }

                    // `frame_len` fits into source slice, destination buffer, and
                    // MTU, copy it:
                    data[..(frame_len as usize)]
                        .copy_to_slice(&mut transmit_buffer[..(frame_len as usize)]);

                    Ok(())
                })
            })
            .unwrap_or(Err(ErrorCode::FAIL));

        if let Err(e) = res {
            // We were unable to copy the frame, put the buffer back. The caller
            // is responsible for informing the process (via a sychronous return
            // value or upcall) and to reset the process' `tx_pending` field:
            *tx_state = TxState::Inactive {
                buffer: transmit_buffer,
            };
            return Err(e);
        }

        // Frame was copied, initiate transmission:
        if let Err((e, returned_buffer)) =
            self.iface
                .transmit_frame(transmit_buffer, frame_len, transmission_identifier as usize)
        {
            // We were unable to initiate the transmission put the buffer
            // back. The caller is responsible for informing the process (via a
            // sychronous return value or upcall) and to reset the process'
            // `tx_pending` field:
            *tx_state = TxState::Inactive {
                buffer: returned_buffer,
            };
            return Err(e);
        }

        Ok(())
    }

    // Attempt to complete a transmission requested by a process.
    //
    // If this process is still alive, it will reset its `tx_pending` state and
    // enqueue an upcall. Either case, it will return the `frame_buffer` and
    // reset the `tx_state` to `Inactive`.
    fn complete_transmission(
        &self,
        err: Result<(), ErrorCode>,
        frame_buffer: &'static mut [u8],
        len: u16,
        transmission_identifier: u32,
        timestamp: Option<u64>,
    ) {
        let process_id = self
            .tx_state
            .map(|tx_state| {
                // First, replace the `frame_buffer` and reset the `tx_state` to
                // `Inactive`. Also extract the currently transmitting process from
                // the `tx_state`, which must be `Active` when entering into this
                // callback:
                match core::mem::replace(
                    tx_state,
                    TxState::Inactive {
                        buffer: frame_buffer,
                    },
                ) {
                    TxState::Active { process } => process,
                    TxState::Inactive { .. } => panic!(),
                }
            })
            .unwrap();

        // Now, attempt to complete this process' transmission, if its still
        // alive and has subscribed to the transmission upcall / allowed a TX
        // info buffer respectively:
        let _ = self.apps.enter(process_id, |grant, kernel_data| {
            // If the application has allowed a "TX info" buffer, write some
            // transmission metadata to it:
            let ts_bytes = timestamp.map_or(u64::to_be_bytes(0), |ts| u64::to_be_bytes(ts));
            let _ = kernel_data
                .get_readwrite_processbuffer(rw_allow::TX_FRAME_INFO)
                .and_then(|tx_frame_info| {
                    tx_frame_info.mut_enter(|tx_info_buf| {
                        if tx_info_buf.len() >= 8 {
                            tx_info_buf[0..8].copy_from_slice(&ts_bytes);
                        }
                    })
                });

            // Encode combined flags / length upcall parameter:
            let flags_len: u32 = {
                let flags_bytes = u16::to_be_bytes((timestamp.is_some() as u16) << 0);

                let len_bytes = u16::to_be_bytes(len);

                u32::from_be_bytes([flags_bytes[0], flags_bytes[1], len_bytes[0], len_bytes[1]])
            };

            kernel_data
                .schedule_upcall(
                    upcall::TX_FRAME,
                    (
                        into_statuscode(err),
                        flags_len as usize,
                        transmission_identifier as usize,
                    ),
                )
                .ok();

            // Reset the `tx_pending` state of this app:
            grant.tx_pending = None;
        });
    }

    pub fn command_transmit_frame(
        &self,
        process_id: ProcessId,
        len: u16,
        transmission_identifier: u32,
    ) -> Result<(), ErrorCode> {
        self.apps
            .enter(process_id, |grant, kernel_data| {
                // Make sure that this process does not have another pending
                // transmission:
                if grant.tx_pending.is_some() {
                    return Err(ErrorCode::BUSY);
                }

                // Copy this command call's argument into the process' grant:
                grant.tx_pending = Some((len, transmission_identifier));

                // If we don't have an active transmission, try to enqueue this
                // frame for synchronous transmission:
                self.tx_state
                    .map(|tx_state| {
                        if matches!(tx_state, TxState::Inactive { .. }) {
                            if let Err(e) = self.transmit_frame_from_grant(
                                tx_state,
                                process_id,
                                grant,
                                kernel_data,
                            ) {
                                // Reset the `tx_pending` field:
                                grant.tx_pending = None;

                                // Return the error:
                                Err(e)
                            } else {
                                // Transmission initiated:
                                Ok(())
                            }
                        } else {
                            // Transmission is enqueued:
                            Ok(())
                        }
                    })
                    .unwrap()
            })
            .unwrap_or(Err(ErrorCode::FAIL))
    }
}

/// Userspace system call driver interface implementation
impl<'a, E: EthernetAdapterDatapath<'a>> SyscallDriver for EthernetTapDriver<'a, E> {
    fn command(
        &self,
        command_num: usize,
        arg1: usize,
        arg2: usize,
        process_id: ProcessId,
    ) -> CommandReturn {
        match command_num {
            // Check if driver is installed
            0 => CommandReturn::success(),

            // Query process-specific RX stats
            1 => self
                .apps
                .enter(process_id, |grant, _kernel_data| {
                    CommandReturn::success_u32_u32_u32(
                        grant.rx_frames,
                        grant.rx_bytes,
                        grant.rx_frames_dropped,
                    )
                })
                .unwrap_or(CommandReturn::failure(ErrorCode::FAIL)),

            // Query process-specific TX stats
            2 => self
                .apps
                .enter(process_id, |grant, _kernel_data| {
                    CommandReturn::success_u32_u32(grant.tx_frames, grant.tx_bytes)
                })
                .unwrap_or(CommandReturn::failure(ErrorCode::FAIL)),

            // Transmit frame from the `allow_ro::TX_FRAME` buffer:
            3 => match self.command_transmit_frame(process_id, arg1 as u16, arg2 as u32) {
                Ok(()) => CommandReturn::success(),
                Err(e) => CommandReturn::failure(e),
            },

            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::process::Error> {
        self.apps.enter(processid, |_, _| {})
    }
}

/// Callback client for the underlying [`EthernetAdapterDatapath`]:
impl<'a, E: EthernetAdapterDatapath<'a>> EthernetAdapterDatapathClient
    for EthernetTapDriver<'a, E>
{
    fn transmit_frame_done(
        &self,
        err: Result<(), ErrorCode>,
        frame_buffer: &'static mut [u8],
        len: u16,
        transmission_identifier: usize,
        timestamp: Option<u64>,
    ) {
        // Complete this transmission. This will reset both the process'
        // `tx_pending` state, and the global `tx_state`:
        self.complete_transmission(
            err,
            frame_buffer,
            len,
            transmission_identifier as u32,
            timestamp,
        );

        // Now, check for any processes that have a pending transmission. We
        // break out of the loop on the first successfully initiated
        // transmission, as we only have one transmit buffer for this capsule:
        self.tx_state
            .map(|tx_state| {
                for process_grant in self.apps.iter() {
                    let process_id = process_grant.processid();
                    let transmission_initiated = process_grant.enter(|grant, kernel_data| {
                        let (_, transmission_identifier) = match grant.tx_pending {
                            None => {
                                // Skip, this process does not have a pending transmission:
                                return false;
                            }
                            Some(tx_pending) => tx_pending,
                        };

                        // This process does have a pending transmission, submit it:
                        if let Err(e) =
                            self.transmit_frame_from_grant(tx_state, process_id, grant, kernel_data)
                        {
                            // Initializing the transmission failed. We must
                            // report this error through a callback, as no other
                            // callback will be raised for this transmission:
                            let _ = kernel_data.schedule_upcall(
                                upcall::TX_FRAME,
                                (
                                    e as usize,
                                    0, // flags are ignored in case of an error
                                    transmission_identifier as usize,
                                ),
                            );

                            // Remove the pending transmission from this process:
                            grant.tx_pending = None;

                            // This error may transient and be due to the
                            // particular enqueued transmission of this process
                            // (like exceeding the MTU). Try scheduling a
                            // transmission for the next pending process in the
                            // next loop iteration:
                            false
                        } else {
                            // Transmission initiated successfully:
                            true
                        }
                    });

                    if transmission_initiated {
                        // Can only initate one transmission:
                        break;
                    }

                    // Otherwise, continue:
                }
            })
            .unwrap();
    }

    fn received_frame(&self, frame: &[u8], timestamp: Option<u64>) {
        // Generate a header to prefix the frame contents placed in the
        // processes' `TX_FRAMES` [`StreamingProcessSlice`]:
        let mut frame_header = [0; rx_frame_header::LENGTH];

        // frame_header[0..2]: flags
        frame_header[rx_frame_header::FLAGS_BYTES]
            .copy_from_slice(&u16::to_ne_bytes((timestamp.is_some() as u16) << 0));

        let len_u16: u16 = match frame.len().try_into() {
            Ok(len) => len,
            Err(_) => {
                kernel::debug!(
                    "Incoming frame exceeds {} bytes ({} bytes), discarding.",
                    u16::MAX,
                    frame.len()
                );
                return;
            }
        };
        // frame_header[2..4]: length (excluding frame_header)
        frame_header[rx_frame_header::FRAME_LENGTH_BYTES]
            .copy_from_slice(&u16::to_ne_bytes(len_u16));

        // frame_header[4..12]: timestamp
        frame_header[rx_frame_header::RECEIVE_TIMESTAMP_BYTES]
            .copy_from_slice(&u64::to_ne_bytes(timestamp.unwrap_or(0)));

        // For each process, try to place the new frame (with header) into its
        // allowed streaming process slice. If at least one frame is contained
        // in the slice, generate an upcall, but only if one is not already
        // scheduled.
        self.apps.iter().for_each(|process_grant| {
            process_grant.enter(|grant, kernel_data| {
                let rx_frames_buffer =
                    match kernel_data.get_readwrite_processbuffer(rw_allow::RX_FRAMES) {
                        Ok(buf) => buf,
                        Err(_) => return,
                    };
                let _ = rx_frames_buffer.mut_enter(|rx_frames_slice| {
                    let rx_frames_streaming = StreamingProcessSlice::new(rx_frames_slice);

                    // The process has allowed a streaming process slice for
                    // incoming frames, attempt to place the frame and its
                    // header into the buffer:
                    let append_res = rx_frames_streaming
                        .append_chunk_from_iter(frame_header.iter().chain(frame.iter()).copied());

                    // We only schedule an upcall if we were able to append this
                    // chunk without error, and it was the first chunk we
                    // appended to this buffer (as other upcalls will be
                    // redundant -- the process will be able to observe
                    // subsequent frames in the buffer). In any case, we
                    // increment the per-process counters:
                    let first_chunk = match append_res {
                        Err(_) => {
                            // We weren't able to append the chunk, increment
                            // counters:
                            grant.rx_frames_dropped = grant.rx_frames_dropped.wrapping_add(1);
                            grant.rx_bytes_dropped =
                                grant.rx_bytes_dropped.wrapping_add(len_u16 as u32);
                            return;
                        }
                        Ok((first_chunk, _)) => {
                            // We successfully appended this chunk:
                            grant.rx_frames = grant.rx_frames.wrapping_add(1);
                            grant.rx_bytes = grant.rx_bytes.wrapping_add(len_u16 as u32);
                            first_chunk
                        }
                    };

                    // Schedule an upcall if this is the first non-zero length
                    // chunk appended to this slice:
                    if first_chunk {
                        let _ = kernel_data.schedule_upcall(upcall::RX_FRAME, (0, 0, 0));
                    }
                });
            });
        });
    }
}

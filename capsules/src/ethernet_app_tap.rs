//! # Userspace application TAP-like network driver
//!
//! This capsule provides a userspace syscall driver designed to expose a raw
//! IEEE 802.3 (Ethernet) compatible network interface (resembling a
//! Linux/FreeBSD TAP virtual network device).
//!
//! The driver is non-virtualized, meaning only a single userspace process can
//! access the driver at any given time. Access must be explicitly requested and
//! is then locked until the application relinquishes access or terminates.
//!
//! ## Interface description
//!
//! This driver can both deliver incoming IEEE 802.1 Ethernet packets from the
//! backing [`EthernetAdapter`] interface to a userspace application, as well as
//! accept packets from the application for transmission over the
//! [`EthernetAdapter`] interface.
//!
//! It provides a single upcall to userspace per delivered packet, storing it in
//! a provided buffer. The application has to acknowledge this upcall prior to
//! another packet being delivered. If the application is not able to keep up
//! with the rate of received packets, the driver may enqueue and/or drop
//! incoming packets. It will count dropped packets and provide this information
//! to the application.
//!
//! Futhermore, userspace can transmit a packet by providing a buffer and
//! issuing a command-style system call. The interface can transmit at most a
//! single packet at any given time. Copying the to-be transmitted packet into
//! kernel memory is a synchronous operation, while the actual transmission may
//! be asynchronous. Thus an upcall will be issued to the application once the
//! transmission has commenced. Userspace must not issue a transmission while a
//! previous one has not been acknowledged by means of an upcall.
//!
//! To convey further information about transmitted packets, an application can
//! allow a "TX info" buffer into the kernel, which the driver can use to supply
//! additional metadata, such as the packet transmission timestamp (if supported
//! by the [`EthernetAdapter`]). The application must explicitly acknowledge
//! reception of this data following a TX upcall, to allow the driver to reuse
//! this buffer for the next transmitted packet.
//!
//! ## System call interface
//!
//! ### Command-type system calls
//!
//! Implemented through [`TapDriver::command`].
//!
//! - **Command system call `0`**: Check if the driver is installed.
//!
//!   Returns the [`CommandReturn::success`] variant if the driver is installed,
//!   or [`CommandReturn::failure`] with associated ENOSUPPORT otherwise.
//!
//! - **Command system call `1`**: Try to acquire lock of the driver.
//!
//!   If this returns the [`CommandReturn::success`] variant, exclusive access
//!   to the drver is granted. Otherwise returns [`CommandReturn::failure`] with
//!   associated [`ErrorCode::BUSY`].
//!
//! - **Command system call `2`**: Release the lock of the driver.
//!
//!   This allows other processes to request usage of the driver.
//!
//!   Returns [`CommandReturn::success`] if the lock was previously held,
//!   [`CommandReturn::failure`] with associated [`ErrorCode::INVAL`] otherwise.
//!
//! - **Command system call `3`**: Query generic interface status.
//!
//!   Returns [`CommandReturn::success_u32_u32`], with
//!   1. boolean variable of whether the interface is currently
//!      physically "up",
//!   2. the maximum MTU that can be sent or received over this
//!      interface.
//!
//! - **Command system call `4`**: Query the interface RX statistics.
//!
//!   Returns [`CommandReturn::success_u32_u32_u32`] with a tuple of 32-bit
//!   counters local to the process: `(rx_packets, rx_bytes,
//!   rx_undelivered)`. The counters will wrap at `u32::MAX`.
//!
//! - **Command system call `5`**: Query the interface TX statistics.
//!
//!   Returns [`CommandReturn::success_u32_u32`] with a tuple of 32-bit counters
//!   local to the process: `(tx_packets, tx_bytes)`.
//!
//! - **Command system call `6`**: Transmit a packet located in the
//!   packet_transmit_buffer.
//!
//!   Arguments:
//!   1. transmit at most `n` bytes of the buffer (starting at `0`, limited by
//!      `MAX_MTU`, at most `u16::MAX` bytes)
//!   2. packet identifier (`u32`): identifier passed back in the packet
//!      transmission upcall (upcall `2`).
//!
//!   Returns:
//!   - [`CommandReturn::success`] if the packet was queued for transmission
//!   - [`CommandReturn::failure`] with [`ErrorCode::BUSY`] if a packet is
//!     currently being transmitted (wait for upcall)
//!   - [`CommandReturn::failure`] with [`ErrorCode::SIZE`] if the TX buffer is
//!     to small to possibly contain the entire packet, or the passed packet
//!     exceeds the interface MTU
//!
//! - **Command system call `7`**: Acknowledge a packet located in the
//!   `packet_receive_buffer` (read-write allow #0).
//!
//!   Issuing this command allows the driver to modify the packet in
//!   this buffer and place a new one.
//!
//!   **Important**: per TRD104, the application may only access the contents of
//!   a _read-write processbuffer_ when it is not simulatenously allowed to the
//!   kernel. Hence, the application is required to issue an "unallow" operation
//!   to process the received packet. However, the application **must** re-allow
//!   a buffer to the _read-write allow buffer slot `0`_ (receive packet buffer)
//!   **before** issuing this acknowledgement operation. The driver is incapable
//!   of performing an operation in response to an allow system call. To resume
//!   packet reception from queued receive packets, the driver must thus check
//!   and immediately start the reception into an allowed buffer in response to
//!   this acknowledgement system call. Otherwise an application may risk
//!   receiving packets delayed or not at all.
//!
//!   Returns [`CommandReturn::success`] if a non-acknowledged packet was
//!   located in the buffer, otherwise [`CommandReturn::failure`] with
//!   [`ErrorCode::ALREADY`].
//!
//! ## Subscribe-type system calls
//!
//! - **Upcall `0`**: _(currently not supported)_ Register an upcall to be
//!   informed when the driver was released by another process.
//!
//! - **Upcall `1`**: Register an upcall to be called when a new packet has been
//!   received.
//!
//!   Callback arguments:
//!   1. u32[16..=31] flags
//!      - bit `31`: RX error
//!
//!      - if _RX error_:
//!        - bit `16`: error reason
//!          - 0: unknown error
//!          - 1: receive buffer too small
//!      - else:
//!        - bit `17`: RX timestamp type: hardware = 1, software = 0
//!        - bit `16`: RX timestamp valid
//!
//!      - bits `0..=15`: received bytes (big-endian)
//!
//!   if not _RX error_:
//!
//!   2. u32: RX timestamp (high bits)
//!   3. u32: RX timestamp (low bits)
//!
//! - **Upcall `2`**: Register a callback to be called when a packet
//!   transmission has been completed.
//!
//!   Callback arguments:
//!   1. bits `16..=311: flags
//!      - bit `31`: TX error
//!
//!      - if _TX error_:
//!        - bit 16: error reason
//!          - 1: *reserved*
//!          - 0: unknown error
//!      - else:
//!        - bit 17: TX timestamp type: hardware = 1, software = 0
//!        - bit 16: TX timestamp valid
//!
//!     - bits `0..=15`: transmitted bytes (big endian)
//!
//!   2. packet identifier (`u32`): identifier to associate an invocation of
//!      _command system call `6`_ to its corresponding upcall.
//!
//! ## Read-write allow type system calls:
//!
//! - **Read-write allow buffer `0`**: Allow a buffer for received packets to be
//!   written by the driver. It must be sufficiently large to accomodate the
//!   received packets.
//!
//! - **Read-write allow buffer `1`**: Allow a buffer for transmitted packet
//!   metadata to be written by the driver.
//!
//!   The allowed buffer must be at least 8 bytes in size. Individual fields may
//!   be valid or invalid, depending on the flags of the packet transmission
//!   upcall (upcall `2`) flags.
//!
//!   Layout:
//!   - bytes `0..=7`: packet timestamp (u64; big endian)
//!
//!     Transmission timestamp of a packet provided by the underlying
//!     [`EthernetAdapter`] implementation.
//!
//! ## Read-only allow type system calls:
//!
//! - **Read-only allow buffer `0`**: Allow a buffer containing a packet to
//!   transmit by the driver. The packet transmission must still be scheduled by
//!   issuing the appropriate command system call (*command system call `6`*).

use core::cell::Cell;
use kernel::collections::queue::Queue;
use kernel::collections::ring_buffer::RingBuffer;
use kernel::grant::{AllowRoCount, AllowRwCount, Grant, UpcallCount};
use kernel::hil::ethernet::{EthernetAdapter, EthernetAdapterClient};
use kernel::processbuffer::{ReadableProcessBuffer, WriteableProcessBuffer};
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::utilities::cells::{MapCell, TakeCell};
use kernel::ErrorCode;
use kernel::ProcessId;

/// Syscall driver number.
pub const DRIVER_NUM: usize = crate::driver::NUM::EthernetTAP as usize;

/// Maximum size of a packet which can be transmitted over the underlying
/// [`EthernetAdapter`] device. Currently hard-coded to `1522` bytes.
pub const MAX_MTU: usize = 1522;

mod upcall {
    pub const _DRIVER_RELEASED: usize = 0;
    pub const RX_PACKET: usize = 1;
    pub const TX_PACKET: usize = 2;
    pub const COUNT: u8 = 3;
}

mod ro_allow {
    pub const TX_PACKET: usize = 0;
    pub const COUNT: u8 = 1;
}

mod rw_allow {
    pub const RX_PACKET: usize = 0;
    pub const TX_PACKET_INFO: usize = 1;
    pub const COUNT: u8 = 2;
}

pub struct App {
    // Per-process interface statistics
    tx_packets: u32,
    tx_bytes: u32,
    rx_packets: u32,
    rx_bytes: u32,
    rx_packets_missed: u32,
    // Per-process interface state
    rx_acked: bool,
    tx_info_acked: bool,
}

impl Default for App {
    fn default() -> Self {
        App {
            tx_packets: 0,
            tx_bytes: 0,
            rx_packets: 0,
            rx_bytes: 0,
            rx_packets_missed: 0,
            rx_acked: true,
            tx_info_acked: true,
        }
    }
}

pub struct TapDriver<'a, E: EthernetAdapter<'a>> {
    /// The underlying [`EthernetAdapter`] network device
    nic: &'a E,

    /// Per-process state
    apps: Grant<
        App,
        UpcallCount<{ upcall::COUNT }>,
        AllowRoCount<{ ro_allow::COUNT }>,
        AllowRwCount<{ rw_allow::COUNT }>,
    >,

    /// Exclusive process-lock state
    current_app: Cell<Option<ProcessId>>,

    /// Ring-buffer for incoming packets which cannot be written to the
    /// application-provided buffer immediately.
    ///
    /// Must be emptied when switching apps.
    rx_packets: MapCell<RingBuffer<'static, ([u8; MAX_MTU], u16, Option<u64>, bool)>>,

    /// Transmission buffer for packets to transmit from the application. Should
    /// be able to hold at least [`MAX_MTU`].
    tx_buffer: TakeCell<'static, [u8]>,
}

impl<'a, E: EthernetAdapter<'a>> TapDriver<'a, E> {
    pub fn new(
        nic: &'a E,
        grant: Grant<
            App,
            UpcallCount<{ upcall::COUNT }>,
            AllowRoCount<{ ro_allow::COUNT }>,
            AllowRwCount<{ rw_allow::COUNT }>,
        >,
        tx_buffer: &'static mut [u8],
        rx_packets: &'static mut [([u8; MAX_MTU], u16, Option<u64>, bool)],
    ) -> Self {
        TapDriver {
            nic,
            apps: grant,
            rx_packets: MapCell::new(RingBuffer::new(rx_packets)),
            current_app: Cell::new(None),
            tx_buffer: TakeCell::new(tx_buffer),
        }
    }

    /// Efficiently check whether the passed `process_id` has acquired the lock.
    ///
    /// This does not perform any liveness-checks and assumes that the passed
    /// `process_id` is alive. Sufficient to validate that an issued command
    /// system call by the passed `process_id` is allowed to be processed.
    fn lock_acquired(&self, process_id: ProcessId) -> bool {
        if let Some(lock_pid) = self.current_app.get() {
            process_id == lock_pid
        } else {
            false
        }
    }

    /// Checks whether the process currently holding the lock is alive. If it is
    /// not, this further cleans up the `current_app` state.
    fn lock_process_alive(&self) -> bool {
        if let Some(process_id) = self.current_app.get() {
            let res = self.apps.enter(process_id, |_, _| true).unwrap_or(false);

            if !res {
                // We know the process has died, clean up the current
                // app and its associated state
                self.current_app.set(None);
                self.rx_packets.map(|rb| rb.empty());
            }

            res
        } else {
            false
        }
    }

    /// Initiate transmission of a packet provided by userspace in the read-only
    /// process buffer.
    fn transmit_packet(
        &self,
        process_id: ProcessId,
        len: u16,
        packet_identifier: usize,
    ) -> Result<(), ErrorCode> {
        self.apps
            .enter(process_id, |_grant, kernel_data| {
                let nic_tx_buffer = self.tx_buffer.take().ok_or(ErrorCode::BUSY)?;

                let process_buffer_len = kernel_data
                    .get_readonly_processbuffer(ro_allow::TX_PACKET)
                    .and_then(|tx_packet| Ok(tx_packet.len()))
                    .unwrap_or(0);

                if (len as usize) > nic_tx_buffer.len()
                    || (len as usize) > MAX_MTU
                    || (len as usize) > process_buffer_len
                {
                    self.tx_buffer.replace(nic_tx_buffer);
                    return Err(ErrorCode::SIZE);
                }

                // The length of the kernel buffer is sufficient to transmit
                // this packet, copy it into the kernel-internal buffer:
                let res = kernel_data
                    .get_readonly_processbuffer(ro_allow::TX_PACKET)
                    .and_then(|tx_packet| {
                        tx_packet.enter(|data| {
                            data[0..(len as usize)]
                                .copy_to_slice(&mut nic_tx_buffer[0..(len as usize)]);
                            Ok(())
                        })
                    })
                    .unwrap_or(Err(ErrorCode::FAIL));

                if let Err(e) = res {
                    // We were unable to copy the slice, put the buffer back:
                    self.tx_buffer.replace(nic_tx_buffer);
                    return Err(e);
                }

                // We've got the packet copied into the buffer, start
                // transmission:
                if let Err((e, returned_buffer)) =
                    self.nic.transmit(nic_tx_buffer, len, packet_identifier)
                {
                    // An error occured while trying to transmit, put the buffer
                    // back:
                    self.tx_buffer.replace(returned_buffer);
                    return Err(e);
                }

                Ok(())
            })
            .unwrap_or(Err(ErrorCode::FAIL))
    }

    /// Acknowledge a received packet placed into the read-write
    /// processbuffer. This allows the kernel to re-use the buffer for a new
    /// packet.
    fn acknowledge_rx_packet(&self, process_id: ProcessId) -> Result<(), ErrorCode> {
        self.apps
            .enter(process_id, |grant, _| {
                if grant.rx_acked {
                    return Err(ErrorCode::ALREADY);
                }

                grant.rx_acked = true;

                Ok(())
            })
            .unwrap_or(Err(ErrorCode::FAIL))
            .and_then(|_| {
                // Check if other packets are remaining in the RingBuffer and
                // write them to the processbuffer, scheduling a callback.
                self.rx_packets.map(|ring_buffer| {
                    if let Some((buffer, len, ts, ts_src)) = ring_buffer.dequeue() {
                        let _ = self.write_packet(process_id, &buffer, len, ts, ts_src);
                    }
                });

                Ok(())
            })
    }

    /// Acknowledge the "TX info" message written to the respective read-write
    /// processbuffer.
    ///
    /// The kernel will still be able to transmit packets even without this
    /// acknowledgement, however newly transmitted packets will not write to the
    /// allowed processbuffer before issuing the application upcall.
    fn acknowledge_tx_info(&self, process_id: ProcessId) -> Result<(), ErrorCode> {
        self.apps
            .enter(process_id, |grant, _| {
                if grant.tx_info_acked {
                    return Err(ErrorCode::ALREADY);
                }

                grant.tx_info_acked = true;

                Ok(())
            })
            .unwrap_or(Err(ErrorCode::FAIL))
    }

    /// Write a received packet into the allowed read-write process buffer.
    fn write_packet(
        &self,
        process_id: ProcessId,
        packet: &[u8],
        len: u16,
        timestamp: Option<u64>,
        timestamp_src: bool,
    ) -> Result<(), ErrorCode> {
        if packet.len() < len as usize {
            // The passed buffer must hold the entire packet
            return Err(ErrorCode::FAIL);
        }

        self.apps
            .enter(process_id, |grant, kernel_data| {
                // Regardless of any potential packet reception errors, count
                // this packet towards the statistics
                grant.rx_packets = grant.rx_packets.wrapping_add(1);
                grant.rx_bytes = grant.rx_bytes.wrapping_add(len as u32);

                // Determine the length of the application's packet reception
                // buffer:
                let app_rx_len = kernel_data
                    .get_readwrite_processbuffer(rw_allow::RX_PACKET)
                    .and_then(|rx_packet| Ok(rx_packet.len()))
                    .unwrap_or(0);

                // If the app's allowed buffer is of insufficent size, deliver
                // an error callback with the respective error code set:
                if app_rx_len < len as usize {
                    grant.rx_packets_missed = grant.rx_packets_missed.wrapping_add(1);
                    kernel_data
                        .schedule_upcall(
                            upcall::RX_PACKET,
                            (
                                0
                             | 1 << 31  // RX error
                             | 1 << 16, // error: receive buffer too small
                                0,
                                0,
                            ),
                        )
                        .ok();
                    return Err(ErrorCode::NOMEM);
                }

                // The buffer has sufficient size, copy the packet:
                kernel_data
                    .get_readwrite_processbuffer(rw_allow::RX_PACKET)
                    .and_then(|rx_packet| {
                        rx_packet.mut_enter(|data| {
                            data[0..(len as usize)].copy_from_slice(&packet[0..(len as usize)]);
                        })
                    })
                    .map_err(|_| ErrorCode::NOMEM)?;

                // Set the internal state to be non-acked, such that
                // we won't try to write more packets to the buffer:
                grant.rx_acked = false;

                // Inform the application:
                let ts_bytes: [u8; 8] =
                    timestamp.map_or(u64::to_be_bytes(0), |ts| u64::to_be_bytes(ts));
                let status_bytes: [u8; 2] = [
                    0,
                    (if timestamp.is_some() { 1 << 0 } else { 0 })
                        | (if timestamp_src { 1 << 1 } else { 0 }),
                ];
                let length_bytes: [u8; 2] = u16::to_be_bytes(len);

                kernel_data
                    .schedule_upcall(
                        upcall::RX_PACKET,
                        (
                            u32::from_be_bytes([
                                status_bytes[0],
                                status_bytes[1],
                                length_bytes[0],
                                length_bytes[1],
                            ]) as usize,
                            u32::from_be_bytes([ts_bytes[0], ts_bytes[1], ts_bytes[2], ts_bytes[3]])
                                as usize,
                            u32::from_be_bytes([ts_bytes[4], ts_bytes[5], ts_bytes[6], ts_bytes[7]])
                                as usize,
                        ),
                    )
                    .ok();

                Ok(())
            })
            .unwrap_or(Err(ErrorCode::NOMEM))
    }
}

/// This is the callback client for the underlying
/// [`EthernetAdapter`]:
impl<'a, E: EthernetAdapter<'a>> EthernetAdapterClient for TapDriver<'a, E> {
    fn tx_done(
        &self,
        err: Result<(), ErrorCode>,
        packet_buffer: &'static mut [u8],
        len: u16,
        packet_identifier: usize,
        timestamp: Option<u64>,
    ) {
        // Regardless of any checks below, put the kernel buffer back:
        self.tx_buffer.replace(packet_buffer);

        // Try to issue an upcall to the application:
        if let Some(process_id) = self.current_app.get() {
            let _ = self.apps.enter(process_id, |grant, kernel_data| {
                let ts_bytes = timestamp.map_or(u64::to_be_bytes(0), |ts| u64::to_be_bytes(ts));

                let tx_info_written = if grant.tx_info_acked && timestamp.is_some() {
                    // The application has acknowledged the
                    // previous "TX info" message written to the
                    // allowed process buffer and we have a
                    // timestamp. Attempt to write it to the
                    // process buffer:
                    let written = kernel_data
                        .get_readwrite_processbuffer(rw_allow::TX_PACKET_INFO)
                        .and_then(|tx_packet_info| {
                            tx_packet_info.mut_enter(|tx_info_buf| {
                                if tx_info_buf.len() < 8 {
                                    false
                                } else {
                                    tx_info_buf[0..8].copy_from_slice(&ts_bytes);
                                    true
                                }
                            })
                        })
                        .unwrap_or(false);

                    // If we managed to encode the "TX info"
                    // message into the process buffer, mark it as
                    // unacknowledged:
                    if written {
                        grant.tx_info_acked = false;
                    }

                    written
                } else {
                    // The app hasn't acked the previous tx_info,
                    // so we don't know where to put this OR we
                    // don't have any information we even could
                    // pass to the app
                    false
                };

                // Encode first upcall parameter:
                let (status_bytes, len_bytes) = match err {
                    Ok(()) => (
                        [
                            if tx_info_written { 1 << 6 } else { 0 },
                            if timestamp.is_some() {
                                1 << 1 | 1 << 0
                            } else {
                                0
                            },
                        ],
                        u16::to_be_bytes(len),
                    ),
                    Err(_) => ([1 << 7, 0], [0, 0]),
                };

                kernel_data
                    .schedule_upcall(
                        upcall::TX_PACKET,
                        (
                            u32::from_be_bytes([
                                status_bytes[0],
                                status_bytes[1],
                                len_bytes[0],
                                len_bytes[1],
                            ]) as usize,
                            packet_identifier as usize,
                            0,
                        ),
                    )
                    .ok();
            });
        }
    }

    fn rx_packet(&self, packet: &[u8], timestamp: Option<u64>) {
        if let Some(process_id) = self.current_app.get() {
            // Check whether we can attempt to write the packet to the
            // application directly, or whether it is going to be
            // enqueued into the pending packets ring-buffer.
            let write_packet_now = self
                .apps
                .enter(process_id, |grant, _kernel_data| {
                    // Check if the app has already acknowledged the
                    // most recent reception:
                    if grant.rx_acked {
                        // The app has acknowledged the must recent
                        // reception, so we can safely attempt to write
                        // into the app buffer

                        // Deliver the packet directly
                        true
                    } else {
                        // We can't write the packet directly to the app,
                        // instead write it to the ring buffer
                        self.rx_packets
                            .map(|ring_buffer| {
                                // TODO: avoid creating this on the stack
                                let mut pbuf: [u8; MAX_MTU] = [0; MAX_MTU];

                                if packet.len() > pbuf.len() || ring_buffer.is_full() {
                                    grant.rx_packets_missed += 1;

                                    // Don't deliver the packet directly
                                    false
                                } else {
                                    pbuf[0..(packet.len())].copy_from_slice(packet);
                                    ring_buffer.enqueue((
                                        pbuf,
                                        packet.len() as u16,
                                        timestamp,
                                        timestamp.is_some(),
                                    ));

                                    // Don't deliver the packet directly
                                    false
                                }
                            })
                            .unwrap()
                    }
                })
                .unwrap_or(false);

            // The app should have space in its allowed processbuffer,
            // hence attempt to deliver the packet:
            if write_packet_now {
                let _ = self.write_packet(process_id, packet, packet.len() as u16, timestamp, true);
            }
        }
    }
}

/// Userspace system call driver interface implementation
impl<'a, E: EthernetAdapter<'a>> SyscallDriver for TapDriver<'a, E> {
    fn command(
        &self,
        command_num: usize,
        arg1: usize,
        arg2: usize,
        process_id: ProcessId,
    ) -> CommandReturn {
        match command_num {
            0 /* Check if driver is installed */ => CommandReturn::success(),

            1 /* Acquire lock of the driver */ => {
                if !self.lock_process_alive() {
                    self.current_app.set(Some(process_id));
                    CommandReturn::success()
                } else {
                    CommandReturn::failure(ErrorCode::BUSY)
                }
            },

            2 /* Release the driver */ => {
                if self.lock_acquired(process_id) {
                    self.current_app.set(None);
                    self.rx_packets.map(|rb| rb.empty());
                    // TODO: call callback 0 on all (other) apps
                    CommandReturn::success()
                } else {
                    CommandReturn::failure(ErrorCode::INVAL)
                }
            },

            3 /* Query generic interface status */ => {
                // TODO: determine if interface is "up"
                let ifup = true;
                CommandReturn::success_u32_u32(ifup as u32, MAX_MTU as u32)
            },

            4 /* Query process-specific RX stats */ => {
                self.apps.enter(process_id, |grant, _kernel_data| {
                    CommandReturn::success_u32_u32_u32(
                        grant.rx_packets,
                        grant.rx_bytes,
                        grant.rx_packets_missed,
                    )
                }).unwrap_or(CommandReturn::failure(ErrorCode::FAIL))
            },

            5 /* Query process-specific TX stats */ => {
                self.apps.enter(process_id, |grant, _kernel_data| {
                    CommandReturn::success_u32_u32(
                        grant.tx_packets,
                        grant.tx_bytes,
                    )
                }).unwrap_or(CommandReturn::failure(ErrorCode::FAIL))
            },

            6 /* Transmit packet of the packet_transmit_buffer */ => {
                if self.lock_acquired(process_id) {
                    match self.transmit_packet(process_id, arg1 as u16, arg2 as usize) {
                        Ok(()) => CommandReturn::success(),
                        Err(e) => CommandReturn::failure(e),
                    }
                } else {
                    CommandReturn::failure(ErrorCode::RESERVE)
                }
            },

            7 /* Acknowledge packet in packet_receive_buffer */ => {
                if self.lock_acquired(process_id) {
                    match self.acknowledge_rx_packet(process_id) {
                        Ok(()) => CommandReturn::success(),
                        Err(e) => CommandReturn::failure(e),
                    }
                } else {
                    CommandReturn::failure(ErrorCode::RESERVE)
                }
            },

            8 /* Acknowledge reception of the tx_info blob */ => {
                if self.lock_acquired(process_id) {
                    match self.acknowledge_tx_info(process_id) {
                        Ok(()) => CommandReturn::success(),
                        Err(e) => CommandReturn::failure(e),
                    }
                } else {
                    CommandReturn::failure(ErrorCode::RESERVE)
                }
            },

            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::process::Error> {
        self.apps.enter(processid, |_, _| {})
    }
}

//! Virtual IEEE 802.15.4 MAC device
//!
//! `MuxMac` provides multiplexed access to an 802.15.4 MAC device. This enables
//! a single underlying 802.15.4 radio to be shared transparently by multiple
//! users. For example, the kernel might want to send raw 802.15.4 frames and
//! subsequently 6LoWPAN-encoded and fragmented IP packets. This capsule allows
//! that to happen by providing a mechanism for sequencing transmission attempts,
//! Every radio frame received is provided to all listening clients so that each
//! client can perform its own frame filtering logic.
//!
//! Usage
//! -----
//!
//! ```
//! # use kernel::static_init;
//!
//! // Create the mux.
//! let mux_mac = static_init!(
//!     capsules::ieee802154::virtual_mac::MuxMac<'static>,
//!     capsules::ieee802154::virtual_mac::MuxMac::new(&'static mac_device));
//! mac_device.set_transmit_client(mux_mac);
//! mac_device.set_receive_client(mux_mac);
//!
//! // Everything that uses the virtualized MAC device must create one of these.
//! let virtual_mac = static_init!(
//!     capsules::ieee802154::virtual_mac::MacUser<'static>,
//!     capsules::ieee802154::virtual_mac::MacUser::new(mux_mac));
//! mux_mac.add_user(virtual_mac);
//! ```

use crate::ieee802154::{device, framer};
use crate::net::ieee802154::{Header, KeyId, MacAddress, PanID, SecurityLevel};
use core::cell::Cell;
use kernel::common::cells::{MapCell, OptionalCell};
use kernel::common::{List, ListLink, ListNode};
use kernel::ReturnCode;

/// IEE 802.15.4 MAC device muxer that keeps a list of MAC users and sequences
/// any pending transmission requests. Any received frames from the underlying
/// MAC device are sent to all users.
pub struct MuxMac<'a> {
    mac: &'a dyn device::MacDevice<'a>,
    users: List<'a, MacUser<'a>>,
    inflight: OptionalCell<&'a MacUser<'a>>,
}

impl device::TxClient for MuxMac<'_> {
    fn send_done(&self, spi_buf: &'static mut [u8], acked: bool, result: ReturnCode) {
        self.inflight.take().map(move |user| {
            user.send_done(spi_buf, acked, result);
        });
        self.do_next_op_async();
    }
}

impl device::RxClient for MuxMac<'_> {
    fn receive<'b>(&self, buf: &'b [u8], header: Header<'b>, data_offset: usize, data_len: usize) {
        for user in self.users.iter() {
            user.receive(buf, header, data_offset, data_len);
        }
    }
}

impl<'a> MuxMac<'a> {
    pub const fn new(mac: &'a dyn device::MacDevice<'a>) -> MuxMac<'a> {
        MuxMac {
            mac: mac,
            users: List::new(),
            inflight: OptionalCell::empty(),
        }
    }

    /// Registers a MAC user with this MAC mux device. Each MAC user should only
    /// be registered once.
    pub fn add_user(&self, user: &'a MacUser<'a>) {
        self.users.push_head(user);
    }

    /// Gets the next `MacUser` and operation to perform if an operation is not
    /// already underway.
    fn get_next_op_if_idle(&self) -> Option<(&'a MacUser<'a>, Op)> {
        if self.inflight.is_some() {
            return None;
        }

        let mnode = self.users.iter().find(|node| {
            node.operation.take().map_or(false, |op| {
                let pending = op != Op::Idle;
                node.operation.replace(op);
                pending
            })
        });
        mnode.and_then(|node| {
            node.operation.take().map(|op| {
                node.operation.replace(Op::Idle);
                (node, op)
            })
        })
    }

    /// Performs a non-idle operation on a `MacUser` asynchronously: that is, if the
    /// transmission operation results in immediate failure, then return the
    /// buffer to the `MacUser` via its transmit client.
    fn perform_op_async(&self, node: &'a MacUser<'a>, op: Op) {
        if let Op::Transmit(frame) = op {
            let (result, mbuf) = self.mac.transmit(frame);
            // If a buffer is returned, the transmission failed,
            // otherwise it succeeded.
            mbuf.map_or_else(
                || {
                    self.inflight.set(node);
                },
                |buf| {
                    node.send_done(buf, false, result);
                },
            )
        }
    }

    /// Performs a non-idle operation on a `MacUser` synchronously, returning
    /// the error code and the buffer immediately.
    fn perform_op_sync(
        &self,
        node: &'a MacUser<'a>,
        op: Op,
    ) -> Option<(ReturnCode, Option<&'static mut [u8]>)> {
        if let Op::Transmit(frame) = op {
            let (result, mbuf) = self.mac.transmit(frame);
            if result == ReturnCode::SUCCESS {
                self.inflight.set(node);
            }
            Some((result, mbuf))
        } else {
            None
        }
    }

    /// Begins the next outstanding transmission if there is no ongoing
    /// operation and there is a user waiting to transmit a frame.
    /// Since this is being called asynchronously, return any buffers to the active
    /// `tx_client` via the `send_done` callback in the event of failure.
    fn do_next_op_async(&self) {
        self.get_next_op_if_idle()
            .map(|(node, op)| self.perform_op_async(node, op));
    }

    /// Begins the next outstanding transmission if there is no ongoing
    /// operation and there is a user waiting to transmit a frame. Since this is
    /// being called synchronously, there is a need to identify the MacUser that
    /// just queued its transmission request. This can only be done by comparing
    /// the raw pointer references of the two users, since there is no
    /// type-level way to guarantee that the enqueued user is actually in this Mux device's
    /// `users` list. It's safe because the raw pointer references are never
    /// dereferenced.
    ///
    /// If the newly-enqueued transmission is immediately executed by this mux
    /// device but fails immediately, return the buffer synchronously.
    fn do_next_op_sync(
        &self,
        new_node: &MacUser<'a>,
    ) -> Option<(ReturnCode, Option<&'static mut [u8]>)> {
        self.get_next_op_if_idle().and_then(|(node, op)| {
            if node as *const _ == new_node as *const _ {
                // The new node's operation is the one being scheduled, so the
                // operation is synchronous
                self.perform_op_sync(node, op)
            } else {
                // The operation being scheduled is not the new node, so the
                // operation is asynchronous with respect to the new node.
                self.perform_op_async(node, op);
                None
            }
        })
    }
}

#[derive(Eq, PartialEq, Debug)]
enum Op {
    Idle,
    Transmit(framer::Frame),
}

/// Keep state for each Mac user. All users of the virtualized MAC interface
/// need to create one of these and register it with the MAC device muxer
/// `MuxMac` by calling `MuxMac#add_user`. Then, each `MacUser` behaves exactly
/// like an independent MAC device, except MAC device state is shared between
/// all MacUsers because there is only one MAC device. For example, the MAC
/// device address is shared, so calling `set_address` on one `MacUser` sets the
/// MAC address for all `MacUser`s.
pub struct MacUser<'a> {
    mux: &'a MuxMac<'a>,
    operation: MapCell<Op>,
    next: ListLink<'a, MacUser<'a>>,
    tx_client: Cell<Option<&'a dyn device::TxClient>>,
    rx_client: Cell<Option<&'a dyn device::RxClient>>,
}

impl<'a> MacUser<'a> {
    pub const fn new(mux: &'a MuxMac<'a>) -> MacUser<'a> {
        MacUser {
            mux: mux,
            operation: MapCell::new(Op::Idle),
            next: ListLink::empty(),
            tx_client: Cell::new(None),
            rx_client: Cell::new(None),
        }
    }
}

impl MacUser<'_> {
    fn send_done(&self, spi_buf: &'static mut [u8], acked: bool, result: ReturnCode) {
        self.tx_client
            .get()
            .map(move |client| client.send_done(spi_buf, acked, result));
    }

    fn receive<'b>(&self, buf: &'b [u8], header: Header<'b>, data_offset: usize, data_len: usize) {
        self.rx_client
            .get()
            .map(move |client| client.receive(buf, header, data_offset, data_len));
    }
}

impl<'a> ListNode<'a, MacUser<'a>> for MacUser<'a> {
    fn next(&'a self) -> &'a ListLink<'a, MacUser<'a>> {
        &self.next
    }
}

impl<'a> device::MacDevice<'a> for MacUser<'a> {
    fn set_transmit_client(&self, client: &'a dyn device::TxClient) {
        self.tx_client.set(Some(client));
    }

    fn set_receive_client(&self, client: &'a dyn device::RxClient) {
        self.rx_client.set(Some(client));
    }

    fn get_address(&self) -> u16 {
        self.mux.mac.get_address()
    }

    fn get_address_long(&self) -> [u8; 8] {
        self.mux.mac.get_address_long()
    }

    fn get_pan(&self) -> u16 {
        self.mux.mac.get_pan()
    }

    fn set_address(&self, addr: u16) {
        self.mux.mac.set_address(addr)
    }

    fn set_address_long(&self, addr: [u8; 8]) {
        self.mux.mac.set_address_long(addr)
    }

    fn set_pan(&self, id: u16) {
        self.mux.mac.set_pan(id)
    }

    fn config_commit(&self) {
        self.mux.mac.config_commit()
    }

    fn is_on(&self) -> bool {
        self.mux.mac.is_on()
    }

    fn prepare_data_frame(
        &self,
        buf: &'static mut [u8],
        dst_pan: PanID,
        dst_addr: MacAddress,
        src_pan: PanID,
        src_addr: MacAddress,
        security_needed: Option<(SecurityLevel, KeyId)>,
    ) -> Result<framer::Frame, &'static mut [u8]> {
        self.mux
            .mac
            .prepare_data_frame(buf, dst_pan, dst_addr, src_pan, src_addr, security_needed)
    }

    fn transmit(&self, frame: framer::Frame) -> (ReturnCode, Option<&'static mut [u8]>) {
        // If the muxer is idle, immediately transmit the frame, otherwise
        // attempt to queue the transmission request. However, each MAC user can
        // only have one pending transmission request, so if there already is a
        // pending transmission then we must fail to entertain this one.
        self.operation
            .take()
            .map_or((ReturnCode::FAIL, None), |op| match op {
                Op::Idle => {
                    self.operation.replace(Op::Transmit(frame));
                    self.mux
                        .do_next_op_sync(self)
                        .unwrap_or((ReturnCode::SUCCESS, None))
                }
                Op::Transmit(old_frame) => {
                    self.operation.replace(Op::Transmit(old_frame));
                    (ReturnCode::EBUSY, Some(frame.into_buf()))
                }
            })
    }
}

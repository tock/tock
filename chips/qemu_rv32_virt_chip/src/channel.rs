//! Inter-processor communication channel.

use core::cell::Cell;

use kernel::threadlocal::{ThreadLocal, ThreadLocalAccess, DynThreadId, ThreadLocalDyn, ThreadId};
use kernel::utilities::registers::interfaces::Readable;
use kernel::smp::shared_channel::SharedChannel;
use kernel::smp::portal::Portalable;
use kernel::smp::mutex::Mutex;

use kernel::collections::queue::Queue;
use kernel::collections::ring_buffer::RingBuffer;
use kernel::collections::sync_queue::SyncQueue;
use kernel::collections::atomic_ring_buffer::AtomicRingBuffer;
use kernel::utilities::cells::TakeCell;

use rv32i::csr::CSR;

use crate::portal::{NUM_PORTALS, PORTALS, QemuRv32VirtPortalKey, QemuRv32VirtPortalable};
use crate::portal_cell::QemuRv32VirtPortalCell;
use crate::{plic, clint, interrupts};

#[derive(Copy, Clone)]
pub struct QemuRv32VirtMessage {
    pub sender: DynThreadId,
    pub receiver: DynThreadId,
    pub kind: QemuRv32VirtMessageKind,
}

#[derive(Copy, Clone)]
pub enum QemuRv32VirtMessageKind {
    PortalRequest(QemuRv32VirtPortalKey),
    PortalResponseSuccess(QemuRv32VirtPortalKey, core::ptr::NonNull<()>),
    PortalResponseFailure(QemuRv32VirtPortalKey),
    Ping,
    Pong,
}

impl QemuRv32VirtMessage {
    pub fn receive(&self) -> Option<QemuRv32VirtMessageReceive> {
        (rv32i::support::current_hart_id() == self.receiver)
            .then(|| QemuRv32VirtMessageReceive {
                sender: self.sender,
                kind: self.kind,
            })
    }

    pub fn prepare(receiver: DynThreadId, kind: QemuRv32VirtMessageKind) -> QemuRv32VirtMessagePrepare {
        QemuRv32VirtMessagePrepare { receiver, kind }
    }

}

impl From<QemuRv32VirtMessageReceive> for QemuRv32VirtMessage {
    fn from(value: QemuRv32VirtMessageReceive) -> Self {
        QemuRv32VirtMessage {
            sender: value.sender,
            receiver: rv32i::support::current_hart_id(),
            kind: value.kind,
        }
    }
}

pub struct QemuRv32VirtMessageReceive {
    sender: DynThreadId,
    kind: QemuRv32VirtMessageKind,
}

pub struct QemuRv32VirtMessagePrepare {
    receiver: DynThreadId,
    kind: QemuRv32VirtMessageKind,
}

impl QemuRv32VirtMessagePrepare {
    pub fn finish(self) -> QemuRv32VirtMessage {
        QemuRv32VirtMessage {
            sender: rv32i::support::current_hart_id(),
            receiver: self.receiver,
            kind: self.kind,
        }
    }
}

#[derive(Debug)]
enum ReturnStatus {
    Success,
    FailurePortal,
    FailurePortalKey(QemuRv32VirtPortalKey),
    FailureChannel,
    FailureMessageUnsupported,
}

static NO_CHANNEL: ThreadLocal<0, Option<QemuRv32VirtChannel<'static>>> = ThreadLocal::new([]);

static mut SHARED_CHANNEL: &'static dyn ThreadLocalDyn<Option<QemuRv32VirtChannel<'static>>> = &NO_CHANNEL;

pub unsafe fn set_shared_channel(
    shared_channel: &'static dyn ThreadLocalDyn<Option<QemuRv32VirtChannel<'static>>>
) {
    *core::ptr::addr_of_mut!(SHARED_CHANNEL) = shared_channel;
}

unsafe fn with_shared_channel<R, F>(f: F) -> Option<R>
where
    F: FnOnce(&mut Option<QemuRv32VirtChannel<'static>>) -> R
{
    let threadlocal: &'static dyn ThreadLocalDyn<_> = *core::ptr::addr_of_mut!(SHARED_CHANNEL);
    threadlocal
        .get_mut().map(|c| c.enter_nonreentrant(f))
}

pub unsafe fn with_shared_channel_panic<R, F>(f: F) -> R
where
    F: FnOnce(&mut Option<QemuRv32VirtChannel<'static>>) -> R
{
    with_shared_channel(f).expect("Current thread does not have access to its shared channel")
}

pub struct QemuRv32VirtChannel<'a> {
    channel: &'a AtomicRingBuffer<'a, QemuRv32VirtMessage>,
    unsents: TakeCell<'a, RingBuffer<'a, QemuRv32VirtMessage>>,
    notified: core::cell::Cell<usize>,
}

impl<'a> QemuRv32VirtChannel<'a> {
    pub fn new(
        channel: &'a AtomicRingBuffer<'a, QemuRv32VirtMessage>,
        unsents: &'a mut RingBuffer<'a, QemuRv32VirtMessage>,
    ) -> Self {
        QemuRv32VirtChannel {
            channel,
            unsents: TakeCell::new(unsents),
            notified: Cell::new(0),
        }
    }

    fn process_message(&self, msg: QemuRv32VirtMessageReceive) -> ReturnStatus {
        use QemuRv32VirtMessageKind as M;
        use QemuRv32VirtPortalKey as P;

        match msg.kind {
            M::PortalRequest(portal_key) => {
                let closure = |ps: &mut [Option<&QemuRv32VirtPortalable>; NUM_PORTALS]| {
                    match ps[portal_key as usize] {
                        Some(portal) => {
                            portal.teleport(&msg.sender)
                                .then(|| ReturnStatus::Success)
                                .unwrap_or_else(|| ReturnStatus::FailurePortal)
                        }
                        _ => ReturnStatus::FailurePortalKey(portal_key),
                    }
                };

                unsafe {
                    (&*core::ptr::addr_of!(PORTALS))
                        .get_mut()
                        .expect("This thread doesn't not have access to its local portals")
                        .enter_nonreentrant(closure)
                }
            }
            M::PortalResponseSuccess(portal_key, traveler) => {
                let closure = |ps: &mut [Option<&QemuRv32VirtPortalable>; NUM_PORTALS]| {
                    match ps[portal_key as usize] {
                        Some(portal) => {
                            portal.link(Some(traveler))
                                .map(|_| ReturnStatus::Success)
                                .unwrap_or_else(|| ReturnStatus::FailurePortal)
                        },
                        _ => ReturnStatus::FailurePortalKey(portal_key),
                    }
                };

                unsafe {
                    (&*core::ptr::addr_of!(PORTALS))
                        .get_mut()
                        .expect("This thread doesn't not have access to its local portals")
                        .enter_nonreentrant(closure)
                }
            }
            M::PortalResponseFailure(portal_key) => {
                let closure = |ps: &mut [Option<&QemuRv32VirtPortalable>; NUM_PORTALS]| {
                    match ps[portal_key as usize] {
                        Some(portal) => {
                            portal.link(None)
                                .map(|_| ReturnStatus::Success)
                                .unwrap_or_else(|| ReturnStatus::FailurePortal)
                        },
                        _ => ReturnStatus::FailurePortalKey(portal_key),
                    }
                };

                unsafe {
                    (&*core::ptr::addr_of!(PORTALS))
                        .get_mut()
                        .expect("This thread doesn't not have access to its local portals")
                        .enter_nonreentrant(closure)
                }
            }
            _ => ReturnStatus::FailureMessageUnsupported,
        }
    }

    pub fn service(&self) {
        for _ in 0..(self.channel.len()) {
            if let Some(msg) = self.channel.dequeue() {
                let return_status = msg.receive().map_or_else(
                    || {
                        self.write(msg)
                            .then(|| ReturnStatus::Success)
                            .unwrap_or_else(|| ReturnStatus::FailureChannel)
                    },
                    |m| self.process_message(m),
                );

                match return_status {
                    ReturnStatus::Success => (),
                    _ => panic!("Failed to process inter-kernel message {:?}", return_status),
                }
            } else {
                // No message in the shared queue, end processing
                break
            }
        }
    }

    pub fn service_async(&self) {
        let old_val = self.notified.get();
        self.notified.set(old_val + 1);
    }

    pub fn has_pending_requests(&self) -> bool {
        self.notified.get() != 0
    }

    pub fn service_complete(&self) {
        let old_val = self.notified.get();
        self.notified.set(old_val.saturating_sub(1));
    }

    pub fn has_unsents(&self) -> bool {
        self.unsents.map(|u| u.has_elements()).unwrap()
    }

    pub fn flush_unsents(&self) {
        let _ = self.unsents.map(|unsents| {
            for _ in 0..(unsents.len()) {
                if let Some(msg) = unsents.dequeue() {
                    if self.channel.enqueue(msg) {
                        unsafe {
                            crate::clint::with_clic_panic(|c| c.set_soft_interrupt(msg.receiver.get_id()));
                        }
                    } else {
                        unsents.enqueue(msg);
                        break
                    }
                } else {
                    break
                }
            }
        });
    }
}


impl SharedChannel for QemuRv32VirtChannel<'_> {
    type Message = QemuRv32VirtMessage;

    fn write(&self, message: Self::Message) -> bool {
        let success = self.channel.enqueue(message);
        if !success {
            self.unsents.map(|u| u.enqueue(message)).unwrap_or(false)
        } else {
            success
        }
    }

    fn read(&self) -> Option<Self::Message> {
        self.channel
            .dequeue()
    }
}

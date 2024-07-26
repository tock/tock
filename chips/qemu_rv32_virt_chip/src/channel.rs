//! Inter-processor communication channel.

use core::cell::Cell;

use kernel::deferred_call::{DeferredCall, DeferredCallClient};
use kernel::threadlocal::{ThreadLocal, ThreadLocalAccess, DynThreadId, ThreadLocalDyn};
use kernel::utilities::registers::interfaces::Readable;
use kernel::utilities::cells::OptionalCell;
use kernel::smp::shared_channel::SharedChannel;
use kernel::smp::portal::Portalable;
use kernel::smp::mutex::Mutex;
use kernel::collections::queue::Queue;
use kernel::collections::ring_buffer::RingBuffer;

use rv32i::csr::CSR;

use crate::MAX_THREADS;
use crate::portal::{NUM_PORTALS, PORTALS, QemuRv32VirtPortal};
use crate::portal_cell::QemuRv32VirtPortalCell;

// type Buffer = [u8; BUFFER_SIZE];

// pub const BUFFER_SIZE: usize = 100;
// pub static mut CHANNEL_BUFFER: ThreadLocal<MAX_THREADS, Buffer> = ThreadLocal::init([0; BUFFER_SIZE]);

#[derive(Copy, Clone)]
pub struct QemuRv32VirtMessage {
    pub src: usize,
    pub dst: usize,
    pub body: QemuRv32VirtMessageBody,
}

// use crate::portal_cell::QemuRv32VirtPortalId;

#[derive(Copy, Clone)]
pub enum QemuRv32VirtMessageBody {
    PortalRequest(usize),
    PortalResponse(usize, *const ()),
    Ping,
    Pong,
}

// pub static mut CHANNEL_BUFFER: [Option<QemuRv32VirtMessage>; 128] = [None; 128];

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
    channel: &'a Mutex<RingBuffer<'a, Option<QemuRv32VirtMessage>>>,
    deferred_call: DeferredCall,
}

impl<'a> QemuRv32VirtChannel<'a> {
    pub fn new(
        channel: &'a Mutex<RingBuffer<'a, Option<QemuRv32VirtMessage>>>
    ) -> Self {
        QemuRv32VirtChannel {
            channel,
            deferred_call: DeferredCall::new()
        }
    }

    fn find<P>(
        channel: &mut RingBuffer<'a, Option<QemuRv32VirtMessage>>,
        predicate: P
    ) -> Option<QemuRv32VirtMessage>
    where
        P: Fn(&QemuRv32VirtMessage) -> bool
    {
        let mut len = channel.len();
        while len != 0 {
            let msg = channel.dequeue()
                .expect("Invalid QemuRv32VirtChannel State")
                .expect("Invalid Message Type");
            if predicate(&msg) {
                return Some(msg)
            }
            channel.enqueue(Some(msg));
            len -= 1;
        }
        None
    }

    pub fn service(&self) {
        let hart_id = CSR.mhartid.extract().get();

                            // (hart_id == 1).then(|| {
                            //     panic!(" app thread: handle channel service")
                            // });

        // Acquire the mutex for the entire operation to reserve a slot for portal
        // response. Calling teleport inside the scope will result in a deadlock.
        // TODO: switch to a non-blocking channel
        let mut channel = self.channel.lock();
        if let Some(msg) = Self::find(&mut channel, |msg| msg.dst == hart_id) {
            use QemuRv32VirtMessageBody as M;
            match msg.body {
                M::PortalRequest(portal_id) => {
                    let closure = |ps: &mut [QemuRv32VirtPortal; NUM_PORTALS]| {
                        use QemuRv32VirtPortal as P;
                        let traveler = match ps[portal_id] {
                            P::Uart16550(val) => {
                                let portal = unsafe {
                                    &*(val as *const QemuRv32VirtPortalCell<crate::uart::Uart16550>)
                                };
                                assert!(portal.get_id() == portal_id);
                                portal.take().map(|val| val as *mut _ as *const _)
                            }
                            P::Counter(val) => {
                                let portal = unsafe {
                                    &*(val as *const QemuRv32VirtPortalCell<usize>)
                                };
                                assert!(portal.get_id() == portal_id);
                                portal.take().map(|val| val as *mut _ as *const _)
                            }
                            _ => panic!("Invalid Portal"),
                        };

                        if let Some(val) = traveler {
                            assert!(channel.enqueue(Some(QemuRv32VirtMessage {
                                src: hart_id,
                                dst: msg.src,
                                body: QemuRv32VirtMessageBody::PortalResponse(
                                    portal_id,
                                    val
                                ),
                            })));

                            unsafe {
                                kernel::thread_local_static_access!(crate::clint::CLIC, DynThreadId::new(hart_id))
                                    .expect("This thread does not have access to CLIC")
                                    .enter_nonreentrant(|clic| {
                                        clic.set_soft_interrupt(msg.src);
                                    })
                            };
                        }
                    };

                    unsafe {
                        (&*core::ptr::addr_of!(PORTALS))
                            .get_mut()
                            .expect("This thread doesn't not have access to its local portals")
                            .enter_nonreentrant(closure);
                    }
                }
                M::PortalResponse(portal_id, traveler) => {
                    let closure = |ps: &mut [QemuRv32VirtPortal; NUM_PORTALS]| {
                        use QemuRv32VirtPortal as P;
                        match ps[portal_id] {
                            P::Uart16550(val) => {
                                let portal = unsafe {
                                    &*(val as *const QemuRv32VirtPortalCell<crate::uart::Uart16550>)
                                };
                                assert!(portal.get_id() == portal_id);
                                portal.link(unsafe { &mut *(traveler as *mut _) })
                                    .expect("Failed to link the uart portal");
                            }
                            P::Counter(val) => {
                                let portal = unsafe {
                                    &*(val as *const QemuRv32VirtPortalCell<usize>)
                                };
                                assert!(portal.get_id() == portal_id);
                                portal.link(unsafe { &mut *(traveler as *mut _) })
                                    .expect("Failed to link the counter portal");
                            }
                            _ => panic!("Invalid Portal"),
                        };
                    };

                    unsafe {
                        (&*core::ptr::addr_of!(PORTALS))
                            .get_mut()
                            .expect("This thread doesn't not have access to its local portals")
                            .enter_nonreentrant(closure);
                    }
                }
                _ => panic!("Unsupported message"),
            }
        }
    }

    pub fn service_async(&self) {
        self.deferred_call.set();
    }
}


impl DeferredCallClient for QemuRv32VirtChannel<'_> {
    fn handle_deferred_call(&self) {
        self.service();
    }

    fn register(&'static self) {
        self.deferred_call.register(self);
    }
}



impl SharedChannel for QemuRv32VirtChannel<'_> {
    type Message = QemuRv32VirtMessage;

    fn write(&self, message: Self::Message) -> bool {
        self.channel
            .lock()
            .enqueue(Some(message))
    }

    fn read(&self) -> Option<Self::Message> {
        self.channel
            .lock()
            .dequeue()
            .map(|val| val.expect("Invalid message"))
    }
}

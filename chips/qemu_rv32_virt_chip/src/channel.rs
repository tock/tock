//! Inter-processor communication channel.

use core::cell::Cell;

use kernel::deferred_call::{DeferredCallThread, DeferredCallClient};
use kernel::threadlocal::{ThreadLocal, ThreadLocalAccess, DynThreadId, ThreadLocalDyn};
use kernel::utilities::registers::interfaces::Readable;
use kernel::smp::shared_channel::SharedChannel;
use kernel::smp::mutex::Mutex;
use kernel::thread_local_static_access;
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

pub static mut CHANNEL_BUFFER: [Option<QemuRv32VirtMessage>; 128] = [None; 128];

pub struct QemuRv32VirtChannel<'a>(Mutex<RingBuffer<'a, Option<QemuRv32VirtMessage>>>);

impl<'a> QemuRv32VirtChannel<'a> {
    pub fn new(buffer: &'a mut [Option<QemuRv32VirtMessage>]) -> Self {
        QemuRv32VirtChannel(Mutex::new(
            RingBuffer::new(buffer)
        ))
    }

    fn find<P>(channel: &mut RingBuffer<'a, Option<QemuRv32VirtMessage>>, predicate: P) -> Option<QemuRv32VirtMessage>
    where
        P: Fn(&QemuRv32VirtMessage) -> bool
    {
        let mut len = channel.len();
        while len != 0 {
            let msg = channel.dequeue().expect("Invalid QemuRv32VirtChannel State").expect("Invalid Message Type");
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
        let clic = unsafe {
            thread_local_static_access!(crate::clint::CLIC, DynThreadId::new(hart_id))
            .expect("This thread does not have access to CLIC")
        };

        let mut channel = self.0.lock();
        if let Some(msg) = Self::find(&mut channel, |msg| msg.dst == hart_id) {
            use QemuRv32VirtMessageBody as M;
            match msg.body {
                // M::Ping => {
                //     let mut channel = self.0.lock();
                //     channel.push(Some(QemuRv32VirtMessage {
                //         src: hart_id,
                //         dst: msg.src,
                //         body: QemuRv32VirtMessageBody::Pong,
                //     }));
                //     clic.set_soft_interrupt(msg.src);
                // }
                // M::Pong => {
                //     let mut channel = self.0.lock();
                //     channel.push(Some(QemuRv32VirtMessage {
                //         src: hart_id,
                //         dst: msg.src,
                //         body: QemuRv32VirtMessageBody::Ping,
                //     }));
                //     clic.set_soft_interrupt(msg.src);
                // }
                M::PortalRequest(portal_id) => {
                    let closure = |ps: &mut [QemuRv32VirtPortal; NUM_PORTALS]| {
                        use QemuRv32VirtPortal as P;
                        let traveler = match ps[portal_id] {
                            P::Uart16550(val) => todo!(),
                            P::Counter(val) => {
                                let portal = unsafe {
                                    &*(val as *const QemuRv32VirtPortalCell<core::sync::atomic::AtomicUsize>)
                                };
                                assert!(portal.get_id() == portal_id);
                                portal.take()
                            }
                            _ => panic!("Invalid Portal"),
                        };

                        if let Some(val) = traveler {
                            assert!(channel.enqueue(Some(QemuRv32VirtMessage {
                                src: hart_id,
                                dst: msg.src,
                                body: QemuRv32VirtMessageBody::PortalResponse(
                                    portal_id,
                                    val as *mut _ as *const _,
                                ),
                            })));
                            clic.set_soft_interrupt(msg.src);
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
                            P::Uart16550(val) => todo!(),
                            P::Counter(val) => {
                                let portal = unsafe {
                                    &*(val as *const QemuRv32VirtPortalCell<core::sync::atomic::AtomicUsize>)
                                };
                                assert!(portal.get_id() == portal_id);
                                // assert!(msg.src == 0);
                                // assert!(msg.dst == 1);
                                unsafe {
                                    portal.replace_none(&mut *(traveler as *mut _))
                                        .expect("Double portal")
                                }
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
                _ => panic!("Invalid Message :("),
            }
        }
    }
}


impl DeferredCallClient for QemuRv32VirtChannel<'_> {
    fn handle_deferred_call(&self) {
        self.service()
    }

    fn register(&'static self) {
        DeferredCallThread::register(self)
    }
}



impl SharedChannel for QemuRv32VirtChannel<'_> {
    type Message = QemuRv32VirtMessage;

    fn write(&self, message: Self::Message) -> bool {
        self.0
            .lock()
            .enqueue(Some(message))
    }

    fn read(&self) -> Option<Self::Message> {
        self.0
            .lock()
            .dequeue()
            .map(|val| val.expect("Invalid Message"))
    }
}

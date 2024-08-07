//! Inter-processor communication channel.

use core::cell::Cell;

use kernel::threadlocal::{ThreadLocal, ThreadLocalAccess, DynThreadId, ThreadLocalDyn};
use kernel::utilities::registers::interfaces::Readable;
use kernel::smp::shared_channel::SharedChannel;
use kernel::smp::portal::Portalable;
use kernel::smp::mutex::Mutex;
use kernel::collections::queue::Queue;
use kernel::collections::ring_buffer::RingBuffer;

use rv32i::csr::CSR;

use crate::portal::{NUM_PORTALS, PORTALS, QemuRv32VirtPortal};
use crate::portal_cell::QemuRv32VirtPortalCell;
use crate::{plic, clint, interrupts};

#[derive(Copy, Clone)]
pub struct QemuRv32VirtMessage {
    pub src: usize,
    pub dst: usize,
    pub body: QemuRv32VirtMessageBody,
}

#[derive(Copy, Clone)]
pub enum QemuRv32VirtMessageBody {
    PortalRequest(usize),
    PortalResponse(usize, *const ()),
    Ping,
    Pong,
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
    channel: &'a Mutex<RingBuffer<'a, Option<QemuRv32VirtMessage>>>,
    notified: core::cell::Cell<usize>,
}

impl<'a> QemuRv32VirtChannel<'a> {
    pub fn new(
        channel: &'a Mutex<RingBuffer<'a, Option<QemuRv32VirtMessage>>>
    ) -> Self {
        QemuRv32VirtChannel {
            channel,
            notified: Cell::new(0),
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

        // Acquire the mutex for the entire operation to reserve a slot for a potential
        // response. Invoking teleport inside the scope will result in a deadlock.
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
                                if let Some(uart) = portal.take() {
                                    uart.save_context();
                                    unsafe {
                                        plic::with_plic_panic(|plic| {
                                            plic.disable(hart_id * 2,
                                                         (interrupts::UART0 as u32).try_into().unwrap());
                                        });
                                    }
                                    Some(uart as *mut _ as *const _)
                                } else {
                                    None
                                }
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

                            let closure = move |c: &mut crate::chip::QemuRv32VirtClint| c.set_soft_interrupt(msg.src);
                            unsafe {
                                crate::clint::with_clic_panic(closure);
                            }
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

                                portal.enter(|uart| {
                                    uart.restore_context();
                                    // Enable uart interrupts
                                    unsafe {
                                        plic::with_plic_panic(|plic| {
                                            plic.enable(hart_id * 2, (interrupts::UART0 as u32).try_into().unwrap());
                                        });
                                    }
                                    // Try continue from the last transmit in case of missing interrupts
                                    let _ = uart.try_transmit_continue();
                                });
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
        let old_val = self.notified.get();
        self.notified.set(old_val + 1);
    }

    pub fn has_pending_requests(&self) -> bool {
        self.notified.get() != 0
    }

    pub fn service_complete(&self) {
        let old_val = self.notified.get();
        self.notified.set(old_val - 1);
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

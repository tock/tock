use kernel::hil::portal::{Portal, PortalClient};
// use capsules_core::portals::teleportable_uart::{UartPortal, UartPortalClient, UartTraveler};

use kernel::collections::sync_queue::SyncQueue;
use kernel::collections::atomic_ring_buffer::AtomicRingBuffer;
use kernel::threadlocal::{DynThreadId, ThreadId, ThreadLocal, ThreadLocalDyn};
use kernel::errorcode::ErrorCode;
use kernel::utilities::cells::{TakeCell, OptionalCell};

use core::ptr::NonNull;

use crate::clint;

use capsules_core::portals::mux_demux::MuxTraveler;

// enum QemuRv32VirtTraveler<'a> {
//     Uart(TakeCell<'a, UartTraveler>),
// }

pub enum QemuRv32VirtVoyager {
    Teleport(TakeCell<'static, MuxTraveler>),
    Teleported(TakeCell<'static, MuxTraveler>, Result<(), ErrorCode>),
    Empty,
}

pub type QemuRv32VirtVoyagerReference = NonNull<QemuRv32VirtVoyager>;

pub struct QemuRv32VirtPortal<'a> {
    shared_channel: &'a AtomicRingBuffer<'a, NonNull<QemuRv32VirtVoyager>>,
    portal_dest: DynThreadId,
    voyager: TakeCell<'static, QemuRv32VirtVoyager>,
    received: core::cell::Cell<bool>,
    portal_client: OptionalCell<&'a dyn PortalClient<MuxTraveler>>,
    portal: OptionalCell<&'a dyn Portal<'a, MuxTraveler>>,
}

impl<'a> QemuRv32VirtPortal<'a> {
    pub fn new(
        shared_channel: &'a AtomicRingBuffer<'a, NonNull<QemuRv32VirtVoyager>>,
        portal_dest: DynThreadId,
        voyager: &'static mut QemuRv32VirtVoyager,
    ) -> QemuRv32VirtPortal<'a> {
        QemuRv32VirtPortal {
            shared_channel,
            portal_dest,
            voyager: TakeCell::new(voyager),
            received: core::cell::Cell::new(false),
            portal_client: OptionalCell::empty(),
            portal: OptionalCell::empty(),
        }
    }

    pub fn empty(
        shared_channel: &'a AtomicRingBuffer<'a, NonNull<QemuRv32VirtVoyager>>,
        portal_dest: DynThreadId,
    ) -> QemuRv32VirtPortal<'a> {
        QemuRv32VirtPortal {
            shared_channel,
            portal_dest,
            voyager: TakeCell::empty(),
            received: core::cell::Cell::new(false),
            portal_client: OptionalCell::empty(),
            portal: OptionalCell::empty(),
        }
    }

    pub fn set_downstream_portal(&self, portal: &'a dyn Portal<'a, MuxTraveler>) {
        self.portal.replace(portal);
    }

    pub fn receive_voyager_async(&self) {
        self.received.set(true);
    }

    fn do_receive_voyager(&self, voyager: &'static mut QemuRv32VirtVoyager) {
        match voyager {
            QemuRv32VirtVoyager::Teleport(traveler) => {
                traveler.take().map(|itraveler| {
                    self.portal.map(|portal| portal.teleport(itraveler))
                });
                self.voyager.replace(voyager);
            }
            QemuRv32VirtVoyager::Teleported(traveler, rcode) => {
                traveler.take().map(|itraveler| {
                    self.portal_client.map(|client| client.teleported(itraveler, *rcode))
                });
                self.voyager.replace(voyager);
            }
            _ => ()
        }
    }

    pub fn receive_voyager(&self) {
        for _ in 0..self.shared_channel.len() {
            match self.shared_channel.dequeue() {
                Some(mut traveler) => {
                    self.do_receive_voyager(unsafe { traveler.as_mut() });
                }
                _ => break,
            }
        }
    }

    pub fn received(&self) {
        self.received.set(false);
    }

    pub fn has_received(&self) -> bool {
        self.received.get()
    }
}

impl<'a> Portal<'a, MuxTraveler> for QemuRv32VirtPortal<'a> {
    fn set_portal_client(&self, client: &'a dyn PortalClient<MuxTraveler>) {
        self.portal_client.set(client);
    }

    fn teleport(
        &self,
        traveler: &'static mut MuxTraveler,
    ) -> Result<(), (ErrorCode, &'static mut MuxTraveler)> {
        match self.voyager.take() {
            Some(voyager) => {
                *voyager = QemuRv32VirtVoyager::Teleport(TakeCell::new(traveler));
                self.shared_channel
                    .enqueue(NonNull::new(voyager).unwrap())
                    .then(|| { unsafe {
                        clint::with_clic_panic(|c| c.set_soft_interrupt(self.portal_dest.get_id()));
                    }; })
                    .ok_or_else(move || { match voyager {
                        QemuRv32VirtVoyager::Teleport(traveler) => {
                            let rtraveler = traveler.take().unwrap();
                            self.voyager.replace(voyager);
                            (ErrorCode::FAIL, rtraveler)
                        }
                        _ => unreachable!()
                    }})
            }
            None => Err((ErrorCode::FAIL, traveler))
        }
    }
}

impl<'a> PortalClient<MuxTraveler> for QemuRv32VirtPortal<'a> {
    fn teleported(
        &self,
        traveler: &'static mut MuxTraveler,
        rcode: Result<(), ErrorCode>,
    ) {
        match self.voyager.take() {
            Some(voyager) => {
                *voyager = QemuRv32VirtVoyager::Teleported(TakeCell::new(traveler), rcode);
                self.shared_channel.enqueue(NonNull::new(voyager).unwrap()).then(|| {
                    unsafe {
                        clint::with_clic_panic(|c| c.set_soft_interrupt(self.portal_dest.get_id()));
                    }
                });
            }
            None => (),
        }
    }
}


static NO_PORTAL: ThreadLocal<0, OptionalCell<&'static QemuRv32VirtPortal<'static>>> = ThreadLocal::new([]);

static mut PORTAL: &'static dyn ThreadLocalDyn<OptionalCell<&'static QemuRv32VirtPortal<'static>>> = &NO_PORTAL;

pub unsafe fn set_portal(
    portal: &'static dyn ThreadLocalDyn<OptionalCell<&'static QemuRv32VirtPortal<'static>>>
) {
    *core::ptr::addr_of_mut!(PORTAL) = portal;
}

pub unsafe fn init_portal_panic(
    portal: &'static QemuRv32VirtPortal<'static>,
) {
    let threadlocal: &'static dyn ThreadLocalDyn<_> = *core::ptr::addr_of_mut!(PORTAL);
    threadlocal
        .get_mut()
        .and_then(|p| p.enter_nonreentrant(|v| v.replace(portal).is_none().then(|| ())))
        .unwrap_or_else(|| {
            panic!("Core {} is unable to initialize its local portal",
                   rv32i::support::current_hart_id().get_id())
        })
}

unsafe fn with_portal<R, F>(f: F) -> Option<R>
where
    F: FnOnce(&QemuRv32VirtPortal<'static>) -> R
{
    let threadlocal: &'static dyn ThreadLocalDyn<_> = *core::ptr::addr_of_mut!(PORTAL);
    threadlocal
        .get_mut().and_then(|p| p.enter_nonreentrant(|v| v.map(f)))
}

pub unsafe fn with_portal_panic<R, F>(f: F) -> R
where
    F: FnOnce(&QemuRv32VirtPortal<'static>) -> R
{
    with_portal(f)
        .unwrap_or_else(|| {
            panic!("Core {} does not have access to an initialized portal",
                   rv32i::support::current_hart_id().get_id())
        })
}

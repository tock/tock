use kernel::hil::portal::{Portal, PortalClient};
use capsules_core::portals::teleportable_uart::{UartPortal, UartPortalClient, UartTraveler};

use kernel::collections::sync_queue::SyncQueue;
use kernel::collections::atomic_ring_buffer::AtomicRingBuffer;
use kernel::threadlocal::{DynThreadId, ThreadId, ThreadLocal, ThreadLocalDyn};
use kernel::errorcode::ErrorCode;
use kernel::utilities::cells::{TakeCell, OptionalCell};

use core::ptr::NonNull;

use crate::clint;

enum QemuRv32VirtTraveler<'a> {
    Uart(TakeCell<'a, UartTraveler>),
}

pub enum QemuRv32VirtVoyager<'a> {
    Teleport(QemuRv32VirtTraveler<'a>),
    Teleported(QemuRv32VirtTraveler<'a>),
    Empty,
}

pub type QemuRv32VirtVoyagerReference = NonNull<QemuRv32VirtVoyager<'static>>;

pub struct QemuRv32VirtPortal<'a> {
    shared_channel: &'a AtomicRingBuffer<'a, NonNull<QemuRv32VirtVoyager<'static>>>,
    portal_dest: DynThreadId,
    voyager: TakeCell<'static, QemuRv32VirtVoyager<'static>>,
    received: core::cell::Cell<bool>,
    portal_client: OptionalCell<&'a dyn PortalClient<Traveler=UartTraveler>>,
    portal: OptionalCell<&'a dyn Portal<'a, UartTraveler>>,
}

impl<'a> QemuRv32VirtPortal<'a> {
    pub fn new(
        shared_channel: &'a AtomicRingBuffer<'a, NonNull<QemuRv32VirtVoyager<'static>>>,
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
        shared_channel: &'a AtomicRingBuffer<'a, NonNull<QemuRv32VirtVoyager<'static>>>,
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

    pub fn set_downstream_portal(&self, portal: &'a dyn Portal<'a, UartTraveler>) {
        self.portal.replace(portal);
    }

    pub fn receive_voyager_async(&self) {
        self.received.set(true);
    }

    fn do_receive_voyager(&self, voyager: &'static mut QemuRv32VirtVoyager) {
        match voyager {
            QemuRv32VirtVoyager::Teleport(traveler) => {
                match traveler {
                    QemuRv32VirtTraveler::Uart(uart_traveler) => {
                        uart_traveler.take().map(|utraveler| {
                            self.portal.map(|portal| portal.teleport(utraveler))
                        });
                    }
                };
                self.voyager.replace(voyager);
            }
            QemuRv32VirtVoyager::Teleported(traveler) => {
                match traveler {
                    QemuRv32VirtTraveler::Uart(uart_traveler) => {
                        uart_traveler.take().map(|utraveler| {
                            self.portal_client.map(|client| client.teleported(utraveler))
                        });
                    }
                };
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

impl<'a> Portal<'a, UartTraveler> for QemuRv32VirtPortal<'a> {
    fn set_portal_client(&self, client: &'a dyn PortalClient<Traveler=UartTraveler>) {
        self.portal_client.set(client);
    }

    fn teleport(
        &self,
        traveler: &'static mut UartTraveler,
    ) -> Result<(), (ErrorCode, &'static mut UartTraveler)> {
        match self.voyager.take() {
            Some(voyager) => {
                *voyager = QemuRv32VirtVoyager::Teleport(
                    QemuRv32VirtTraveler::Uart(TakeCell::new(traveler))
                );
                self.shared_channel
                    .enqueue(NonNull::new(voyager).unwrap())
                    .then(|| { unsafe {
                        clint::with_clic_panic(|c| c.set_soft_interrupt(self.portal_dest.get_id()));
                    }; })
                    .ok_or_else(move || { match voyager {
                        QemuRv32VirtVoyager::Teleport(QemuRv32VirtTraveler::Uart(traveler)) => {
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

impl<'a> PortalClient for QemuRv32VirtPortal<'a> {
    type Traveler = UartTraveler;

    fn teleported(
        &self,
        traveler: &'static mut Self::Traveler,
    ) -> Result<(), (ErrorCode, &'static mut Self::Traveler)> {
        match self.voyager.take() {
            Some(voyager) => {
                *voyager = QemuRv32VirtVoyager::Teleported(
                    QemuRv32VirtTraveler::Uart(TakeCell::new(traveler))
                );
                self.shared_channel.enqueue(NonNull::new(voyager).unwrap()).then(|| {
                    unsafe {
                        clint::with_clic_panic(|c| c.set_soft_interrupt(self.portal_dest.get_id()));
                    }
                });
                Ok(())
            }
            None => Err((ErrorCode::FAIL, traveler))
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

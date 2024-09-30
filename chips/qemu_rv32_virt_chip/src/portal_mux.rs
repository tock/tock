use kernel::hil::portal::{Portal, PortalClient};
use capsules_core::portals::teleportable_uart::{UartPortal, UartPortalClient, UartTraveler};

use kernel::collections::list::{List, ListLink, ListNode};
use kernel::utilities::cells::{TakeCell, OptionalCell};
use kernel::errorcode::ErrorCode;
use kernel::deferred_call::{DeferredCall, DeferredCallClient};

use core::cell::Cell;

pub enum MuxTraveler {
    Uart(usize, TakeCell<'static, UartTraveler>),
}

pub struct MuxPortal<'a> {
    portal: &'a dyn Portal<'a, MuxTraveler>,
    portal_clients: List<'a, MuxPortalClient<'a>>,
    inflight: OptionalCell<&'a MuxPortalClient<'a>>,
    deferred_call: DeferredCall,
}

impl<'a> MuxPortal<'a> {
    pub fn new(portal: &'a dyn Portal<'a, MuxTraveler>) -> MuxPortal<'a> {
        MuxPortal {
            portal,
            portal_clients: List::new(),
            inflight: OptionalCell::empty(),
            deferred_call: DeferredCall::new(),
        }
    }

    fn do_next_op(&self) {
        if self.inflight.is_none() {
            let pclient = self.portal_clients.iter().find(|c| c.has_boarding_traveler());
            pclient.map(|client| {
                client.traveler.take().map(|traveler| {
                    match self.portal.teleport(traveler) {
                        Ok(()) => {
                            self.inflight.set(client);
                        }
                        Err((ecode, traveler)) => {
                            client.teleporting.set(false);
                            client.teleported(traveler, Err(ecode));
                        }
                    }
                })
            });
        }
    }

    fn do_next_op_async(&self) {
        self.deferred_call.set()
    }
}

impl<'a> PortalClient<MuxTraveler> for MuxPortal<'a> {
    fn teleported(
        &self,
        traveler: &'static mut MuxTraveler,
        rcode: Result<(), ErrorCode>,
    ) {
        self.inflight.map(move |client| {
            self.inflight.clear();
            client.teleported(traveler, rcode);
        });
        self.do_next_op();
    }
}

impl DeferredCallClient for MuxPortal<'_> {
    fn handle_deferred_call(&self) {
        self.do_next_op();
    }

    fn register(&'static self) {
        self.deferred_call.register(self);
    }
}

#[derive(Clone, Copy)]
pub enum MuxClient<'a> {
    Uart(&'a dyn PortalClient<UartTraveler>),
}

pub struct MuxPortalClient<'a> {
    mux: &'a MuxPortal<'a>,
    next: ListLink<'a, MuxPortalClient<'a>>,
    traveler: TakeCell<'static, MuxTraveler>,
    portal_client: OptionalCell<MuxClient<'a>>,
    teleporting: Cell<bool>,
    id: usize,
}

impl<'a> MuxPortalClient<'a> {
    pub fn new(mux: &'a MuxPortal<'a>, traveler: &'static mut MuxTraveler, id: usize) -> MuxPortalClient<'a> {
        MuxPortalClient {
            mux,
            id,
            next: ListLink::empty(),
            traveler: TakeCell::new(traveler),
            portal_client: OptionalCell::empty(),
            teleporting: Cell::new(false),
        }
    }

    pub fn setup(&'a self) {
        self.mux.portal_clients.push_head(self);
    }

    pub fn has_boarding_traveler(&self) -> bool {
        self.traveler.map_or(false, |traveler| {
            match traveler {
                MuxTraveler::Uart(_, utraveler) => utraveler.is_some(),
            }
        })
    }
}

impl<'a> ListNode<'a, MuxPortalClient<'a>> for MuxPortalClient<'a> {
    fn next(&'a self) -> &'a ListLink<'a, MuxPortalClient<'a>> {
        &self.next
    }
}

impl<'a> Portal<'a, UartTraveler> for MuxPortalClient<'a> {
    fn set_portal_client(&self, client: &'a dyn PortalClient<UartTraveler>) {
        self.portal_client.set(MuxClient::Uart(client))
    }

    fn teleport(
        &self,
        traveler: &'static mut UartTraveler,
    ) -> Result<(), (ErrorCode, &'static mut UartTraveler)> {
        if self.teleporting.get() {
            Err((ErrorCode::BUSY, traveler))
        } else {
            self.teleporting.set(true);
            self.traveler.map(|straveler| {
                *straveler = MuxTraveler::Uart(self.id, TakeCell::new(traveler))
            });
            self.mux.do_next_op_async();
            Ok(())
        }
    }
}

impl<'a> PortalClient<MuxTraveler> for MuxPortalClient<'a> {
    fn teleported(
        &self,
        traveler: &'static mut MuxTraveler,
        rcode: Result<(), ErrorCode>,
    ) {
        match self.portal_client.get() {
            Some(MuxClient::Uart(uart_client)) => {
                match traveler {
                    MuxTraveler::Uart(_, uart_traveler) => {
                        uart_traveler.take().map(|utraveler| uart_client.teleported(utraveler, rcode));
                        self.traveler.replace(traveler);
                        self.teleporting.set(false);
                    }
                    _ => todo!()
                }
            }
            None => (),
        }
    }
}

// --------------------------------------- DEMUX ----------------------------------------------

pub struct DemuxPortal<'a> {
    portals: List<'a, DemuxPortalDevice<'a>>,
    portal_client: OptionalCell<&'a dyn PortalClient<MuxTraveler>>,
}

impl<'a> DemuxPortal<'a> {
    pub fn new() -> DemuxPortal<'a> {
        DemuxPortal {
            portals: List::new(),
            portal_client: OptionalCell::empty(),
        }
    }
}

impl<'a> Portal<'a, MuxTraveler> for DemuxPortal<'a> {
    fn set_portal_client(&self, client: &'a dyn PortalClient<MuxTraveler>) {
        self.portal_client.set(client);
    }

    fn teleport(
        &self,
        traveler: &'static mut MuxTraveler
    ) -> Result<(), (ErrorCode, &'static mut MuxTraveler)> {

        let id = match traveler {
            MuxTraveler::Uart(id, _) => *id,
        };

        self.portals.iter().find(|p| p.id() == id)
            .map(|portal| {
                portal.teleport(traveler)
            })
            .unwrap()
    }
}

impl<'a> PortalClient<MuxTraveler> for DemuxPortal<'a> {
    fn teleported(
        &self,
        traveler: &'static mut MuxTraveler,
        rcode: Result<(), ErrorCode>,
    ) {
        self.portal_client.map(|client| client.teleported(traveler, rcode));
    }
}

#[derive(Clone, Copy)]
pub enum DemuxDevice<'a> {
    Uart(&'a dyn Portal<'a, UartTraveler>),
}


pub struct DemuxPortalDevice<'a> {
    portal: DemuxDevice<'a>,
    demux: &'a DemuxPortal<'a>,
    // portal_client: OptionalCell<&'a dyn PortalClient<MuxTraveler>>,
    traveler: TakeCell<'static, MuxTraveler>,
    next: ListLink<'a, DemuxPortalDevice<'a>>,
    id: usize,
}

impl<'a> DemuxPortalDevice<'a> {
    pub fn new(portal: DemuxDevice<'a>, demux: &'a DemuxPortal<'a>, id: usize) -> DemuxPortalDevice<'a> {
        DemuxPortalDevice {
            portal,
            id,
            demux,
            traveler: TakeCell::empty(),
            next: ListLink::empty(),
        }
    }

    fn id(&self) -> usize {
        self.id
    }

    pub fn setup(&'a self) {
        self.demux.portals.push_head(self);
    }
}

impl<'a> ListNode<'a, DemuxPortalDevice<'a>> for DemuxPortalDevice<'a> {
    fn next(&'a self) -> &'a ListLink<'a, DemuxPortalDevice<'a>> {
        &self.next
    }
}

impl<'a> Portal<'a, MuxTraveler> for DemuxPortalDevice<'a> {
    fn set_portal_client(&self, client: &'a dyn PortalClient<MuxTraveler>) {
        // self.portal_client.set(client);
        unimplemented!()
    }

    fn teleport(
        &self,
        traveler: &'static mut MuxTraveler
    ) -> Result<(), (ErrorCode, &'static mut MuxTraveler)> {
        match traveler {
            MuxTraveler::Uart(_, uart_traveler) => {
                let ret = match self.portal {
                    DemuxDevice::Uart(uart_portal) => {
                        uart_traveler.take().map(|utraveler| {
                            uart_portal.teleport(utraveler)
                        })
                    }
                };
                self.traveler.replace(traveler);
                ret.unwrap()
                    .map_err(|(ecode, ut)| {
                        let traveler = self.traveler.take().unwrap();
                        match traveler {
                            MuxTraveler::Uart(_, uart_traveler) => {
                                uart_traveler.replace(ut);
                            }
                        }
                        (ecode, traveler)
                    })
            }
        }
    }
}

impl<'a> PortalClient<UartTraveler> for DemuxPortalDevice<'a> {
    fn teleported(
        &self,
        traveler: &'static mut UartTraveler,
        rcode: Result<(), ErrorCode>,
    ) {
        self.traveler.take().map(|straveler| {
            *straveler = MuxTraveler::Uart(self.id, TakeCell::new(traveler));
            self.demux.teleported(straveler, rcode);
        });
    }
}

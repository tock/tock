use crate::common::cells::OptionalCell;
use core::cell::Cell;

pub trait DeferredCallMuxBackend {
    fn set(&self);
    fn set_client(&self, client: &'static DeferredCallMuxBackendClient);
}

pub trait DeferredCallMuxBackendClient {
    fn call(&self);
}

pub trait DeferredCallMuxClient {
    fn call(&self, handle: DeferredCallHandle);
}

#[derive(Copy, Clone, Debug)]
pub struct DeferredCallHandle (usize);
pub struct DeferredCallMux {
    backend: &'static DeferredCallMuxBackend,
    clients: &'static [(Cell<bool>, OptionalCell<&'static DeferredCallMuxClient>)],
    handle_counter: Cell<usize>,
}
impl DeferredCallMux {
    pub fn new(
        backend: &'static DeferredCallMuxBackend,
        clients: &'static [(Cell<bool>, OptionalCell<&'static DeferredCallMuxClient>)]
    ) -> DeferredCallMux {
        DeferredCallMux {
            backend: backend,
            clients: clients,
            handle_counter: Cell::new(0),
        }
    }

    pub fn set(&self, handle: DeferredCallHandle) -> Result<bool, ()> {
        let DeferredCallHandle(client_pos) = handle;
        let client = &self.clients[client_pos];

        if let (call_set, true) = (&client.0, client.1.is_some()) {
            if call_set.get() {
                // Already set
                Ok(false)
            } else {
                call_set.set(true);
                self.backend.set();
                Ok(true)
            }
        } else {
            Err(())
        }
    }

    pub fn register(&self, mux_client: &'static DeferredCallMuxClient) -> Result<DeferredCallHandle, ()> {
        let current_counter = self.handle_counter.get();

        if current_counter < self.clients.len() {
            let client = &self.clients[current_counter];
            client.0.set(false);
            client.1.set(mux_client);

            self.handle_counter.set(current_counter + 1);

            Ok(DeferredCallHandle (current_counter))
        } else {
            Err(())
        }
    }
}

impl DeferredCallMuxBackendClient for DeferredCallMux {
    fn call(&self) {
        self.clients
            .iter()
            .map(|(ref call_reqd, ref oc): &(Cell<bool>, OptionalCell<&'static DeferredCallMuxClient>)| -> (&Cell<bool>, &OptionalCell<&'static DeferredCallMuxClient>) { (call_reqd, oc) })
            .enumerate()
            .filter(|(_i, (call_reqd, _oc))| call_reqd.get())
            .filter_map(
                |(i, (call_reqd, oc)): (usize, (&Cell<bool>, &OptionalCell<&'static DeferredCallMuxClient>))| ->
                    Option<(usize, &Cell<bool>, &'static DeferredCallMuxClient)> {
                        oc.map(|c| (i, call_reqd, *c))
                    }
            )
            .for_each(|(i, call_reqd, client)| {
                call_reqd.set(false);
                client.call(DeferredCallHandle (i));
            });
    }
}

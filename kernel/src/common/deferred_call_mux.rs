use crate::common::cells::TakeCell;
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
    clients: TakeCell<'static, [
        Option<(bool, &'static DeferredCallMuxClient)>
    ]>,
    handle_counter: Cell<usize>,
}
impl DeferredCallMux {
    pub fn new(
        backend: &'static DeferredCallMuxBackend,
        clients: &'static mut [Option<(bool, &'static DeferredCallMuxClient)>]
    ) -> DeferredCallMux {
        DeferredCallMux {
            backend: backend,
            clients: TakeCell::new(clients),
            handle_counter: Cell::new(0),
        }
    }

    pub fn set(&self, handle: DeferredCallHandle) -> Result<bool, ()> {
        let DeferredCallHandle(client_pos) = handle;

        self.clients.map(|clients| {
            if let Some(ref mut client) = clients[client_pos] {
                if client.0 {
                    // Already set
                    Ok(false)
                } else {
                    client.0 = true;
                    self.backend.set();
                    Ok(true)
                }
            } else {
                Err(())
            }
        }).unwrap_or(Err(()))
    }

    pub fn register(&self, client: &'static DeferredCallMuxClient) -> Result<DeferredCallHandle, ()> {
        self.clients.map(|clients| {
            let current_counter = self.handle_counter.get();

            if current_counter < clients.len() {
                clients[current_counter] = Some((false, client));
                self.handle_counter.set(current_counter + 1);

                Ok(DeferredCallHandle (current_counter))
            } else {
                Err(())
            }
        }).unwrap_or(Err(()))
    }
}

impl DeferredCallMuxBackendClient for DeferredCallMux {
    fn call(&self) {
        self.clients.map(|clients| {
            clients
                .iter_mut()
                .enumerate()
                .filter_map(|(i, opt_c)| opt_c.map(|o| (i, o)))
                .filter(|(_i, (call_reqd, _))| *call_reqd)
                .for_each(|(i, mut client)| {
                    client.0 = false;
                    client.1.call(DeferredCallHandle (i));
                });
        });
    }
}

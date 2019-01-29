//! Implementation of hardware-abstraction traits required for a
//! [DeferredCallMux]
//!
//! See the documentation of the [DeferredCallMux] for more information.
//!
//! [DeferredCallMux]: kernel::common::deferred_call_mux::DeferredCallMux

use kernel::common::cells::OptionalCell;

use crate::deferred_call_tasks::DeferredCallTask;
use kernel::common::deferred_call::DeferredCall;
use kernel::common::deferred_call_mux::{DeferredCallMuxBackend, DeferredCallMuxBackendClient};

static DEFERRED_CALL: DeferredCall<DeferredCallTask> =
    unsafe { DeferredCall::new(DeferredCallTask::MuxBackend) };

pub static mut MUXBACKEND: Nrf52DeferredCallMuxBackend = Nrf52DeferredCallMuxBackend::new();

pub struct Nrf52DeferredCallMuxBackend {
    client: OptionalCell<&'static DeferredCallMuxBackendClient>,
}

impl Nrf52DeferredCallMuxBackend {
    pub const fn new() -> Nrf52DeferredCallMuxBackend {
        Nrf52DeferredCallMuxBackend {
            client: OptionalCell::empty(),
        }
    }

    pub fn handle_interrupt(&self) {
        self.client.map(|c| c.call());
    }
}

impl DeferredCallMuxBackend for Nrf52DeferredCallMuxBackend {
    fn set(&self) {
        DEFERRED_CALL.set();
    }

    fn set_client(&self, client: &'static DeferredCallMuxBackendClient) {
        self.client.set(client);
    }
}

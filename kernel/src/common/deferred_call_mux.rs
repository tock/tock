//! Hardware-independent kernel interface for deferred calls
//!
//! This allows any struct in the kernel which implements
//! [DeferredCallMuxClient](crate::common::deferred_call_mux::DeferredCallMuxClient)
//! to set and receive deferred calls.
//!
//! These are scheduled "interrupts" in the chip scheduler.
//! They are especially important if some hardware doesn't
//! support real interrupts where they are needed, or for
//! implementing software devices that are supposed to work
//! like hardware devices.
//!
//! Usage
//! -----
//!
//! This example assumes some deferred call mux backend implementation
//! `MUXBACKEND`. For a backend implementation, see
//! `chips/nrf52/src/deferred_call_mux.rs`.
//!
//! The `deferred_call_mux_clients` array size determines how many
//! [DeferredCallHandle](crate::common::deferred_call_mux::DeferredCallHandle)s
//! may be registered with the Mux.
//! When no more slots are available,
//! `deferred_call_mux.register(some_client)` will return `None`.
//!
//! ```
//! # use core::cell::Cell;
//! # use kernel::common::cells::OptionalCell;
//! # use kernel::static_init;
//! use kernel::common::deferred_call_mux::{
//!     DeferredCallMux,
//!     DeferredCallMuxClient,
//!     DeferredCallMuxBackend
//! };
//! #
//! # struct ExampleDeferredCallMuxBackend;
//! # impl DeferredCallMuxBackend for ExampleDeferredCallMuxBackend {
//! #     fn set(&self) { }
//! #     fn set_client(
//! #         &self,
//! #         _c: &'static kernel::common::deferred_call_mux::DeferredCallMuxBackendClient,
//! #     ) { }
//! # }
//! # static mut MUXBACKEND: ExampleDeferredCallMuxBackend =
//! #   ExampleDeferredCallMuxBackend;
//!
//! let deferred_call_mux_clients = unsafe { static_init!(
//!     [(Cell<bool>, OptionalCell<&'static DeferredCallMuxClient>); 1],
//!     [(Cell::new(false), OptionalCell::empty())]
//! ) };
//! let deferred_call_mux = unsafe { static_init!(
//!     DeferredCallMux,
//!     DeferredCallMux::new(&MUXBACKEND, deferred_call_mux_clients)
//! ) };
//! unsafe { MUXBACKEND.set_client(deferred_call_mux) };
//!
//! # struct SomeCapsule;
//! # impl SomeCapsule {
//! #     pub fn new(_mux: &'static DeferredCallMux) -> Self { SomeCapsule }
//! #     pub fn set_deferred_call_handle(
//! #         &self,
//! #         _handle: kernel::common::deferred_call_mux::DeferredCallHandle,
//! #     ) { }
//! # }
//! # impl DeferredCallMuxClient for SomeCapsule {
//! #     fn call(
//! #         &self,
//! #         _handle: kernel::common::deferred_call_mux::DeferredCallHandle,
//! #     ) { }
//! # }
//! #
//! // Here you can register custom capsules, etc.
//! // This could look like:
//! let some_capsule = unsafe { static_init!(
//!     SomeCapsule,
//!     SomeCapsule::new(deferred_call_mux)
//! ) };
//! some_capsule.set_deferred_call_handle(
//!     deferred_call_mux.register(some_capsule).expect("no deferred call slot available")
//! );
//! ```

use crate::common::cells::OptionalCell;
use core::cell::Cell;

/// Hardware-independent abstraction over a deferred call implementation
///
/// An implementing struct needs to expose this interface in a safe manner.
pub trait DeferredCallMuxBackend {
    /// Set the `client`'s `call` to be scheduled.
    fn set(&self);

    fn set_client(&self, client: &'static DeferredCallMuxBackendClient);
}

pub trait DeferredCallMuxBackendClient {
    /// Called once after a `set` on the backend
    fn call(&self);
}

/// Multiplexer over a hardware-independent
/// [DeferredCallMuxBackend](crate::common::deferred_call_mux::DeferredCallMuxBackend)
///
/// This multiplexer has a fixed number of possible clients, which
/// is determined by the `clients`-array passed in with the constructor.
pub struct DeferredCallMux {
    backend: &'static DeferredCallMuxBackend,
    clients: &'static [(Cell<bool>, OptionalCell<&'static DeferredCallMuxClient>)],
    handle_counter: Cell<usize>,
}
impl DeferredCallMux {
    /// Construct a new deferred call multiplexer over a backend
    ///
    /// The `clients` array can be initialized with any value. The recommended
    /// values are (where n is the number of possible clients):
    ///
    /// `[(Cell::new(false), OptionalCell::empty()); n]`
    pub fn new(
        backend: &'static DeferredCallMuxBackend,
        clients: &'static [(Cell<bool>, OptionalCell<&'static DeferredCallMuxClient>)],
    ) -> DeferredCallMux {
        DeferredCallMux {
            backend: backend,
            clients: clients,
            handle_counter: Cell::new(0),
        }
    }

    /// Schedule a deferred call to be called
    ///
    /// The handle addresses the client that will be called.
    ///
    /// If no client for the handle is found (it was unregistered), this
    /// returns `None`. If a call is already scheduled, it returns
    /// `Some(false)`.
    pub fn set(&self, handle: DeferredCallHandle) -> Option<bool> {
        let DeferredCallHandle(client_pos) = handle;
        let client = &self.clients[client_pos];

        if let (call_set, true) = (&client.0, client.1.is_some()) {
            if call_set.get() {
                // Already set
                Some(false)
            } else {
                call_set.set(true);
                self.backend.set();
                Some(true)
            }
        } else {
            None
        }
    }

    /// Register a new client
    ///
    /// On success, a `Some(handle)` will be returned. This handle is later
    /// required to schedule a deferred call.
    pub fn register(
        &self,
        mux_client: &'static DeferredCallMuxClient,
    ) -> Option<DeferredCallHandle> {
        let current_counter = self.handle_counter.get();

        if current_counter < self.clients.len() {
            let client = &self.clients[current_counter];
            client.0.set(false);
            client.1.set(mux_client);

            self.handle_counter.set(current_counter + 1);

            Some(DeferredCallHandle(current_counter))
        } else {
            None
        }
    }
}

impl DeferredCallMuxBackendClient for DeferredCallMux {
    fn call(&self) {
        self.clients
            .iter()
            .map(|(ref call_reqd, ref oc)| (call_reqd, oc))
            .enumerate()
            .filter(|(_i, (call_reqd, _oc))| call_reqd.get())
            .filter_map(|(i, (call_reqd, oc))| oc.map(|c| (i, call_reqd, *c)))
            .for_each(|(i, call_reqd, client)| {
                call_reqd.set(false);
                client.call(DeferredCallHandle(i));
            });
    }
}

/// Client for the
/// [DeferredCallMux](crate::common::deferred_call_mux::DeferredCallMux)
///
/// This trait needs to be implemented for some struct to receive
/// deferred calls from a `DeferredCallMux`.
pub trait DeferredCallMuxClient {
    fn call(&self, handle: DeferredCallHandle);
}

/// Unique identifier for a deferred call registered with a
/// [DeferredCallMux](crate::common::deferred_call_mux::DeferredCallMux)
#[derive(Copy, Clone, Debug)]
pub struct DeferredCallHandle(usize);

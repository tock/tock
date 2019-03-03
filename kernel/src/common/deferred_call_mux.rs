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
//!     DeferredCallMuxClientState,
//!     set_global_mux,
//! };
//!
//! let deferred_call_mux_clients = unsafe { static_init!(
//!     [DeferredCallMuxClientState; 2],
//!     Default::default()
//! ) };
//! let deferred_call_mux = unsafe { static_init!(
//!     DeferredCallMux,
//!     DeferredCallMux::new(deferred_call_mux_clients)
//! ) };
//! assert!(unsafe { set_global_mux(deferred_call_mux) }, true);
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

static mut DEFERRED_CALL_MUX: Option<&'static DeferredCallMux> = None;

/// Sets a global [DeferredCallMux] instance
///
/// This is required before any deferred calls can be retrieved.
/// It may be called only once. Returns `true` if the mux was successfully
/// registered.
pub unsafe fn set_global_mux(mux: &'static DeferredCallMux) -> bool {
    // If the returned reference is identical to the mux argument,
    // it is set in the option. Otherwise, a different mux is
    // already registered and will not be replaced.
    (*DEFERRED_CALL_MUX.get_or_insert(mux)) as *const _ == mux as *const _
}

/// Call the globally registered mux
///
/// Returns `true` if a Mux was registered and has been called.
/// This function needs to be called by the underlying deferred
/// call implementation in the `chip` crate.
pub unsafe fn call_global_mux() -> bool {
    DEFERRED_CALL_MUX.map(|mux| mux.call()).is_some()
}

/// Check if one or more dynamic deferred calls are pending in the
/// globally registered mux
///
/// Returns `None` if no global mux has been registered, or `Some(true)`
/// if the registered mux has one or more pending deferred calls.
pub unsafe fn global_mux_calls_pending() -> Option<bool> {
    DEFERRED_CALL_MUX.map(|mux| mux.has_pending())
}

/// Internal per-client state tracking for the [DeferredCallMux]
pub struct DeferredCallMuxClientState {
    scheduled: Cell<bool>,
    client: OptionalCell<&'static DeferredCallMuxClient>,
}
impl Default for DeferredCallMuxClientState {
    fn default() -> DeferredCallMuxClientState {
        DeferredCallMuxClientState {
            scheduled: Cell::new(false),
            client: OptionalCell::empty(),
        }
    }
}

/// Multiplexer over [deferred calls](crate::common::deferred_call)
///
/// This multiplexer has a fixed number of possible clients, which
/// is determined by the `clients`-array passed in with the constructor.
pub struct DeferredCallMux {
    client_states: &'static [DeferredCallMuxClientState],
    handle_counter: Cell<usize>,
    call_pending: Cell<bool>,
}

impl DeferredCallMux {
    /// Construct a new deferred call
    ///
    /// This needs to be registered with the `set_global_mux` function immediately
    /// afterwards, and should not be changed anymore. Only the globally registered
    /// Mux will receive calls from the underlying deferred call implementation.
    ///
    /// The `clients` array can be initialized using the implementation of [Default]
    /// for the [DeferredCallMuxClientState].
    pub unsafe fn new(client_states: &'static [DeferredCallMuxClientState]) -> DeferredCallMux {
        DeferredCallMux {
            client_states,
            handle_counter: Cell::new(0),
            call_pending: Cell::new(false),
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
        let client_state = &self.client_states[client_pos];

        if let (call_set, true) = (&client_state.scheduled, client_state.client.is_some()) {
            if call_set.get() {
                // Already set
                Some(false)
            } else {
                call_set.set(true);
                self.call_pending.set(true);
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

        if current_counter < self.client_states.len() {
            let client_state = &self.client_states[current_counter];
            client_state.scheduled.set(false);
            client_state.client.set(mux_client);

            self.handle_counter.set(current_counter + 1);

            Some(DeferredCallHandle(current_counter))
        } else {
            None
        }
    }

    /// Check if one or more deferred calls are pending
    ///
    /// Returns `true` if one or more deferred calls are pending.
    pub fn has_pending(&self) -> bool {
        self.call_pending.get()
    }

    /// Call all registered and to-be-scheduled deferred calls
    ///
    /// This function needs to be called by the underlying deferred call implementation.
    /// It may be called without holding the `DeferredCallMux` reference through
    /// `call_global_mux`.
    pub(self) fn call(&self) {
        if self.call_pending.get() {
            // Reset call_pending here, as it may be set again in the deferred calls
            self.call_pending.set(false);

            self.client_states
                .iter()
                .enumerate()
                .filter(|(_i, client_state)| client_state.scheduled.get())
                .filter_map(|(i, client_state)| {
                    client_state
                        .client
                        .map(|c| (i, &client_state.scheduled, *c))
                })
                .for_each(|(i, call_reqd, client)| {
                    call_reqd.set(false);
                    client.call(DeferredCallHandle(i));
                });
        }
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

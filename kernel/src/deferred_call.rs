// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Hardware-independent kernel interface for deferred calls.
//!
//! This allows any struct in the kernel which implements [`DeferredCallClient`]
//! to set and receive deferred calls, Tock's version of software interrupts.
//!
//! These can be used to implement long-running in-kernel algorithms or software
//! devices that are supposed to work like hardware devices. Essentially, this
//! allows the chip to handle more important interrupts, and lets a kernel
//! component return the function call stack up to the scheduler, automatically
//! being called again.
//!
//! Usage
//! -----
//!
//! The `DEFCALLS` array size determines how many [`DeferredCall`]s may be
//! registered. By default this is set to 32. To support more deferred calls,
//! this file would need to be modified to use a larger variable for `BITMASK`
//! (e.g. `BITMASK` could be a u64 and the array size increased to 64). If more
//! than 32 deferred calls are created, the kernel will panic at the beginning
//! of the kernel loop.
//!
//! ```rust
//! use kernel::deferred_call::{DeferredCall, DeferredCallClient};
//! use kernel::static_init;
//!
//! struct SomeCapsule {
//!     deferred_call: DeferredCall
//! }
//! impl SomeCapsule {
//!     pub fn new() -> Self {
//!         Self {
//!             deferred_call: DeferredCall::new(),
//!         }
//!     }
//! }
//! impl DeferredCallClient for SomeCapsule {
//!     fn handle_deferred_call(&self) {
//!         // Your action here
//!     }
//!
//!     fn register(&'static self) {
//!         self.deferred_call.register(self);
//!     }
//! }
//!
//! // main.rs or your component must register the capsule with its deferred
//! // call. This should look like:
//! let some_capsule = unsafe { static_init!(SomeCapsule, SomeCapsule::new()) };
//! some_capsule.register();
//! ```

use tock_cells::map_cell::MapCell;

use crate::core_local::CoreLocal;
use crate::utilities::cells::OptionalCell;
use core::cell::Cell;
use core::marker::Copy;
use core::marker::PhantomData;

/// This trait should be implemented by clients which need to receive
/// [`DeferredCall`]s.
// This trait is not intended to be used as a trait object; e.g. you should not
// create a `&dyn DeferredCallClient`. The `Sized` supertrait prevents this.
pub trait DeferredCallClient: Sized {
    /// Software interrupt function that is called when the deferred call is
    /// triggered.
    fn handle_deferred_call(&self);

    // This function should be implemented as
    // `self.deferred_call.register(&self);`.
    fn register(&'static self);
}

/// This struct serves as a lightweight alternative to the use of trait objects
/// (e.g. `&dyn DeferredCall`). Using a trait object will include a 20 byte
/// vtable per instance, but this alternative stores only the data and function
/// pointers, 8 bytes per instance.
#[derive(Copy, Clone)]
struct DynDefCallRef<'a> {
    data: *const (),
    callback: fn(*const ()),
    _lifetime: PhantomData<&'a ()>,
}

impl<'a> DynDefCallRef<'a> {
    // SAFETY: We define the callback function as being a closure which casts
    // the passed pointer to be the appropriate type (a pointer to `T`) and then
    // calls `T::handle_deferred_call()`. In practice, the closure is optimized
    // away by LLVM when the ABI of the closure and the underlying function are
    // identical, making this zero-cost, but saving us from having to trust that
    // `fn(*const ())` and `fn handle_deferred_call(&self)` will always have the
    // same calling convention for any type.
    fn new<T: DeferredCallClient>(x: &'a T) -> Self {
        Self {
            data: core::ptr::from_ref(x) as *const (),
            callback: |p| unsafe { T::handle_deferred_call(&*p.cast()) },
            _lifetime: PhantomData,
        }
    }
}

impl DynDefCallRef<'_> {
    // More efficient to pass by `self` if we don't have to implement
    // `DeferredCallClient` directly.
    fn handle_deferred_call(self) {
        (self.callback)(self.data)
    }
}

// The below constant lets us get around Rust not allowing short array
// initialization for non-default types.
const EMPTY: OptionalCell<DynDefCallRef<'static>> = OptionalCell::empty();

/// Counter for the number of deferred calls that have been created, this is
/// used to track that no more than 32 deferred calls have been created.
// All 3 of the below global statics are accessed only in this file, and all
// accesses are via immutable references. Tock is single threaded, so each will
// only ever be accessed via an immutable reference from the single kernel
// thread.
static CTR: CoreLocal<Cell<usize>> = unsafe { CoreLocal::new_single_core(Cell::new(0)) };

/// This bitmask tracks which of the up to 32 existing deferred calls have been
/// scheduled. Any bit that is set in that mask indicates the deferred call with
/// its [`DeferredCall::idx`] field set to the index of that bit has been
/// scheduled and not yet serviced.
static BITMASK: CoreLocal<Cell<u32>> = unsafe { CoreLocal::new_single_core(Cell::new(0)) };

/// An array that stores references to up to 32 `DeferredCall`s via the low-cost
/// [`DynDefCallRef`].
// This is a 256 byte array, but at least resides in `.bss`.
static DEFCALLS: CoreLocal<MapCell<[OptionalCell<DynDefCallRef<'static>>; 32]>> =
    unsafe { CoreLocal::new_single_core(MapCell::new([EMPTY; 32])) };

pub struct DeferredCall {
    idx: usize,
}

impl DeferredCall {
    /// Create a new deferred call with a unique ID.
    pub fn new() -> Self {
        // SAFETY: No accesses to CTR are via an &mut, and the Tock kernel is
        // single-threaded so all accesses will occur from this thread.
        CTR.with(|ctr| {
            let idx = ctr.get() + 1;
            ctr.set(idx);
            DeferredCall { idx }
        })
    }

    // To reduce monomorphization bloat, the non-generic portion of register is
    // moved into this function without generic parameters.
    #[inline(never)]
    fn register_internal_non_generic(&self, handler: DynDefCallRef<'static>) {
        DEFCALLS.with(|defcalls| {
            defcalls.map(|defcalls| {
                if self.idx >= defcalls.len() {
                    // This error will be caught by the scheduler at the beginning of the kernel loop,
                    // which is much better than panicking here, before the debug writer is setup.
                    // Also allows a single panic for creating too many deferred calls instead
                    // of NUM_DCS panics (this function is monomorphized).
                } else {
                    defcalls[self.idx].set(handler);
                }
            })
        });
    }

    /// This function registers the passed client with this deferred call, such
    /// that calls to [`DeferredCall::set()`] will schedule a callback on the
    /// [`handle_deferred_call()`](DeferredCallClient::handle_deferred_call)
    /// method of the passed client.
    pub fn register<DC: DeferredCallClient>(&self, client: &'static DC) {
        let handler = DynDefCallRef::new(client);
        self.register_internal_non_generic(handler);
    }

    /// Schedule a deferred callback on the client associated with this deferred
    /// call.
    pub fn set(&self) {
        BITMASK.with(|bitmask| bitmask.set(bitmask.get() | (1 << self.idx)));
    }

    /// Check if a deferred callback has been set and not yet serviced on this
    /// deferred call.
    pub fn is_pending(&self) -> bool {
        // SAFETY: No accesses to BITMASK are via an &mut, and the Tock kernel is
        // single-threaded so all accesses will occur from this thread.
        BITMASK.with(|bitmask| bitmask.get() & (1 << self.idx) == 1)
    }

    /// Services and clears the next pending [`DeferredCall`], returns which
    /// index was serviced.
    pub fn service_next_pending() -> Option<usize> {
        DEFCALLS.with(|defcalls| {
            defcalls.and_then(|defcalls| {
                let val = BITMASK.with(|bitmask| bitmask.get());
                if val == 0 {
                    None
                } else {
                    let bit = val.trailing_zeros() as usize;
                    let new_val = val & !(1 << bit);
                    BITMASK.with(|bitmask| bitmask.set(new_val));
                    defcalls[bit].map(|dc| {
                        dc.handle_deferred_call();
                        bit
                    })
                }
            })
        })
    }

    /// Returns true if any deferred calls are waiting to be serviced, false
    /// otherwise.
    pub fn has_tasks() -> bool {
        BITMASK.with(|bitmask| bitmask.get() != 0)
    }

    /// This function should be called at the beginning of the kernel loop to
    /// verify that deferred calls have been correctly initialized. This
    /// function verifies two things:
    ///
    /// 1. That <= [`DEFCALLS.len()`] deferred calls have been created, which is
    ///    the maximum this interface supports.
    ///
    /// 2. That exactly as many deferred calls were registered as were created,
    ///    which helps to catch bugs if board maintainers forget to call
    ///    [`register()`](DeferredCall::register) on a created [`DeferredCall`].
    ///
    /// Neither of these checks are necessary for soundness, but they are
    /// necessary for confirming that [`DeferredCall`]s will actually be
    /// delivered as expected. This function costs about 300 bytes, so you can
    /// remove it if you are confident your setup will not exceed 32 deferred
    /// calls, and that all of your components register their deferred calls.
    // Ignore the clippy warning for using `.filter(|opt| opt.is_some())` since
    // we don't actually have an Option (we have an OptionalCell) and
    // IntoIterator is not implemented for OptionalCell.
    #[allow(clippy::iter_filter_is_some)]
    pub fn verify_setup() {
        DEFCALLS.with(|defcalls| {
	    defcalls.map(|defcalls| {
		let num_deferred_calls = CTR.with(|ctr| ctr.get());
		let num_registered_calls = defcalls.iter().filter(|opt| opt.is_some()).count();
		if num_deferred_calls > defcalls.len() {
		    panic!("ERROR: too many deferred calls: {}", num_deferred_calls);
		} else if num_deferred_calls != num_registered_calls {
		    panic!(
			"ERROR: {} deferred calls, {} registered. A component may have forgotten to register a deferred call.",
			num_deferred_calls, num_registered_calls
		    );
		}
	    })
	});
    }
}

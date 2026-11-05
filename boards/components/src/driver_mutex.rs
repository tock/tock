// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Component for initializing DriverMutex instances.

use capsules_core::driver_mutex::{DriverMutex, DriverMutexClient};
use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::deferred_call::DeferredCallClient;
use kernel::utilities::cells::OptionalCell;

/// Statically allocates the storage needed to finalize a [`DriverMutexComponent`].
///
/// `$t` is the concrete type of the underlying resource being guarded. This must match the `T` type
/// parameter of both `DriverMutex` and `DriverMutexComponent`.
///
/// `$n` specifies the number of clients that will be supported by the `DriverMutex`. This must be
/// a valid `const` expression of type `usize`.
#[macro_export]
macro_rules! driver_mutex_component_static {
    ($t:ty, $n:expr) => {{
        (
            kernel::static_buf!(
                [kernel::utilities::cells::OptionalCell<
                    &'static dyn capsules_core::driver_mutex::DriverMutexClient,
                >; $n]
            ),
            kernel::static_buf!([core::mem::MaybeUninit<usize>; $n + 1]),
            kernel::static_buf!(capsules_core::driver_mutex::DriverMutex<$t>),
        )
    }};
}

/// Component helper interface used to initialize a [`DriverMutex`] instance.
///
/// The `C` parameter controls the number of clients that will be supported by the `DriverMutex`,
/// and `Q` controls the size of the internal queue buffer. `Q` must be exactly 1 greater than `C`
/// to account for overhead in Tock's [`kernel::collections::ring_buffer::RingBuffer`]
/// implementation.
///
/// In most cases, you do not need to specify any type parameters explicitly; the compiler can infer
/// them based on the arguments passed to [`driver_mutex_component_static!`]. For instance:
///
/// ```rust,ignore
/// let my_resource = static_init!(MyResource, MyResource::new());
///
/// let my_mutex = DriverMutexComponent::new(my_resource)
///     .finalize(driver_mutex_component_static!(MyResource, 2));
/// ```
pub struct DriverMutexComponent<T: 'static, const C: usize, const Q: usize> {
    resource: &'static T,
}

impl<T, const C: usize, const Q: usize> DriverMutexComponent<T, C, Q> {
    /// Constructs and returns a new instance of [`DriverMutexComponent`].
    ///
    /// `resource` is the underlying resource whose access is being guarded.
    ///
    /// # Panics
    ///
    /// Panics if `Q != C + 1`. See type-level docs above for more info.
    pub fn new(resource: &'static T) -> Self {
        assert_eq!(C + 1, Q);

        Self { resource }
    }
}

// Reason for the where clause is because Default is only implemented for N <= 32.
// https://github.com/rust-lang/rust/issues/61415
impl<T: 'static, const C: usize, const Q: usize> Component for DriverMutexComponent<T, C, Q>
where
    [OptionalCell<&'static dyn DriverMutexClient>; C]: Default,
{
    type StaticInput = (
        &'static mut MaybeUninit<[OptionalCell<&'static dyn DriverMutexClient>; C]>,
        &'static mut MaybeUninit<[MaybeUninit<usize>; Q]>,
        &'static mut MaybeUninit<DriverMutex<T>>,
    );
    type Output = &'static DriverMutex<T>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let clients = s.0.write(Default::default());
        let queue_buf = s.1.write([const { MaybeUninit::uninit() }; Q]);

        let mutex =
            s.2.write(DriverMutex::new(self.resource, clients, queue_buf));
        mutex.register();

        mutex
    }
}

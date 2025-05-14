// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use core::future::Future;
use core::future::IntoFuture;
use core::pin::Pin;

use core::task::Context;
use core::task::Poll;
use core::task::RawWaker;
use core::task::RawWakerVTable;
use core::task::Waker;

use kernel::debug;
use kernel::static_init;
use kernel::utilities::cells::MapCell;
use kernel::ErrorCode;

fn waker_clone<D: AsyncDriver + 'static>(ptr_poller: *const ()) -> RawWaker {
    let executor = unsafe { &*(ptr_poller as *const Executor<D>) };
    RawWaker::new(ptr_poller, executor.waker_vtable())
}

fn waker_wake<D: AsyncDriver + 'static>(ptr_poller: *const ()) {
    let executor = unsafe { &*(ptr_poller as *const Executor<D>) };
    executor.poll();
}

fn waker_wake_by_ref<D: AsyncDriver + 'static>(ptr_poller: *const ()) {
    let executor = unsafe { &*(ptr_poller as *const Executor<D>) };
    executor.poll();
}

fn waker_drop(_ptr_poller: *const ()) {}

pub trait Runner {
    fn execute(&'static self) -> Result<(), ErrorCode>;
}

pub struct Executor<D: AsyncDriver + 'static> {
    future: MapCell<D::F>,
    waker_vtable: &'static RawWakerVTable,
    driver: &'static D,
}

impl<D: AsyncDriver> Executor<D> {
    pub fn new(driver: &'static D) -> Executor<D> {
        Executor {
            future: MapCell::empty(),
            waker_vtable: unsafe {
                static_init!(
                    RawWakerVTable,
                    RawWakerVTable::new(
                        waker_clone::<D>,
                        waker_wake::<D>,
                        waker_wake_by_ref::<D>,
                        waker_drop,
                    )
                )
            },
            driver,
        }
    }

    fn poll(&'static self) {
        debug!("poll");
        let self_ptr = core::ptr::from_ref::<Self>(self) as *const ();
        if let Some(Some(value)) = self.future.map(|future| {
            let waker = unsafe { Waker::from_raw(RawWaker::new(self_ptr, self.waker_vtable)) };
            let mut context = Context::from_waker(&waker);
            match unsafe { Pin::new_unchecked(future) }.poll(&mut context) {
                Poll::Ready(value) => Some(value),
                Poll::Pending => None,
            }
        }) {
            drop(self.future.take());
            self.driver.done(value)
        }
        debug!("poll done");
    }

    fn waker_vtable(&self) -> &'static RawWakerVTable {
        self.waker_vtable
    }
}

impl<D: AsyncDriver> Runner for Executor<D> {
    fn execute(&'static self) -> Result<(), ErrorCode> {
        if self.future.is_none() {
            self.future.replace(self.driver.run());
            self.poll();
            Ok(())
        } else {
            Err(ErrorCode::BUSY)
        }
    }
}

pub trait AsyncDriver {
    type F: Future + 'static;

    /// The asynchronous part of
    /// the driver
    fn run(&'static self) -> Self::F;

    /// Optional methods that is used by the [`Executor`] to
    /// notify the driver that the execution of the future
    /// ended.
    fn done(&self, _value: <Self::F as IntoFuture>::Output) {}
}

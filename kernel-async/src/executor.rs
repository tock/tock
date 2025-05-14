use core::future::Future;
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

fn waker_clone<'a, D: AsyncDriver + 'static>(ptr_poller: *const ()) -> RawWaker {
    let executor = unsafe { &*(ptr_poller as *const Executor<D>) };
    RawWaker::new(ptr_poller, executor.waker_vtable())
}

fn waker_wake<'a, D: AsyncDriver + 'static>(ptr_poller: *const ()) {
    let executor = unsafe { &*(ptr_poller as *const Executor<D>) };
    executor.poll();
}

fn waker_wake_by_ref<'a, D: AsyncDriver + 'static>(ptr_poller: *const ()) {
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

impl<'a, D: AsyncDriver> Executor<D> {
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
        let self_ptr = self as *const Self as *const ();
        if let Some(true) = self.future.map(|future| {
            let waker = unsafe { Waker::from_raw(RawWaker::new(self_ptr, self.waker_vtable)) };
            let mut context = Context::from_waker(&waker);
            match unsafe { Pin::new_unchecked(future) }.poll(&mut context) {
                Poll::Ready(()) => true,
                Poll::Pending => false,
            }
        }) {
            drop(self.future.take());
            self.ready()
        };
        debug!("poll done");
    }
    fn ready(&self) {
        self.driver.done();
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
    type F: Future<Output = ()> + 'static;

    fn run(&'static self) -> Self::F;

    fn done(&self) {}
}

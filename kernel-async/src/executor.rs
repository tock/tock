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
use kernel::utilities::cells::OptionalCell;
use kernel::ErrorCode;

fn waker_clone<'a, T: 'static, F: Future<Output = T> + 'static, I: Fn(T) -> F>(
    ptr_poller: *const (),
) -> RawWaker {
    debug!("clone");
    let executor = unsafe { &*(ptr_poller as *const Executor<'a, T, F, I>) };
    RawWaker::new(ptr_poller, executor.waker_vtable())
}

fn waker_wake<'a, T: 'static, F: Future<Output = T> + 'static, I: Fn(T) -> F>(
    ptr_poller: *const (),
) {
    debug!("wake");
    // let vtable_ptr_poller = unsafe { ptr_poller.offset(1) };
    // let poller: &dyn Poller = unsafe { transmute((ptr_poller, vtable_ptr_poller)) };
    let poller: &dyn Poller =
        unsafe { &*(ptr_poller as *const Executor<'a, T, F, I> as *const dyn Poller) };
    poller.poll();
}

fn waker_wake_by_ref<'a, T: 'static, F: Future<Output = T> + 'static, I: Fn(T) -> F>(
    ptr_poller: *const (),
) {
    debug!("wake_by_ref");
    // let vtable_ptr_poller = unsafe { ptr_poller.offset(1) };
    // let poller: &dyn Poller = unsafe { transmute((ptr_poller, vtable_ptr_poller)) };
    let poller: &dyn Poller =
        unsafe { &*(ptr_poller as *const Executor<'a, T, F, I> as *const dyn Poller) };
    poller.poll();
}

fn waker_drop(_ptr_poller: *const ()) {
    debug!("drop");
}

pub trait Runner<T: 'static> {
    fn execute(&'static self, t: T) -> Result<(), (ErrorCode, T)>;
}

pub trait Poller {
    fn poll(&'static self);
}

pub trait ExecutorClient<T: 'static = ()> {
    fn ready(&self, t: T);
}

pub struct Executor<'a, T: 'static, F: Future<Output = T> + 'static, I: Fn(T) -> F> {
    init: I,
    future: MapCell<F>,
    client: OptionalCell<&'a dyn ExecutorClient<T>>,
    waker_vtable: &'static RawWakerVTable,
}

impl<'a, T: 'static, F: Future<Output = T> + 'static, I: Fn(T) -> F> Executor<'a, T, F, I> {
    pub fn new(init: I) -> Executor<'a, T, F, I> {
        Executor {
            init,
            future: MapCell::empty(),
            client: OptionalCell::empty(),
            waker_vtable: unsafe {
                static_init!(
                    RawWakerVTable,
                    RawWakerVTable::new(
                        waker_clone::<T, F, I>,
                        waker_wake::<T, F, I>,
                        waker_wake_by_ref::<T, F, I>,
                        waker_drop,
                    )
                )
            },
        }
    }

    pub fn set_client(&self, client: &'a dyn ExecutorClient<T>) {
        self.client.replace(client);
    }

    fn ready(&self, v: T) {
        self.client.map(|client| {
            client.ready(v);
        });
    }

    fn waker_vtable(&self) -> &'static RawWakerVTable {
        self.waker_vtable
    }
}

impl<'a, T: 'static, F: Future<Output = T> + 'static, I: Fn(T) -> F> Runner<T>
    for Executor<'a, T, F, I>
{
    fn execute(&'static self, t: T) -> Result<(), (ErrorCode, T)> {
        if self.future.is_none() {
            self.future.replace((self.init)(t));
            self.poll();
            Ok(())
        } else {
            Err((ErrorCode::BUSY, t))
        }
    }
}

impl<'a, T: 'static, F: Future<Output = T> + 'static, I: Fn(T) -> F> Poller
    for Executor<'a, T, F, I>
{
    fn poll(&'static self) {
        debug!("poll");
        let self_ptr = self as *const Self as *const ();
        self.future.map(|future| {
            let waker = unsafe { Waker::from_raw(RawWaker::new(self_ptr, self.waker_vtable)) };
            let mut context = Context::from_waker(&waker);
            match unsafe { Pin::new_unchecked(future) }.poll(&mut context) {
                Poll::Ready(v) => {
                    self.future.take();
                    self.ready(v)
                }
                Poll::Pending => {}
            }
        });
    }
}

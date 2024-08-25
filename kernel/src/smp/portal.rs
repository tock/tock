/// PortalCell

use core::cell::{UnsafeCell, OnceCell};
use crate::threadlocal::ThreadId;

pub trait Portalable {
    type Entrant;

    fn conjure(&self);
    fn teleport(&self, dst: &dyn ThreadId) -> bool;
    fn link(&self, entrant: Self::Entrant) -> Option<()>;
}

pub struct PortalCell<'a, T: ?Sized> {
    id: usize,
    value: UnsafeCell<Option<&'a mut T>>,
    lock: OnceCell<core::ptr::NonNull<T>>,
}

impl<'a, T: ?Sized> PortalCell<'a, T> {

    // Safety? need to be unsafe
    pub fn empty(id: usize) -> PortalCell<'a, T> {
        PortalCell { id, value: UnsafeCell::new(None), lock: OnceCell::new() }
    }

    pub fn new(value: &'a mut T, id: usize) -> PortalCell<'a, T> {
        let lock = OnceCell::new();
        lock.set(value.into()).unwrap_or_else(|_| unreachable!());
        let value = UnsafeCell::new(Some(value));
        PortalCell { id, value, lock }
    }

    pub fn get_id(&self) -> usize {
        self.id
    }

    pub fn is_none(&self) -> bool {
        let inner = self.value.get();
        unsafe { (*inner).is_none() }
    }

    pub fn is_some(&self) -> bool {
        !self.is_none()
    }

    pub fn enter<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&mut T) -> R,
    {
        let inner = self.value.get();
        unsafe { (*inner).as_mut().map(|value| f(value)) }
    }

    pub fn take(&self) -> Option<&'a mut T> {
        let inner = self.value.get();
        unsafe { (*inner).take() }
    }

    pub fn replace(&self, value: &'a mut T) -> bool {
        if self.is_none() {
            if let Some(lock) = self.lock.get() {
                if *lock == value.into() {
                    unsafe {
                        (*self.value.get()).replace(value);
                    }
                    true
                } else {
                    false
                }
            } else {
                self.lock.set(value.into()).unwrap_or_else(|_| unreachable!());
                unsafe {
                    (*self.value.get()).replace(value);
                }
                true
            }
        } else {
            false
        }
    }

}

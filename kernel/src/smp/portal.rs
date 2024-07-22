/// PortalCell

// use core::cell::Cell;
use core::cell::{UnsafeCell, OnceCell};

use crate::platform::platform::KernelResources;
use crate::platform::chip::{Chip, ChipAtomic};
use crate::threadlocal::ThreadId;

pub trait Portalable {
    type Entrant;

    fn conjure(&self);
    fn teleport(&self, dst: &dyn ThreadId);
    fn link(&self, entrant: Self::Entrant) -> Option<()>;
}

// pub struct PortalCell<'a, const TAG: usize, T: ?Sized> {
//     val: Cell<Option<&'a mut T>>,
// }

// impl<'a, const TAG: usize, T: ?Sized> PortalCell<'a, TAG, T> {

//     pub fn empty() -> PortalCell<'a, TAG, T> {
//         PortalCell { val: Cell::new(None) }
//     }

//     pub fn new(value: &'a mut T) -> PortalCell<'a, TAG, T> {
//         PortalCell { val: Cell::new(Some(value)) }
//     }

//     pub fn get_tag(&self) -> usize { TAG }

//     pub fn is_none(&self) -> bool {
//         let inner = self.take();
//         let ret = inner.is_none();
//         self.val.replace(inner);
//         ret
//     }

//     pub fn is_some(&self) -> bool {
//         !self.is_none()
//     }

//     pub fn map<F, R>(&self, f: F) -> Option<R>
//     where
//         F: FnOnce(&mut T) -> R,
//     {
//         self.take()
//             .map(|val| {
//                 let ret = f(val);
//                 self.val.replace(Some(val));
//                 ret
//             })
//     }

//     pub fn take(&self) -> Option<&'a mut T> {
//         self.val.take()
//     }


//     // Safety: PortalCell ensures its inner value is either None or the
//     // same reference at the creation time and thus expose the following
//     pub unsafe fn replace_none(&self, val: &'a mut T) -> Option<()> {
//         let inner = self.take();
//         if inner.is_none() {
//             self.val.replace(Some(val));
//             Some(())
//         } else {
//             self.val.replace(inner);
//             None
//         }
//     }

// }

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

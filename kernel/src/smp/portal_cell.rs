/// PortalCell

// use core::cell::Cell;
use core::cell::UnsafeCell;

use crate::platform::platform::KernelResources;
use crate::platform::chip::{Chip, ChipAtomic};
use crate::threadlocal::ThreadId;

pub trait Portalable<KR: KernelResources<C>, C: Chip + ChipAtomic> {
    type Entrant;

    fn conjure(&self, resources: &KR, chip: &C);
    fn teleport(&self, resources: &KR, chip: &C, dst: &dyn ThreadId);
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
    val: UnsafeCell<Option<&'a mut T>>,
    tag: usize,
}

impl<'a, T: ?Sized> PortalCell<'a, T> {

    // TODO: Need to be unsafe
    pub fn empty(tag: usize) -> PortalCell<'a, T> {
        PortalCell { val: UnsafeCell::new(None), tag }
    }

    pub fn new(value: &'a mut T, tag: usize) -> PortalCell<'a, T> {
        PortalCell { val: UnsafeCell::new(Some(value)), tag }
    }

    pub fn get_tag(&self) -> usize { self.tag }

    pub fn is_none(&self) -> bool {
        let inner = self.val.get();
        unsafe { (*inner).is_none() }
    }

    pub fn is_some(&self) -> bool {
        !self.is_none()
    }

    pub fn enter<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&mut T) -> R,
    {
        let inner = self.val.get();
        unsafe { (*inner).as_mut().map(|val| f(val)) }
    }

    pub fn take(&self) -> Option<&'a mut T> {
        let inner = self.val.get();
        unsafe { (*inner).take() }
    }

    pub unsafe fn replace_none(&self, val: &'a mut T) -> Option<()> {
        if self.is_none() {
            Some(unsafe {
                (*self.val.get()).replace(val);
            })
        } else {
            None
        }
    }

}

// impl<'a, const ID: usize, T: hil::uart::Transmit<'a> + ?Sized> hil::uart::Transmit<'a> for PortalCell<'a, ID, T> {
//     fn set_receive_client(&self, client: &'a dyn hil::uart::ReceiveClient) {
//         todo!();
//     }

//     fn receive_buffer(
//         &self,
//         rx_buffer: &'static mut [u8],
//         rx_len: usize,
//     ) -> Result<(), (ErrorCode, &'static mut [u8])> {
//         todo!();
//     }

//     fn receive_abort(&self) -> Result<(), ErrorCode> {
//         todo!();
//     }

//     fn receive_word(&self) -> Result<(), ErrorCode> {
//         todo!();
//     }
// }


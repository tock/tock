//! Data structure for sharing kernel-owned memory with applications.

use core::cell::Cell;
use core::{
    ops::{Deref, DerefMut},
    ptr::NonNull,
    slice,
};

use crate::{AppId, ReturnCode};

#[derive(Copy, Clone, Eq, PartialEq)]
enum BufferBorrow {
    /// The buffer is not borrowed.
    None,
    /// The buffer is borrowed for exclusive kernel access.
    KernelExclusive,
    /// The buffer is borrowed for exclusive application access.
    AppExclusive(AppId),
}

/// Bookkeeping information on a buffer allocation.
pub struct ZeroCopyBuffer {
    buf: NonNull<u8>,
    physical_len: usize,
    logical_len: Cell<usize>,
    borrow: Cell<BufferBorrow>,
}

impl ZeroCopyBuffer {
    /// Takes ownership of a raw static buffer to create a ZeroCopyBuffer.
    pub fn new(buf: &'static mut [u8]) -> Self {
        Self {
            // Safety: buf is guaranteed to produce a non-null pointer.
            buf: unsafe { NonNull::new_unchecked(buf.as_mut_ptr()) },
            physical_len: buf.len(),
            logical_len: Cell::new(buf.len()),
            borrow: Cell::new(BufferBorrow::None),
        }
    }

    pub fn reset(&self) {
        self.logical_len.set(self.physical_len);
    }

    pub fn len(&self) -> usize {
        self.logical_len.get()
    }

    pub fn set_len(&self, len: usize) {
        // TODO: validate this length.
        self.logical_len.set(len);
    }

    pub fn physical_len(&self) -> usize {
        self.physical_len
    }

    pub fn as_kernel_mut_ref(&'static self) -> Option<KernelRef> {
        match self.borrow.get() {
            BufferBorrow::AppExclusive(_) | BufferBorrow::KernelExclusive => None,
            BufferBorrow::None => {
                self.borrow.set(BufferBorrow::KernelExclusive);
                Some(KernelRef {
                    buf: unsafe {
                        slice::from_raw_parts_mut(self.buf.as_ptr(), self.logical_len.get())
                    },
                    ctx: self,
                })
            }
        }
    }

    pub fn as_app_ref(&'static self, app_id: &AppId) -> Option<AppRef> {
        match self.borrow.get() {
            BufferBorrow::AppExclusive(_) | BufferBorrow::KernelExclusive => None,
            BufferBorrow::None => {
                // TODO: handle failure here.
                unsafe { self.map_into_app(*app_id) };
                self.borrow.set(BufferBorrow::AppExclusive(*app_id));
                Some(AppRef { ctx: self })
            }
        }
    }

    /// Configures the MPU to allow `app_id` to access the zero-copy buffer.
    unsafe fn map_into_app(&'static self, app_id: AppId) -> ReturnCode {
        app_id
            .kernel
            .process_map_or(ReturnCode::FAIL, app_id, |process| {
                match process.add_mpu_region(
                    self.buf.as_ptr() as *const u8,
                    self.physical_len,
                    self.physical_len,
                ) {
                    Some(_) => ReturnCode::SUCCESS,
                    None => ReturnCode::ENOMEM,
                }
            })
    }
}

/// Kernel reference to a zero-copy buffer.
pub struct KernelRef<'a> {
    buf: &'a mut [u8],
    ctx: &'static ZeroCopyBuffer,
}

impl<'a> KernelRef<'a> {
    pub fn into_app_buffer(self, app_id: AppId) -> AppRef {
        // Drop the ZeroCopyBuffer immediately.
        let ctx = self.ctx;
        core::mem::drop(self);
        // This is safe since this function has consumed and dropped the only existing reference to the buffer in kernel space.
        // TODO: handle failure here.
        unsafe { ctx.map_into_app(app_id) };
        ctx.borrow.set(BufferBorrow::AppExclusive(app_id));
        AppRef { ctx: ctx }
    }
}

impl<'a> Deref for KernelRef<'a> {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &[u8] {
        self.buf
    }
}

impl<'a> DerefMut for KernelRef<'a> {
    #[inline]
    fn deref_mut(&mut self) -> &mut [u8] {
        self.buf
    }
}

impl<'a> Drop for KernelRef<'a> {
    fn drop(&mut self) {
        self.ctx.borrow.set(BufferBorrow::None);
    }
}

/// App reference to a zero-copy buffer.
pub struct AppRef {
    ctx: &'static ZeroCopyBuffer,
}

impl AppRef {
    pub fn into_kernel_buffer<'a>(self) -> KernelRef<'a> {
        // Drop the AppZeroCopyBuffer immediately so it's unmapped from app memory.
        let ctx = self.ctx;
        core::mem::drop(self);
        ctx.borrow.set(BufferBorrow::KernelExclusive);
        KernelRef {
            buf: unsafe { slice::from_raw_parts_mut(ctx.buf.as_ptr(), ctx.logical_len.get()) },
            ctx: ctx,
        }
    }

    pub fn as_ptr(&self) -> *const u8 {
        self.ctx.buf.as_ptr()
    }

    pub fn len(&self) -> usize {
        self.ctx.logical_len.get()
    }

    pub fn physical_len(&self) -> usize {
        self.ctx.physical_len
    }
}

impl Drop for AppRef {
    fn drop(&mut self) {
        // TODO: unmap
        self.ctx.borrow.set(BufferBorrow::None);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::static_init;

    #[test]
    fn kernel_ref() {
        const BUF_LEN: usize = 4096;
        let memory = unsafe { static_init!([u8; BUF_LEN], [0; BUF_LEN]) };
        let ctx = unsafe { static_init!(ZeroCopyBuffer, ZeroCopyBuffer::new(memory)) };

        let buf = ctx.as_kernel_mut_ref().expect("failed to borrow buffer");
        assert_eq!(buf.len(), BUF_LEN);
    }
}

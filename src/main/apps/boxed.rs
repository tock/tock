use core::raw::Slice;
use core::ops::{Deref,DerefMut};
use core::ptr::Unique;

use super::app::{app};

/// Header for allocated chunks of memory.
///
/// The `inuse` flag designates whether the Chunk is currently allocated (true)
/// or is free (false).
///
/// The `len` field specifies the size of this Chunk **not including the
/// header**. The Chunk must be at least as large as the value for which it is
/// allocated, but may be larger.
///
/// Allocated data is immeidately after the header in memory.
#[derive(Clone,Copy)]
#[repr(C)]
pub struct Chunk {
    pub inuse: bool,
    pub len: usize
}

impl Chunk {
    unsafe fn data(&self) -> *const u8 {
        (self as *const Chunk).offset(1) as *const u8
    }
}

pub struct BoxMgr {
    pub mem: Slice<u8>,
    pub offset: usize,
    pub drops: usize,
    pub allocs: usize,
    pub chunks: [*mut Chunk; 100]
}

pub struct BoxMgrStats {
    pub allocated_bytes: usize,
    pub num_allocated: usize,
    pub active: usize,
    pub allocs: usize,
    pub drops: usize,
    pub free: usize
}

impl BoxMgr {
    pub fn new(mem_start: *mut u8, mem_size: usize, appsize: usize) -> BoxMgr {
        BoxMgr {
            mem: Slice {
                data: unsafe { mem_start.offset(appsize as isize) },
                len: mem_size - appsize
            },
            offset: 0,
            drops: 0,
            allocs: 0,
            chunks: [0 as *mut Chunk; 100]
        }
    }

    pub fn stats(&self) -> BoxMgrStats {
        let allocated = self.offset;
        let num_allocated = self.chunks.iter().
                filter(|c| !c.is_null()).count();
        let active = unsafe {
            self.chunks.iter().
                filter(|c| !c.is_null() && (***c).inuse).count()
        };
        BoxMgrStats {
            allocated_bytes: allocated,
            num_allocated: num_allocated,
            active: active,
            drops: self.drops,
            allocs: self.allocs,
            free: self.mem.len - num_allocated
        }
    }
}

pub struct Box<T>{ inner: Unique<T> }

impl<T> Box<T> {
    
    pub unsafe fn from_raw(raw: *mut T) -> Box<T> {
        Box { inner: Unique::new(raw) }
    }

    pub fn raw(&self) -> *mut T {
        *self.inner
    }

    pub unsafe fn uninitialized(size: usize) -> Box<T> {
        use core::mem;
        let myapp = &mut (&mut *app).memory;
        myapp.allocs += 1;

        // First, see if there is an available chunk of the right size
        for chunk in myapp.chunks.iter_mut().filter(|c| !c.is_null()) {
            let c : &mut Chunk = mem::transmute(*chunk);
            if !c.inuse && c.len >= size {
                c.inuse = true;
                let data = c.data();
                return Box {
                    inner: Unique::new(data as *mut T)
                };
            }
        }

        match myapp.chunks.iter_mut().filter(|c| c.is_null()).next() {
            Some(slot) => {
                let freemem = myapp.mem.data.offset(myapp.offset as isize);
                let chunk = &mut *(freemem as *mut Chunk);
                myapp.offset += mem::size_of::<Chunk>();

                let chunk_align = mem::align_of::<Chunk>();
                let size = if size % chunk_align == 0 {
                    size
                } else {
                    size + chunk_align - (size % chunk_align)
                };
                chunk.len = size;
                chunk.inuse = true;

                *slot = chunk as *mut Chunk;

                let data = myapp.mem.data.offset(myapp.offset as isize);
                myapp.offset += size;

                Box{ inner: Unique::new(data as *mut T) }
            },
            None => {
                panic!("OOM")
            }
        }
    }

    pub fn new(x: T) -> Box<T> {
        use core::mem;
        use core::intrinsics::copy;

        let size = mem::size_of::<T>();
        unsafe {
            let mut d = Self::uninitialized(size);
            copy(&x, &mut *d, 1);
            mem::forget(x);
            d
        }
    }
}

impl<T> Deref for Box<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe {
            &**self.inner
        }
    }
}

impl<T> DerefMut for Box<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe {
            &mut **self.inner
        }
    }
}

impl<T> Drop for Box<T> {
    fn drop(&mut self) {
        unsafe {
            use core::{mem, ptr};

            mem::drop(ptr::read(*self.inner));

            let chunk = (*self.inner as *mut T as *mut Chunk).offset(-1);
            (&mut *chunk).inuse = false;
            let myapp = &mut (*app).memory;
            myapp.drops += 1;
        }
    }
}

pub unsafe fn uninitialized_box_slice<T>(size: usize) -> Box<&'static mut [T]> {
    use core::mem;
    let slice_size = mem::size_of::<Slice<u8>>();
    let mut bx : Box<Slice<u8>> =
        Box::uninitialized(slice_size + size * mem::size_of::<T>());
    bx.len = size;
    bx.data = (bx.raw()).offset(1) as *const u8;
    mem::transmute(bx)
}


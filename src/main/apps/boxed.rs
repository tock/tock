use core::raw::Slice;
use core::ops::{Deref,DerefMut};
use core::nonzero::NonZero;

use super::app::{app};

#[derive(Clone,Copy)]
struct Chunk {
    inuse: bool,
    slice: Slice<u8>
}

pub struct BoxMgr {
    mem: Slice<u8>,
    offset: usize,
    drops: usize,
    chunks: [*mut Chunk; 100]
}

pub struct BoxMgrStats {
    pub allocated_bytes: usize,
    pub num_allocated: usize,
    pub active: usize,
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
            free: self.mem.len - num_allocated
        }
    }
}

pub struct Box<T: ?Sized>{ pointer: NonZero<*mut T> }

impl<T> Box<T> {
    
    pub fn raw(&self) -> *mut T {
        *self.pointer
    }

    pub unsafe fn uninitialized(size: usize) -> Box<T> {
        use core::mem;
        let myapp = &mut (&mut *app).memory;

        // First, see if there is an available chunk of the right size
        for chunk in myapp.chunks.iter_mut().filter(|c| !c.is_null()) {
            let c = &mut **chunk;
            if !c.inuse && c.slice.len >= size {
                c.inuse = true;
                return Box { pointer: NonZero::new(c.slice.data as *mut T) };
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
                chunk.slice = Slice {
                    data: myapp.mem.data.offset(myapp.offset as isize),
                    len: size
                };

                myapp.offset += size;

                chunk.inuse = true;

                *slot = chunk;

                Box{ pointer: NonZero::new(chunk.slice.data as *mut T) }
            },
            None => {
                panic!("OOM")
            }
        }
    }

    #[allow(dead_code)]
    pub fn new(x: T) -> Box<T> {
        use core::mem;
        let size = mem::size_of::<T>();
        let mut d = unsafe { Self::uninitialized(size) };
        *d = x;
        d
    }
}

impl<T> Deref for Box<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe {
            &**self.pointer
        }
    }
}

impl<T> DerefMut for Box<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe {
            &mut **self.pointer
        }
    }
}

impl<T: ?Sized> Drop for Box<T> {
    fn drop(&mut self) {
        unsafe {
            use core::mem;
            let chunk_size = mem::size_of::<Chunk>() as isize;
            let chunk = (*self.pointer as *mut T as *mut u8)
                            .offset(0 - chunk_size) as *mut Chunk;
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
    bx.data = (*bx.pointer as *const u8).offset(slice_size as isize);
    mem::transmute(bx)
}


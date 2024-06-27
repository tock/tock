use core::any::Any;
use core::fmt::Debug;
use core::ops::{Range, RangeFrom};

use cortex_m_semihosting::{hprint, hprintln};

use crate::ErrorCode;

/// Internal `PacketBufferDyn` trait, shared across various packet buffer
/// backends (such as [`PacketSlice`]).
///
/// This is a safe interface, but should not be used directly. Instead,
/// manipulate `PacketBufferDyn`s using the [`PacketBufferMut`] container.

pub unsafe trait PacketBufferDyn: Any + Debug {
    /// Length of the allocated data in this buffer (excluding head- and
    /// tailroom).
    fn len(&self) -> usize;

    /// Available headroom in the underlying buffer.
    fn headroom(&self) -> usize;

    /// Available tailroom in the underlying buffer.
    fn tailroom(&self) -> usize;

    /// Length of the writeable data in this buffer
    ///
    /// Equal to payload size + headroom available + tailroom available
    fn capacity(&self) -> usize;

    /// Force-reclaim a given amount of headroom in this buffer. This will
    /// ignore any current data stored in the buffer (but not immediately
    /// overwrite it). It will not move past the tailroom marker.
    ///
    /// This method returns a boolean indicating success. A `false` return value
    /// indicates that the `PacketBufferDyn` was not modified.
    fn reclaim_headroom(&mut self, new_headroom: usize) -> bool;

    fn reclaim_tailroom(&mut self, new_tailroom: usize) -> bool;

    /// Force-reset the payload to length `0`, and set a new headroom
    /// pointer. This will ensure that a subsequent prepend operation starts at
    /// this new headroom pointer.
    ///
    /// This method returns a boolean indicating success. It may fail if
    /// `new_headroom > self.headroom() + self.len() + self.tailroom()`. A
    /// `false` return value indicates that the `PacketBufferDyn` was not
    /// modified.
    fn reset(&mut self, new_headroom: usize) -> bool;

    fn copy_from_slice_or_err(&mut self, src: &[u8]) -> Result<(), ErrorCode>;

    fn append_from_slice_max(&mut self, src: &[u8]) -> usize;

    // has to be guaranteed to fit !!!!!
    unsafe fn prepand_unchecked(&mut self, header: &[u8]);

    fn payload(&self) -> &[u8];

    fn payload_mut(&mut self) -> &mut [u8];

    // fn iter_mut<'a>(&'a mut self) -> impl Iterator<Item = &mut u8> + 'a;
}

// TODO: do we need this?
// impl<T: PacketBufferDyn + Any + ?Sized> PacketBufferDyn for &'static T {
//     fn len(&self) -> usize {
//         (**self).len()
//     }
// }

// impl<T: PacketBufferDyn + Any + ?Sized> PacketBufferDyn for &'static mut T {
//     fn len(&self) -> usize {
//         (**self).len()
//     }

//     fn headroom(&self) -> usize {
//         (**self).headroom()
//     }

//     fn tailroom(&self) -> usize {
//         (**self).tailroom()
//     }

//     fn reclaim_headroom(&mut self, new_headroom: usize) -> bool {
//         (**self).reclaim_headroom(new_headroom)
//     }

//     fn reset(&mut self, new_headroom: usize) -> bool {
//         (**self).reset(new_headroom)
//     }

//     fn copy_from_slice_or_err(&mut self, src: &[u8]) -> Result<(), ErrorCode> {
// 	(**self).copy_from_slice_or_err(src)
//     }
// }

/// Mutable reference to a packet buffer, with explicit headroom (`HEAD`) and
/// tailroom (`TAIL`) annotations.
///
/// This wraper type guarantees that the underlying buffer has
/// - a headroom of at least `HEAD` bytes, and
/// - a tailroom of at least `TAIL` bytes.
///
/// Methods on this struct generally consume the original packet buffer
/// reference, and return a new one with different const generic
/// annotations. These methods ensure that the const generic parameters are
/// consistent with the inner reference's advertised head- and tailroom.
///
/// This wrapper can be constructed from an arbitrary mutable [`PacketBufferDyn`]
/// reference using [`PacketBufferMut::new`], which will ensure that the `HEAD`
/// and `TAIL` constraints hold initially.
///
/// The original type can be restored through the [`PacketBufferDyn::downcast`]
/// method. It can also be destructed into its inner reference type using
/// [`PacketBufferDyn::into_inner`].
#[repr(transparent)]
// TODO: should fix the debug trait
#[derive(Debug)]

pub struct PacketBufferMut<const HEAD: usize, const TAIL: usize> {
    pub inner: &'static mut dyn PacketBufferDyn,
}

impl<const HEAD: usize, const TAIL: usize> PacketBufferMut<HEAD, TAIL> {
    #[inline(always)]
    pub fn new(inner: &'static mut dyn PacketBufferDyn) -> Option<Self> {
        //     HEAD,
        //     inner.tailroom(),
        //     TAIL
        // );
        if inner.headroom() >= HEAD && inner.tailroom() >= TAIL {
            Some(PacketBufferMut { inner })
        } else {
            None
        }
    }

    /// Length of the allocated space for the structure
    ///
    ///
    /// Length of the allocated data in this buffer (excluding head- and
    /// tailroom).
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Actual available headroom in the underlying buffer. Must be greater or
    /// equal to the `HEAD` parameter.
    #[inline(always)]
    pub fn headroom(&self) -> usize {
        self.inner.headroom()
    }

    /// Actual available tailroom in the underlying buffer. Must be greater or
    /// equal to the `TAIL` parameter.
    #[inline(always)]
    pub fn tailroom(&self) -> usize {
        self.inner.tailroom()
    }

    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    /// Reduce the advertised headroom of this buffer, without modifying the
    /// underlying reference.
    ///
    /// This uses an assertion to ensure that `NEW_HEAD <= HEAD`. Because this
    /// assertion exclusively uses compile-time accessible constants, a
    /// violation of this constraint is going to result in a compile time
    /// error. However, this error will only be raised when generating the final
    /// monomorphized types, and as such will not occur on builds using `cargo
    /// check`, etc. See [1].
    ///
    /// [1]: https://github.com/rust-lang/rust/issues/99682
    #[inline(always)]
    pub fn reduce_headroom<const NEW_HEAD: usize>(self) -> PacketBufferMut<NEW_HEAD, TAIL> {
        let _: () = assert!(NEW_HEAD <= HEAD);
        PacketBufferMut { inner: self.inner }
    }

    #[inline(always)]
    pub fn reduce_tailroom<const NEW_TAIL: usize>(self) -> PacketBufferMut<HEAD, NEW_TAIL> {
        let _: () = assert!(NEW_TAIL <= TAIL);
        PacketBufferMut { inner: self.inner }
    }

    /// Attempt to restore the headroom of this buffer in a non-destructive way
    /// (not discarding any data in the underlying buffer).
    ///
    /// For this method to return `Ok(_)`, the underlying buffer's
    /// [`PacketBufferDyn::headroom`] must be larger or equal to
    /// `NEW_HEAD`. Otherwise, the old `self` is returned in the `Err(_)`
    /// variant.
    #[inline(always)]
    pub fn restore_headroom<const NEW_HEAD: usize>(
        self,
    ) -> Result<PacketBufferMut<NEW_HEAD, TAIL>, Self> {
        if self.inner.headroom() >= NEW_HEAD {
            Ok(PacketBufferMut { inner: self.inner })
        } else {
            Err(self)
        }
    }

    #[inline(always)]
    pub fn restore_tailroom<const NEW_TAIL: usize>(
        self,
    ) -> Result<PacketBufferMut<HEAD, NEW_TAIL>, Self> {
        if self.inner.tailroom() >= NEW_TAIL {
            Ok(PacketBufferMut { inner: self.inner })
        } else {
            Err(self)
        }
    }

    /// Force-reclaim a given amount of headroom in this buffer.
    ///
    /// This will ignore any current data stored in the buffer (but not
    /// immediately overwrite it). It will not move past the tailroom marker.
    ///
    // TODO: document return value, and that in the `Err(_)` case the buffer has
    // not been modified.
    #[inline(always)]
    pub fn reclaim_headroom<const NEW_HEAD: usize>(
        self,
    ) -> Result<PacketBufferMut<NEW_HEAD, TAIL>, Self> {
        if self.inner.reclaim_headroom(NEW_HEAD) {
            Ok(PacketBufferMut { inner: self.inner })
        } else {
            Err(self)
        }
    }

    #[inline(always)]
    pub fn reclaim_tailroom<const NEW_TAIL: usize>(
        self,
    ) -> Result<PacketBufferMut<HEAD, NEW_TAIL>, Self> {
        if self.inner.reclaim_tailroom(NEW_TAIL) {
            Ok(PacketBufferMut { inner: self.inner })
        } else {
            Err(self)
        }
    }

    pub fn reset<const NEW_HEAD: usize, const NEW_TAIL: usize>(
        self,
    ) -> Result<PacketBufferMut<NEW_HEAD, NEW_TAIL>, Self> {
        if NEW_HEAD + NEW_TAIL < self.inner.capacity() {
            assert!(self.inner.reset(NEW_HEAD));
            Ok(PacketBufferMut { inner: self.inner })
        } else {
            Err(self)
        }
    }

    #[inline(always)]
    pub fn downcast<T: PacketBufferDyn>(self) -> Option<&'static mut T> {
        let any_buffer: &'static mut dyn Any = self.inner as _;
        any_buffer.downcast_mut::<T>()
    }

    pub fn prepend<const NEW_HEAD: usize, const N: usize>(
        self,
        header: &[u8; N],
    ) -> PacketBufferMut<NEW_HEAD, TAIL> {
        // used like this to be a compile time check
        assert!(NEW_HEAD <= HEAD - N);
        unsafe {
            self.inner.prepand_unchecked(header);
        }

        self.reduce_headroom()
    }

    // pub fn append<const NEW_TAIL: usize, const N: usize>(
    pub fn append<const NEW_TAIL: usize>(self, tail: &[u8]) -> PacketBufferMut<HEAD, NEW_TAIL> {
        assert!(NEW_TAIL <= TAIL - tail.len());

        self.inner.append_from_slice_max(tail);
        self.reduce_tailroom()
    }

    pub fn copy_from_slice_or_err(&mut self, src: &[u8]) -> Result<(), ErrorCode> {
        self.inner.copy_from_slice_or_err(src)
    }

    pub fn payload(&self) -> &[u8] {
        self.inner.payload()
    }

    pub fn payload_mut(&mut self) -> &mut [u8] {
        self.inner.payload_mut()
    }
}

// PacketSliceMut is a transparent wrapper around a byte, such that we
// can take a dyn reference to it (must be Sized). We create it from a
// slice by storing the slice's length in the first usize words and
// never modifying that.
#[repr(transparent)]
// TODO: should fix the debug trait
#[derive(Debug)]
pub struct PacketSliceMut {
    // Use the first `core::mem::size_of<usize>()` bytes as the
    // original slice length, second word as headroom, and the third
    // word as tailroom.
    _inner: u8,
}

impl PacketSliceMut {
    const SLICE_LENGTH_BYTES: Range<usize> =
        (0 * core::mem::size_of::<usize>())..(1 * core::mem::size_of::<usize>());
    const HEADROOM_BYTES: Range<usize> =
        (1 * core::mem::size_of::<usize>())..(2 * core::mem::size_of::<usize>());
    const TAILROOM_BYTES: Range<usize> =
        (2 * core::mem::size_of::<usize>())..(3 * core::mem::size_of::<usize>());
    const DATA_SLICE: RangeFrom<usize> = (3 * core::mem::size_of::<usize>())..;

    // TODO: horribly unsafe, check and document safety!
    pub fn new<'a>(
        slice: &'a mut [u8],
        headroom: usize,
    ) -> Result<&'a mut PacketSliceMut, &'a mut [u8]> {
        if slice.len() < Self::DATA_SLICE.start {
            Err(slice)
        } else {
            // Write the slice's length into its first word:
            let length = slice.len();
            slice[Self::SLICE_LENGTH_BYTES].copy_from_slice(&usize::to_ne_bytes(length));

            // Start with zero headroom, and full tailroom (simulating an empty slice)
            slice[Self::HEADROOM_BYTES].copy_from_slice(&usize::to_ne_bytes(headroom));
            slice[Self::TAILROOM_BYTES].copy_from_slice(&usize::to_ne_bytes(
                length - Self::DATA_SLICE.start - headroom,
            ));

            // Discard the slice, storing only a reference to its first
            // byte. The safety of this infrastructure depends on us having
            // written the correct length to the first word
            // (`SLICE_LENGTH_BYTES`), and us _never_ overwriting that word.
            Ok(
                unsafe {
                    core::mem::transmute::<&'a mut u8, &'a mut PacketSliceMut>(&mut slice[0])
                },
            )
        }
    }

    pub fn into_inner(&'static mut self) -> &'static mut [u8] {
        let length = self.get_inner_slice_length();
        unsafe { core::slice::from_raw_parts_mut(self as *mut _ as *mut u8, length) }
    }

    fn get_inner_slice_length(&self) -> usize {
        // We use this function for restoring the inner slice, and as such we
        // must avoid using those methods here. We're only interested in the
        // first word, and the slice is guaranteed to be of sufficient length
        // for that:
        let _: () = assert!(Self::SLICE_LENGTH_BYTES.start == 0);
        let length_slice = unsafe {
            core::slice::from_raw_parts(self as *const _ as *const u8, Self::SLICE_LENGTH_BYTES.end)
        };

        // The `length_slice` is guaranteed to have the correct length (one
        // usize word), so this panic should be elided:
        usize::from_ne_bytes(length_slice.try_into().unwrap())
    }

    fn restore_inner_slice<'a>(&'a self) -> &'a [u8] {
        let length = self.get_inner_slice_length();

        // `get_inner_slice_length` does not keep a reference to the underlying
        // memory in scope, so now construct the final slice with the correct
        // length:
        unsafe { core::slice::from_raw_parts(self as *const _ as *const u8, length) }
    }

    // TODO: document safety. Unsafe because the slice can be used to change the
    // `SLICE_LENGTH_BYTES` attribute.
    unsafe fn restore_inner_slice_mut<'a>(&'a mut self) -> &'a mut [u8] {
        let length = self.get_inner_slice_length();

        // `get_inner_slice_length` does not keep a reference to the underlying
        // memory in scope, so now construct the final slice with the correct
        // length:
        core::slice::from_raw_parts_mut(self as *mut _ as *mut u8, length)
    }

    pub fn get_headroom(&self) -> usize {
        usize::from_ne_bytes(
            self.restore_inner_slice()[Self::HEADROOM_BYTES]
                .try_into()
                .unwrap(),
        )
    }

    fn set_headroom(&mut self, headroom: usize) {
        unsafe {
            self.restore_inner_slice_mut()[Self::HEADROOM_BYTES]
                .copy_from_slice(&usize::to_ne_bytes(headroom));
        }
    }

    fn get_tailroom(&self) -> usize {
        usize::from_ne_bytes(
            self.restore_inner_slice()[Self::TAILROOM_BYTES]
                .try_into()
                .unwrap(),
        )
    }

    fn set_tailroom(&mut self, tailroom: usize) {
        unsafe {
            self.restore_inner_slice_mut()[Self::TAILROOM_BYTES]
                .copy_from_slice(&usize::to_ne_bytes(tailroom));
        }
    }

    pub fn data_slice<'a>(&'a self) -> &'a [u8] {
        let slice = self.restore_inner_slice();

        &slice[Self::DATA_SLICE]
    }

    pub fn data_slice_mut<'a>(&'a mut self) -> &'a mut [u8] {
        unsafe { &mut self.restore_inner_slice_mut()[Self::DATA_SLICE] }
    }
}

//     fn headroom_mut<'a>(&'a mut self) -> &'a mut usize {
// 	let slice = unsafe { core::mem::transmute::<&'a mut Self, &'a mut [u8]>(self) };
// 	unsafe { core::mem::transmute::<&'a mut u8, &'a mut usize>(
// 	    &mut slice[0 * core::mem::size_of::<usize>()]
// 	) }
//     }

//     fn tailroom_mut<'a>(&'a mut self) -> &'a mut usize {
// 	let slice = unsafe { core::mem::transmute::<&'a mut Self, &'a mut [u8]>(self) };
// 	unsafe { core::mem::transmute::<&'a mut u8, &'a mut usize>(
// 	    &mut slice[1 * core::mem::size_of::<usize>()]
// 	) }
//     }

//     fn data_mut<'a>(&'a mut self) -> &'a mut [u8] {
// 	let slice = unsafe { core::mem::transmute::<&'a mut Self, &'a mut [u8]>(self) };
// 	&mut slice[2 * core::mem::size_of::<usize>()..]
//     }

//     fn headroom_mut<'a>(&'a mut self) -> &'a mut usize {
// 	let slice = unsafe { core::mem::transmute::<&'a mut Self, &'a mut [u8]>(self) };
// 	unsafe { core::mem::transmute::<&'a mut u8, &'a mut usize>(
// 	    &mut slice[0 * core::mem::size_of::<usize>()]
// 	) }
//     }

//     fn tailroom_mut<'a>(&'a mut self) -> &'a mut usize {
// 	let slice = unsafe { core::mem::transmute::<&'a mut Self, &'a mut [u8]>(self) };
// 	unsafe { core::mem::transmute::<&'a mut u8, &'a mut usize>(
// 	    &mut slice[1 * core::mem::size_of::<usize>()]
// 	) }
//     }

//     fn data_mut<'a>(&'a mut self) -> &'a mut [u8] {
// 	let slice = unsafe { core::mem::transmute::<&'a mut Self, &'a mut [u8]>(self) };
// 	&mut slice[2 * core::mem::size_of::<usize>()..]
//     }
// }

unsafe impl PacketBufferDyn for PacketSliceMut {
    fn len(&self) -> usize {
        // self.data_slice().len() - self.headroom() - self.tailroom()
        self.get_inner_slice_length()
    }

    fn headroom(&self) -> usize {
        self.get_headroom()
    }

    fn tailroom(&self) -> usize {
        self.get_tailroom()
    }

    fn capacity(&self) -> usize {
        self.data_slice().len()
    }

    fn reclaim_headroom(&mut self, new_headroom: usize) -> bool {
        if new_headroom <= self.data_slice().len() - self.tailroom() {
            self.set_headroom(new_headroom);
            true
        } else {
            false
        }
    }

    fn reclaim_tailroom(&mut self, new_tailroom: usize) -> bool {
        if new_tailroom <= self.data_slice().len() - self.headroom() {
            self.set_tailroom(new_tailroom);
            true
        } else {
            false
        }
    }

    fn reset(&mut self, new_headroom: usize) -> bool {
        if new_headroom > self.data_slice().len() {
            false
        } else {
            self.set_headroom(new_headroom);
            self.set_tailroom(self.data_slice().len() - new_headroom);
            true
        }
    }

    fn copy_from_slice_or_err(&mut self, src: &[u8]) -> Result<(), ErrorCode> {
        let headroom: usize = self.get_headroom();
        let available: &mut [u8] = &mut self.data_slice_mut()[headroom..];
        if available.len() < src.len() {
            Err(ErrorCode::SIZE)
        } else {
            available
                .iter_mut()
                .zip(src.iter())
                .for_each(|(dst, src)| *dst = *src);
            self.set_tailroom(self.data_slice().len() - self.get_headroom() - src.len());
            Ok(())
        }
    }

    fn append_from_slice_max(&mut self, src: &[u8]) -> usize {
        let slice_length = self.data_slice().len();
        let tailroom = self.get_tailroom();
        let offset = slice_length - tailroom;
        let count = core::cmp::min(tailroom, src.len());

        self.data_slice_mut()[offset..(offset + count)]
            .iter_mut()
            .zip(src[..count].iter())
            .for_each(|(dst, src)| *dst = *src);

        self.set_tailroom(tailroom - count);
        count
    }

    unsafe fn prepand_unchecked(&mut self, header: &[u8]) {
        self.set_headroom(self.get_headroom().saturating_sub(header.len()));
        let headroom = self.get_headroom();
        self.data_slice_mut()[headroom..headroom + header.len()].copy_from_slice(header);
    }

    fn payload(&self) -> &[u8] {
        let headroom = self.headroom();
        let tailroom = self.tailroom();
        let capacity = self.capacity();

        //     capacity,
        //     headroom,
        //     tailroom
        // );

        &self.data_slice()[headroom..(capacity - tailroom)]
    }

    fn payload_mut(&mut self) -> &mut [u8] {
        let headroom = self.headroom();
        let tailroom = self.tailroom();
        let capacity = self.capacity();

        &mut self.data_slice_mut()[headroom..(capacity - tailroom)]
    }
    // TODO same for tail

    // fn iter_mut<'a>(&'a mut self) -> impl core::slice::IterMut<Item = &mut u8> + 'a {
    // 	let headroom = self.get_headroom();
    // 	let tailroom = self.get_tailroom();
    // 	let slice = self.data_slice_mut();
    // 	let length = slice.len();
    // 	slice[headroom..(length - tailroom)].iter_mut()
    // }
}

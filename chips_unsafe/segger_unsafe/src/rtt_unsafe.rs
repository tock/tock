// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Unsafe components of the Segger RTT implementation.
//!
//! The RTT protocol defines the in-memory data structure for RTT channels.
//! This requires us to use 'raw slices' (raw pointers and lengths) to
//! represent the channels. It further defines semantics for read and write
//! indicies into the channels, and rules around memory ordering that target
//! devices must manually implement.

use core::marker::PhantomData;
use core::sync::atomic::{fence, Ordering};
use kernel::utilities::cells::VolatileCell;
use kernel::utilities::leasable_buffer::{SubSlice, SubSliceMut};

// The function name and semantics come from this nightly intrinsic:
// https://doc.rust-lang.org/std/intrinsics/fn.volatile_copy_nonoverlapping_memory.html
// which has the semantics we want. But for the moment we manually
// do the implementation to avoid the unstable intrinsic.
//
// SAFETY: Callers must validate that [src] and [dst] are valid buffers of
// [count] size, which do not overlap. (N.b., while the current implementation
// does not exploit the non-overlapping constraint, a future one may, thus it
// must be considered in the interface here.)
unsafe fn volatile_copy_nonoverlapping_memory(dst: *mut u8, src: *const u8, count: usize) {
    for i in 0..count {
        core::ptr::write_volatile(dst.add(i), *src.add(i));
    }
}

/// A channel for communicating from target to host.
///
/// Semantics for "up" channels defined by the RTT protocol:
///  - [channel_name]: C-string, unchanged after init
///  - [buffer]: Target-written ring buffer
///  - [length]: Ring buffer size, unchanged after init
///  - [write_position]: The next location the target will write to
///  - [read_position]: The next location the host will read from
///  - [flags]:
///     - Flags[31:24]: Used for validity check, must be zero.
///     - Flags[23:2]: Reserved for future use.
///     - Flags[1:0]: RTT operating mode.
///        - These are ignored by the host. On Segger-provided implementations,
///          it notes whether the target operates in blocking (2) or
///          non-blocking mode (and whether non-blocking should drop messages
///          when they will not fit (0, default) or write what can fit and trim
///          the rest (1)).
///
/// Note that host-written memory regions _must_ have static lifetimes. The
/// RTT protocol does not allow for configuration to change during runtime.
/// Once a down buffer is created and detected by a debugger, the storage
/// behind [read_position] can be modified at any time and this permission
/// can never be revoked. The `'a` lifetime is provided for the up buffer
/// itself. As this is written by the host (and, at-worst read by a debugger)
/// the storage behind [buffer] does not _need_ to be `'static`, and this
/// allows us to avoid requiring creators to provide a `'static mut` buffer.
#[repr(C)]
pub struct SeggerRttUpBuffer {
    channel_name: *const u8,
    buffer: *mut u8,
    length: u32,
    write_position: VolatileCell<u32>,
    read_position: VolatileCell<u32>,
    flags: u32,
    _lifetime: PhantomData<&'static ()>,
}

impl SeggerRttUpBuffer {
    pub fn new(name: &[u8], buffer: &mut [u8]) -> SeggerRttUpBuffer {
        SeggerRttUpBuffer {
            channel_name: name.as_ptr(),
            buffer: buffer.as_mut_ptr(),
            length: buffer.len() as u32,
            write_position: VolatileCell::new(0),
            read_position: VolatileCell::new(0),
            flags: 0,
            _lifetime: PhantomData,
        }
    }

    /// Busy wait until the debugger has read up buffer contents.
    // Buffer is defined to be empty when write_position==read_position.
    pub fn spin_until_sync(&self) {
        let write_position = self.write_position.get();
        fence(Ordering::SeqCst);
        while write_position != self.read_position.get() {
            core::hint::spin_loop();
        }
    }

    /// Write as many bytes from [src] as possible into up buffer until the up
    /// buffer is full. The [src] subslice is set to the portion of [src] NOT
    /// yet written on function return.
    ///
    /// Note: This function DOES NOT check the debugger read position before
    /// writing. The assumption is that either (1) no debugger is connected,
    /// so waiting for a read would wait forever, or (2) if one is connected,
    /// it can read memory faster than this method is invoked.
    pub fn write_until_full(&self, src: &mut SubSlice<u8>) {
        let index = self.write_position.get() as usize;

        // First, write what we can into the "back half" of the ring.
        let back_half_len = core::cmp::min(self.length as usize - index, src.len());
        unsafe {
            // SAFETY: write_position is defined to be within buffer
            let dst = self.buffer.add(index);
            // SAFETY: both raw slices are defined by us here
            volatile_copy_nonoverlapping_memory(dst, src.as_ptr(), back_half_len);
        }
        let mut new_index = (index + back_half_len) as u32 % self.length;
        src.slice(back_half_len..src.len());

        // Now, if there is anything left in [src], write what we can into the
        // "front half" of the ring
        if src.len() != 0 {
            // We know [new_index] is 0 here, what is more important is that we
            // can write [index]-1 bytes
            let front_half_len = core::cmp::min(index - 1, src.len());
            unsafe {
                // SAFETY: both raw slices are defined by us here
                volatile_copy_nonoverlapping_memory(self.buffer, src.as_ptr(), front_half_len);
            }
            new_index = front_half_len as u32;
            src.slice(front_half_len..src.len());
        }

        // Force memory writes to complete, then advance the write_position
        // pointer so the debugger knows to start a read
        fence(Ordering::SeqCst);
        self.write_position.set(new_index);
        fence(Ordering::SeqCst);
    }
}

/// A channel for communicating from host to target.
///
/// Semantics for "down" channels defined by the RTT protocol:
///  - [channel_name]: C-string, unchanged after init
///  - [buffer]: Host-written ring buffer
///  - [length]: Ring buffer size, unchanged after init
///  - [write_position]: The next location the host will write to
///  - [read_position]: The next location the target will read from
///  - [flags]:
///     - Flags[31:24]: Used for validity check, must be zero.
///     - Flags[23:2]: Reserved for future use.
///     - Flags[1:0]: RTT operating mode.
///         - Nominally the same as up channels, it is unclear from
///           available documentation whether Segger host implementations
///           will respect blocking or preferred trim semantics. The Tock
///           implementation leaves this at 0 (default), which will cause
///           the host to drop messages in full if the buffer is full.
///
/// Note that host-written memory regions _must_ have static lifetimes. The
/// RTT protocol does not allow for configuration to change during runtime.
/// Once a down buffer is created and detected by a debugger, the storage
/// behind [buffer] and [write_position] can be modified at any time and this
/// permission can never be revoked.
#[repr(C)]
pub struct SeggerRttDownBuffer {
    channel_name: *const u8,
    buffer: *const u8,
    length: u32,
    write_position: VolatileCell<u32>,
    read_position: VolatileCell<u32>,
    flags: u32,
    _lifetime: PhantomData<&'static ()>,
}

impl SeggerRttDownBuffer {
    pub fn new(name: &[u8], buffer: &'static [u8]) -> SeggerRttDownBuffer {
        SeggerRttDownBuffer {
            channel_name: name.as_ptr(),
            buffer: buffer.as_ptr(),
            length: buffer.len() as u32,
            write_position: VolatileCell::new(0),
            read_position: VolatileCell::new(0),
            flags: 0,
            _lifetime: PhantomData,
        }
    }

    /// Read available data from the debugger, up to [dst.len()] bytes.
    /// The [dst] subslice region is updated as it is filled, returning
    /// with the unused portion (if any) as the active region.
    pub fn try_read(&self, dst: &mut SubSliceMut<u8>) {
        // Try to read so long as we have room for more data
        while dst.len() != 0 {
            // Ensure all reads/writes to position data have already happened.
            fence(Ordering::SeqCst);
            let write_position = self.write_position.get();
            let read_position = self.read_position.get();

            if write_position == read_position {
                // There is no new data available.
                break;
            }

            // The volatile slice read mechanism is a one-shot linear read. On
            // this iteration, we will either read all available data or read
            // through the end of the ring buffer, and then read from the front
            // on the next iteration.
            let read_through = if write_position > read_position {
                write_position
            } else {
                self.length
            };

            // Read as many bytes are available, up to remaining buffer space.
            let read_length = core::cmp::min(read_through - read_position, self.length);

            unsafe {
                // SAFETY: both raw slices are defined by us here
                volatile_copy_nonoverlapping_memory(
                    dst.as_mut_ptr(),
                    self.buffer.add(read_position as usize),
                    read_length as usize,
                );
            }

            // Inform the host of what we have read.
            self.read_position
                .set((read_position + read_length) % self.length);
        }
        // Make sure final writes are sync'd
        fence(Ordering::SeqCst);
    }
}

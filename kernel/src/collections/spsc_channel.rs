// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! Lock-free single-producer single-consumer (SPSC) ring channels for
//! inter-hart communication.
//!
//! # Types
//!
//! * [`SpscChannel`] — a unidirectional ring channel.  One caller owns the
//!   producer end (`push`) and a different caller owns the consumer end
//!   (`pop`).
//! * [`BiChannel`] — a bidirectional pair: two [`SpscChannel`]s in opposite
//!   directions, accessed through a named "side A" / "side B" interface.
//!
//! # Contrast with [`RingBuffer`]
//!
//! [`kernel::collections::ring_buffer::RingBuffer`] is a single-threaded ring
//! buffer that requires `&mut self` for both enqueue and dequeue.  It is
//! designed for cooperative, single-hart kernel code.
//!
//! [`SpscChannel`] uses `&self` with [`UnsafeCell`] interior mutability and
//! explicit [`core::sync::atomic`] fences, making it safe to use from two
//! concurrently-executing harts — as long as the SPSC invariant holds (one
//! producer, one consumer).
//!
//! # Memory ordering
//!
//! Each direction is an independent SPSC channel with standard
//! acquire/release ordering:
//!
//! * **Producer** (`push`): relaxed load of `tail`; Acquire load of `head`
//!   (so the producer sees the consumer's most-recent advance before checking
//!   for space); write the entry; Release fence (entry data visible before
//!   index advances); relaxed store of `tail`.
//!
//! * **Consumer** (`pop`): relaxed load of `head`; Acquire load of `tail`
//!   (so the consumer sees the producer's entry before reading it); read the
//!   entry; Release fence (entry read complete before index advances);
//!   relaxed store of `head`.
//!
//! This is the standard SPSC ordering proof: the Release fence on the
//! producer pairs with the Acquire load on the consumer and vice-versa,
//! ensuring no data races and no reordering of entry writes across the index
//! update.
//!
//! [`RingBuffer`]: crate::collections::ring_buffer::RingBuffer

use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use core::sync::atomic::{fence, AtomicU32, Ordering};

// ─────────────────────────── SpscChannel ────────────────────────────────────

/// A unidirectional lock-free SPSC ring channel.
///
/// `N` must be a power of two.  The channel can hold up to `N` entries
/// before it reports full.
///
/// # Safety contract
///
/// The caller must ensure the **SPSC invariant**: at most one execution
/// context (hart, thread, or interrupt level) calls [`push`] at any time,
/// and at most one different execution context calls [`pop`] at any time.
/// Violating this causes data races.
///
/// [`push`]: SpscChannel::push
/// [`pop`]: SpscChannel::pop
pub struct SpscChannel<const N: usize, T: Copy> {
    entries: UnsafeCell<[MaybeUninit<T>; N]>,
    /// Advanced only by the producer.
    tail: AtomicU32,
    /// Advanced only by the consumer.
    head: AtomicU32,
}

// Safety: the SPSC invariant (documented above) prevents concurrent writes
// to the same slot and concurrent writes to the same index.
unsafe impl<const N: usize, T: Copy> Sync for SpscChannel<N, T> {}

impl<const N: usize, T: Copy> SpscChannel<N, T> {
    #[allow(clippy::eq_op)]
    const _IS_POW2: () = assert!(N != 0 && (N & (N - 1)) == 0, "N must be a power of two");
    const MASK: u32 = (N as u32) - 1;

    /// Creates a new, empty channel.  Usable in `static` initializers.
    pub const fn new() -> Self {
        SpscChannel {
            entries: UnsafeCell::new([const { MaybeUninit::uninit() }; N]),
            tail: AtomicU32::new(0),
            head: AtomicU32::new(0),
        }
    }

    /// Number of entries currently in the channel.
    ///
    /// Approximate under concurrent use: both indices are read with
    /// `Relaxed`, so the value may be stale by the time it is used.
    pub fn len(&self) -> u32 {
        let tail = self.tail.load(Ordering::Relaxed);
        let head = self.head.load(Ordering::Relaxed);
        tail.wrapping_sub(head)
    }

    /// Returns `true` if the channel contains no entries.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns `true` if the channel is at capacity.
    pub fn is_full(&self) -> bool {
        self.len() == N as u32
    }

    /// Push an entry (producer only).
    ///
    /// Returns `false` without blocking if the channel is full.
    pub fn push(&self, val: T) -> bool {
        let tail = self.tail.load(Ordering::Relaxed);
        // Acquire: see the consumer's most-recent head advance so we don't
        // overwrite a slot that hasn't been read yet.
        let head = self.head.load(Ordering::Acquire);
        if tail.wrapping_sub(head) == N as u32 {
            return false;
        }
        let slot = (tail & Self::MASK) as usize;
        // Safety: we are the sole writer; slot is in bounds.
        unsafe { (*self.entries.get())[slot].write(val) };
        // Release: entry data must be visible before the consumer sees the
        // updated tail.
        fence(Ordering::Release);
        self.tail.store(tail.wrapping_add(1), Ordering::Relaxed);
        true
    }

    /// Pop an entry (consumer only).
    ///
    /// Returns `None` if the channel is currently empty.
    pub fn pop(&self) -> Option<T> {
        let head = self.head.load(Ordering::Relaxed);
        // Acquire: see the producer's tail update so we know the slot is
        // fully written before we read it.
        let tail = self.tail.load(Ordering::Acquire);
        if head == tail {
            return None;
        }
        let slot = (head & Self::MASK) as usize;
        // Safety: we are the sole reader; slot is in bounds and initialized
        // (the producer wrote it before advancing tail, which we just saw).
        let val = unsafe { (*self.entries.get())[slot].assume_init() };
        // Release: finish reading the entry before advancing head (so the
        // producer knows the slot is free).
        fence(Ordering::Release);
        self.head.store(head.wrapping_add(1), Ordering::Relaxed);
        Some(val)
    }

    /// Block until an entry is available, then return it (consumer only).
    ///
    /// Spins using [`core::hint::spin_loop`] while waiting.
    pub fn spin_pop(&self) -> T {
        loop {
            if let Some(val) = self.pop() {
                return val;
            }
            core::hint::spin_loop();
        }
    }
}

// ─────────────────────────── BiChannel ──────────────────────────────────────

/// A bidirectional channel between two execution contexts, "side A" and
/// "side B".
///
/// Internally two independent [`SpscChannel`]s:
/// * `a_to_b` — side A produces, side B consumes.
/// * `b_to_a` — side B produces, side A consumes.
///
/// All SPSC invariants from [`SpscChannel`] apply per direction.
pub struct BiChannel<const N: usize, T: Copy> {
    a_to_b: SpscChannel<N, T>,
    b_to_a: SpscChannel<N, T>,
}

// Safety: derived from the inner SpscChannel Sync impls.
unsafe impl<const N: usize, T: Copy> Sync for BiChannel<N, T> {}

impl<const N: usize, T: Copy> BiChannel<N, T> {
    /// Creates a new, empty bidirectional channel.  Usable in `static`
    /// initializers.
    pub const fn new() -> Self {
        BiChannel {
            a_to_b: SpscChannel::new(),
            b_to_a: SpscChannel::new(),
        }
    }

    // ── Side A (producer on a→b, consumer on b→a) ───────────────────────

    /// Push a value toward side B (call from side A only).
    ///
    /// Returns `false` without blocking if the A→B channel is full.
    pub fn a_send(&self, val: T) -> bool {
        self.a_to_b.push(val)
    }

    /// Pop a value sent by side B (call from side A only).
    ///
    /// Returns `None` if no reply is available yet.
    pub fn a_recv(&self) -> Option<T> {
        self.b_to_a.pop()
    }

    /// Block until side B sends a value, then return it (call from side A
    /// only).
    pub fn a_spin_recv(&self) -> T {
        self.b_to_a.spin_pop()
    }

    // ── Side B (consumer on a→b, producer on b→a) ───────────────────────

    /// Push a value toward side A (call from side B only).
    ///
    /// Returns `false` without blocking if the B→A channel is full.
    pub fn b_send(&self, val: T) -> bool {
        self.b_to_a.push(val)
    }

    /// Pop a value sent by side A (call from side B only).
    ///
    /// Returns `None` if no message is available yet.
    pub fn b_recv(&self) -> Option<T> {
        self.a_to_b.pop()
    }

    /// Block until side A sends a value, then return it (call from side B
    /// only).
    pub fn b_spin_recv(&self) -> T {
        self.a_to_b.spin_pop()
    }
}

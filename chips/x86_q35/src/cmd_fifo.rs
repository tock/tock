// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! Minimal, non-alloc ring FIFO for small fixed-size queues.

// Why `FifoItem::EMPTY` and const init?
// We need to create the FIFO in static memory (no heap). That means the backing
// array `[T; N]` must be initialized in a `const` context. I believe we can't  directly
// call `T::default()` in `const fn`, so `[T::default(); N]` won’t work.
// By requiring `T: FifoItem` with a `const EMPTY`, we can do `[T::EMPTY; N]`
// and safely build the FIFO at compile time

use core::cell::Cell;

pub(crate) trait FifoItem: Copy {
    /// Const zero/empty value used to initialize the backing array.
    const EMPTY: Self;
}

pub struct Fifo<T: Copy + FifoItem, const N: usize> {
    buf: [Cell<T>; N],
    head: Cell<usize>,
    tail: Cell<usize>,
    len: Cell<usize>,
}

impl<T: Copy + FifoItem, const N: usize> Fifo<T, N> {
    pub(crate) const fn new() -> Self {
        Self {
            buf: [const { Cell::new(T::EMPTY) }; N],
            head: Cell::new(0),
            tail: Cell::new(0),
            len: Cell::new(0),
        }
    }

    #[inline]
    pub(crate) fn is_empty(&self) -> bool {
        self.len.get() == 0
    }

    #[inline]
    pub(crate) fn is_full(&self) -> bool {
        self.len.get() == N
    }

    /// Push at head; returns Err(item) if full
    #[inline]
    pub(crate) fn push(&self, item: T) -> Result<(), T> {
        if self.is_full() {
            return Err(item);
        }
        let h = self.head.get();
        self.buf[h].set(item);
        self.head.set((h + 1) % N);
        self.len.set(self.len.get() + 1);
        Ok(())
    }

    /// Copy the current head entry (None if empty).
    #[inline]
    pub(crate) fn peek_copy(&self) -> Option<T> {
        if self.is_empty() {
            None
        } else {
            Some(self.buf[self.tail.get()].get())
        }
    }

    /// Read-modify-write the current head entry; returns closure result.
    #[inline]
    pub(crate) fn peek_update<R>(&self, f: impl FnOnce(&mut T) -> R) -> Option<R> {
        if self.is_empty() {
            None
        } else {
            let t = self.tail.get();
            let mut v = self.buf[t].get();
            let r = f(&mut v);
            self.buf[t].set(v);
            Some(r)
        }
    }

    /// Pop from tail; returns a copy of the removed item.
    #[inline]
    pub(crate) fn pop(&self) -> Option<T> {
        if self.is_empty() {
            return None;
        }
        let t = self.tail.get();
        let item = self.buf[t].get();
        self.tail.set((t + 1) % N);
        self.len.set(self.len.get() - 1);
        Some(item)
    }
}

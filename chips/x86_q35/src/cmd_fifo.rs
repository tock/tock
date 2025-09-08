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

pub(crate) trait FifoItem: Copy {
    /// Const zero/empty value used to initialize the backing array.
    const EMPTY: Self;
}

pub(crate) struct Fifo<T: FifoItem, const N: usize> {
    buf: [T; N],
    head: usize,
    tail: usize,
    len: usize,
}

impl<T: FifoItem, const N: usize> Fifo<T, N> {
    pub(crate) const fn new() -> Self {
        Self {
            buf: [T::EMPTY; N],
            head: 0,
            tail: 0,
            len: 0,
        }
    }

    #[inline]
    pub(crate) fn is_empty(&self) -> bool {
        self.len == 0
    }
    #[inline]
    pub(crate) fn is_full(&self) -> bool {
        self.len == N
    }
    #[inline]
    pub(crate) fn push(&mut self, item: T) -> Result<(), ()> {
        if self.is_full() {
            return Err(());
        }
        self.buf[self.head] = item;
        self.head = (self.head + 1) % N;
        self.len += 1;
        Ok(())
    }

    #[inline]
    pub(crate) fn peek(&self) -> Option<&T> {
        if self.is_empty() {
            None
        } else {
            Some(&self.buf[self.tail])
        }
    }

    #[inline]
    pub(crate) fn peek_mut(&mut self) -> Option<&mut T> {
        if self.is_empty() {
            None
        } else {
            Some(&mut self.buf[self.tail])
        }
    }

    #[inline]
    pub(crate) fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }
        let item = self.buf[self.tail];
        self.tail = (self.tail + 1) % N;
        self.len -= 1;
        Some(item)
    }
}

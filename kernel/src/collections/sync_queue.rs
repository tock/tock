// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Interface for inter-kernel-safe queue structure.

pub trait SyncQueue<T>: Sync + Send {
    /// Returns true if there are any items in the queue, false otherwise.
    fn has_elements(&self) -> bool;

    /// Returns true if the queue is full, false otherwise.
    fn is_full(&self) -> bool;

    /// Returns how many elements are in the queue.
    fn len(&self) -> usize;

    /// If the queue isn't full, add a new element to the back of the queue.
    /// Returns whether the element was added.
    fn enqueue(&self, val: T) -> bool;

    /// Add a new element to the back of the queue, poping one from the front if necessary.
    fn push(&self, val: T) -> Option<T>;

    /// Remove the element from the front of the queue.
    fn dequeue(&self) -> Option<T>;

    /// Remove all elements from the ring buffer.
    fn empty(&self);

    /// Retains only the elements that satisfy the predicate.
    fn retain<F>(&self, f: F)
    where
        F: FnMut(&T) -> bool;
}

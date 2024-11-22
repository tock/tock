// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Interface for queue structure.

pub trait Queue<T> {
    /// Returns true if there are any items in the queue, false otherwise.
    fn has_elements(&self) -> bool;

    /// Returns true if the queue is full, false otherwise.
    fn is_full(&self) -> bool;

    /// Returns how many elements are in the queue.
    fn len(&self) -> usize;

    /// If the queue isn't full, add a new element to the back of the queue.
    /// Returns whether the element was added.
    #[flux_rs::sig(fn(self: &strg Self, _) -> bool ensures self: Self)]
    fn enqueue(&mut self, val: T) -> bool;

    /// Add a new element to the back of the queue, poping one from the front if necessary.
    #[flux_rs::sig(fn(self: &strg Self, _) -> Option<T> ensures self: Self)]
    fn push(&mut self, val: T) -> Option<T>;

    /// Remove the element from the front of the queue.
    #[flux_rs::sig(fn(self: &strg Self) -> Option<T> ensures self: Self)]
    fn dequeue(&mut self) -> Option<T>;

    /// Remove and return one (the first) element that matches the predicate.
    #[flux_rs::sig(fn(self: &strg Self, _) -> Option<T> ensures self: Self)]
    fn remove_first_matching<F>(&mut self, f: F) -> Option<T>
    where
        F: Fn(&T) -> bool;

    /// Remove all elements from the ring buffer.
    #[flux_rs::sig(fn(self: &strg Self) ensures self: Self)]
    fn empty(&mut self);

    /// Retains only the elements that satisfy the predicate.
    #[flux_rs::sig(fn(self: &strg Self, _) ensures self: Self)]
    fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&T) -> bool;
}

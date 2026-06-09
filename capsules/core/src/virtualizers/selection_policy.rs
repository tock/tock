// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive SRL 2026.

//! Selection Policy that determines which device to choose first in the virtualizer.

use core::cell::Cell;

/// Trait that determines which device to choose first
/// from the list of devices in the virtualizer.
pub trait SelectionPolicy<I> {
    /// Function that selects the device from the list based on the predicate.
    fn select<It>(&self, it: It, ready: impl Fn(&I) -> bool) -> Option<I>
    where
        It: Iterator<Item = I> + Clone;
}

/// Default policy which selects the first ready device starting from the beginning of the list.
pub struct InsertionFirstPolicy;

impl<I> SelectionPolicy<I> for InsertionFirstPolicy {
    fn select<It>(&self, mut it: It, ready: impl Fn(&I) -> bool) -> Option<I>
    where
        It: Iterator<Item = I>,
    {
        it.find(|node| ready(node))
    }
}

/// Round Robin policy which selects the first ready device based on the last access position.
/// It stores the position of the last selected device to introduce fairness.
pub struct RoundRobinPolicy {
    last_access_position: Cell<usize>,
}

impl<I> SelectionPolicy<I> for RoundRobinPolicy {
    fn select<It>(&self, it: It, ready: impl Fn(&I) -> bool) -> Option<I>
    where
        It: Iterator<Item = I> + Clone,
    {
        it.clone()
            .enumerate()
            .skip(self.last_access_position.get() + 1)
            .chain(it.enumerate().take(self.last_access_position.get()))
            .find(|(_, node)| ready(node))
            .map(|(index, node)| {
                self.last_access_position.set(index);
                node
            })
    }
}

impl Default for RoundRobinPolicy {
    fn default() -> Self {
        Self {
            last_access_position: Cell::new(0),
        }
    }
}

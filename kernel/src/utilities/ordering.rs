// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive SRL 2025.

//! Support for data ordering.

use super::pair::Pair;

use core::fmt::Display;
use core::marker::PhantomData;
use core::num::NonZero;
use core::ops::Sub;

/// A relation between two values.
pub trait Relation<T> {
    fn relation(first: &T, second: &T) -> bool;
}

/// Smaller relation.
pub enum Smaller {}
/// Smaller or equal relation.
pub enum SmallerOrEqual {}

impl<T: Ord> Relation<T> for Smaller {
    fn relation(first: &T, second: &T) -> bool {
        first.lt(second)
    }
}

impl<T: Ord> Relation<T> for SmallerOrEqual {
    fn relation(first: &T, second: &T) -> bool {
        first.le(second)
    }
}

/// Two values that respect the given relation.
#[repr(transparent)]
pub struct RelationalPair<T, R: Relation<T>> {
    pair: Pair<T, T>,
    phantom_data: PhantomData<R>,
}

/// Two references that respect the given relation.
#[repr(transparent)]
pub struct RelationalPairImmutableReference<'a, T, R: Relation<T>> {
    pair: Pair<&'a T, &'a T>,
    phantom_data: PhantomData<R>,
}

impl<T, R: Relation<T>> RelationalPair<T, R> {
    /// # Safety
    ///
    /// The caller must ensure that R::relation(first, second) is true.
    pub const unsafe fn new_unchecked(first: T, second: T) -> Self {
        let pair = Pair::new(first, second);

        Self {
            pair,
            phantom_data: PhantomData,
        }
    }

    pub fn new(first: T, second: T) -> Result<Self, ()> {
        if R::relation(&first, &second) {
            // SAFETY: because of the if condition, R::relation(first, second) == true
            let value = unsafe { Self::new_unchecked(first, second) };
            Ok(value)
        } else {
            Err(())
        }
    }

    pub const fn as_pair(&self) -> &Pair<T, T> {
        &self.pair
    }

    pub fn to_pair(self) -> Pair<T, T> {
        self.pair
    }

    pub const fn as_first(&self) -> &T {
        self.as_pair().as_first()
    }

    pub const fn as_second(&self) -> &T {
        self.as_pair().as_second()
    }

    pub fn to_first(self) -> T {
        self.pair.to_first()
    }

    pub fn to_second(self) -> T {
        self.pair.to_second()
    }

    pub fn consume(self) -> (T, T) {
        self.to_pair().consume()
    }
}

impl<T: Ord + core::fmt::LowerHex> Display for SmallerPair<T> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            formatter,
            "({:#x}, {:#x})",
            self.as_first(),
            self.as_second()
        )
    }
}

impl<'a, T, R: Relation<T>> RelationalPairImmutableReference<'a, T, R> {
    /// # Safety
    ///
    /// The caller must ensure that R::relation(first, second) is true.
    pub const unsafe fn new_unchecked(first: &'a T, second: &'a T) -> Self {
        let pair = Pair::new(first, second);

        Self {
            pair,
            phantom_data: PhantomData,
        }
    }

    pub fn new(first: &'a T, second: &'a T) -> Result<Self, ()> {
        if R::relation(first, second) {
            // SAFETY: because of the if condition, R::relation(first, second) == true
            let value = unsafe { Self::new_unchecked(first, second) };
            Ok(value)
        } else {
            Err(())
        }
    }

    pub const fn as_pair(&self) -> &Pair<&'a T, &'a T> {
        &self.pair
    }

    pub fn to_pair(self) -> Pair<&'a T, &'a T> {
        self.pair
    }

    pub const fn as_first(&self) -> &'a T {
        self.as_pair().as_first()
    }

    pub const fn as_second(&self) -> &'a T {
        self.as_pair().as_second()
    }

    pub fn to_first(self) -> &'a T {
        self.pair.to_first()
    }

    pub fn to_second(self) -> &'a T {
        self.pair.to_second()
    }

    pub fn consume(self) -> (&'a T, &'a T) {
        self.to_pair().consume()
    }
}

pub type SmallerPair<T> = RelationalPair<T, Smaller>;
pub type SmallerOrEqualPair<T> = RelationalPair<T, SmallerOrEqual>;
pub type SmallerOrEqualPairImmutableReference<'a, T> =
    RelationalPairImmutableReference<'a, T, SmallerOrEqual>;

impl<T: Ord> SmallerPair<T> {
    pub const fn as_smaller(&self) -> &T {
        self.as_first()
    }

    pub const fn as_bigger(&self) -> &T {
        self.as_second()
    }

    pub fn to_smaller(self) -> T {
        self.to_first()
    }

    pub fn to_bigger(self) -> T {
        self.to_second()
    }

    pub fn intersect_with_left<'a>(
        &'a self,
        value: &'a T,
    ) -> Result<SmallerOrEqualPair<&'a T>, ()> {
        let smaller = self.as_smaller();
        let bigger = self.as_bigger();

        if value < bigger {
            SmallerOrEqualPair::new(smaller, value)
        } else {
            Err(())
        }
    }

    pub fn contain_with_left<'a>(&'a self, value: &'a T) -> Result<SmallerPair<&'a T>, ()> {
        let smaller = self.as_smaller();
        let bigger = self.as_bigger();

        if value < bigger {
            SmallerPair::new(smaller, value)
        } else {
            Err(())
        }
    }
}

impl<T: Ord> SmallerPair<T>
where
    for<'a> &'a T: Sub<Output = isize>,
{
    pub fn compute_difference(&self) -> NonZero<usize> {
        let smaller = self.as_smaller();
        let bigger = self.as_bigger();
        // CAST: `bigger` > `smaller` ==> `bigger` - `smaller` > 0
        let difference = bigger.sub(smaller) as usize;
        // SAFETY: `bigger` > `smaller` ==> `bigger` - `smaller` > 0
        unsafe { NonZero::new_unchecked(difference) }
    }
}

impl<T: Ord> SmallerOrEqualPair<T> {
    pub const fn as_smaller(&self) -> &T {
        self.as_first()
    }

    pub const fn as_bigger(&self) -> &T {
        self.as_second()
    }

    pub fn to_smaller(self) -> T {
        self.to_first()
    }

    pub fn to_bigger(self) -> T {
        self.to_second()
    }
}

impl<T: Ord> SmallerOrEqualPair<T>
where
    for<'a> &'a T: Sub<Output = isize>,
{
    pub fn compute_difference(&self) -> usize {
        let smaller = self.as_smaller();
        let bigger = self.as_bigger();
        // CAST: `bigger` >= `smaller` ==> `bigger` - `smaller` >= 0
        bigger.sub(smaller) as usize
    }
}

impl<'a, T: Ord> SmallerOrEqualPairImmutableReference<'a, T> {
    pub const fn as_smaller(&self) -> &'a T {
        self.as_first()
    }

    pub const fn as_bigger(&self) -> &'a T {
        self.as_second()
    }

    pub fn to_smaller(self) -> &'a T {
        self.to_first()
    }

    pub fn to_bigger(self) -> &'a T {
        self.to_second()
    }
}

impl<'a, T: Ord> SmallerOrEqualPairImmutableReference<'a, T>
where
    &'a T: Sub<Output = isize>,
{
    pub fn compute_difference(&self) -> usize {
        let smaller = self.as_smaller();
        let bigger = self.as_bigger();
        // CAST: `bigger` >= `smaller` ==> `bigger` - `smaller` >= 0
        bigger.sub(smaller) as usize
    }
}

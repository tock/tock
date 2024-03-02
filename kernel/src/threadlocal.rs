// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use core::cell::UnsafeCell;

unsafe trait ThreadId {}

#[derive(Copy, Clone, Debug)]
pub struct ConstThreadId<const THREAD_ID: usize>;

impl<const THREAD_ID: usize> ConstThreadId<THREAD_ID> {
    pub unsafe fn new() -> Self {
        ConstThreadId
    }

    pub fn dyn_id(&self) -> DynThreadId {
        DynThreadId(THREAD_ID)
    }
}

unsafe impl<const THREAD_ID: usize> ThreadId for ConstThreadId<THREAD_ID> {}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct DynThreadId(usize);

impl DynThreadId {
    pub unsafe fn new(thread_id: usize) -> Self {
        DynThreadId(thread_id)
    }
}

unsafe impl ThreadId for DynThreadId {}

#[repr(transparent)]
pub struct ThreadLocal<const NUM_THREADS: usize, T>(UnsafeCell<[T; NUM_THREADS]>);

impl<const NUM_THREADS: usize, T: Copy> ThreadLocal<NUM_THREADS, T> {
    pub const fn init(init: T) -> Self {
        ThreadLocal(UnsafeCell::new([init; NUM_THREADS]))
    }
}

impl<const NUM_THREADS: usize, T> ThreadLocal<NUM_THREADS, T> {
    pub const fn new(val: [T; NUM_THREADS]) -> Self {
        ThreadLocal(UnsafeCell::new(val))
    }

    #[inline(always)]
    fn get_cell_slice<'a>(&'a self) -> &'a [UnsafeCell<T>; NUM_THREADS] {
        unsafe {
            core::mem::transmute::<&UnsafeCell<[T; NUM_THREADS]>, &[UnsafeCell<T>; NUM_THREADS]>(
                &self.0,
            )
        }
    }
}

unsafe trait ThreadLocalAccess<ID: ThreadId, T> {
    fn get_mut(&self, _id: ID) -> Option<*mut T>;
}

unsafe impl<const NUM_THREADS: usize, const THREAD_ID: usize, T>
    ThreadLocalAccess<ConstThreadId<THREAD_ID>, T> for ThreadLocal<NUM_THREADS, T>
{
    #[inline(always)]
    fn get_mut(&self, _id: ConstThreadId<THREAD_ID>) -> Option<*mut T> {
        let _: () = assert!(THREAD_ID < NUM_THREADS);
        Some(self.get_cell_slice()[THREAD_ID].get())
    }
}

unsafe impl<const NUM_THREADS: usize, T> ThreadLocalAccess<DynThreadId, T>
    for ThreadLocal<NUM_THREADS, T>
{
    #[inline(always)]
    fn get_mut(&self, id: DynThreadId) -> Option<*mut T> {
        self.get_cell_slice().get(id.0).map(|uc| uc.get())
    }
}

// TODO: document safety
unsafe impl<const NUM_THREADS: usize, T> Sync for ThreadLocal<NUM_THREADS, T> {}

// Needs to be unsafe, because will return a pointer that is going to
// be dereferenced.
pub unsafe trait ThreadLocalDyn<T> {
    fn get_mut(&self) -> Option<*mut T>;
}

// Can implement directly if we have no threads:
unsafe impl<T> ThreadLocalDyn<T> for ThreadLocal<0, T> {
    fn get_mut(&self) -> Option<*mut T> {
        None
    }
}

pub struct SingleThread<T>(ThreadLocal<1, T>);

impl<T> SingleThread<T> {
    pub unsafe fn new(val: T) -> Self {
        SingleThread(ThreadLocal::new([val]))
    }
}

unsafe impl<T> ThreadLocalDyn<T> for SingleThread<T> {
    fn get_mut(&self) -> Option<*mut T> {
        <ThreadLocal<1, T> as ThreadLocalAccess<ConstThreadId<0>, T>>::get_mut(&self.0, unsafe {
            ConstThreadId::<0>::new()
        })
    }
}

impl<T> core::ops::Deref for SingleThread<T> {
    type Target = ThreadLocal<1, T>;

    fn deref(&self) -> &Self::Target {
	&self.0
    }
}

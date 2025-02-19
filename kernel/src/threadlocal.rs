// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use core::cell::UnsafeCell;

pub unsafe trait ThreadId {
    fn get_id(&self) -> usize;
}

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

unsafe impl<const THREAD_ID: usize> ThreadId for ConstThreadId<THREAD_ID> {
    #[inline(always)]
    fn get_id(&self) -> usize {
        THREAD_ID
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct DynThreadId(usize);

impl DynThreadId {
    pub unsafe fn new(thread_id: usize) -> Self {
        DynThreadId(thread_id)
    }
}

unsafe impl ThreadId for DynThreadId {
    #[inline(always)]
    fn get_id(&self) -> usize {
        self.0
    }
}

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

pub unsafe trait ThreadLocalAccess<ID: ThreadId, T> {
    fn get_mut<'a>(&'a self, id: ID) -> Option<NonReentrant<'a, T>>;
}

unsafe impl<const NUM_THREADS: usize, const THREAD_ID: usize, T>
    ThreadLocalAccess<ConstThreadId<THREAD_ID>, T> for ThreadLocal<NUM_THREADS, T>
{
    #[inline(always)]
    fn get_mut<'a>(&'a self, _id: ConstThreadId<THREAD_ID>) -> Option<NonReentrant<'a, T>> {
        let _: () = assert!(THREAD_ID < NUM_THREADS);
        Some(NonReentrant(&self.get_cell_slice()[THREAD_ID]))
    }
}

unsafe impl<const NUM_THREADS: usize, T>
    ThreadLocalAccess<DynThreadId, T> for ThreadLocal<NUM_THREADS, T>
{
    #[inline(always)]
    fn get_mut<'a>(&'a self, id: DynThreadId) -> Option<NonReentrant<'a, T>> {
        self.get_cell_slice().get(id.0).map(move |uc| NonReentrant(uc))
    }
}

pub unsafe trait ThreadLocalAssumeInit<ID: ThreadId, T> {
    unsafe fn assume_init_mut<'a>(&'a self, id: ID) -> Option<NonReentrant<'a, T>>;
}

unsafe impl<const NUM_THREADS: usize, const THREAD_ID: usize, T>
    ThreadLocalAssumeInit<ConstThreadId<THREAD_ID>, T> for ThreadLocal<NUM_THREADS, core::mem::MaybeUninit<T>>
{
    #[inline(always)]
    unsafe fn assume_init_mut<'a>(&'a self, _id: ConstThreadId<THREAD_ID>) -> Option<NonReentrant<'a, T>> {
        let _: () = assert!(THREAD_ID < NUM_THREADS);
        Some(NonReentrant(
            core::mem::transmute::<&UnsafeCell<core::mem::MaybeUninit<T>>, &UnsafeCell<T>>(
                &self.get_cell_slice()[THREAD_ID]
            )
        ))
    }
}

unsafe impl<const NUM_THREADS: usize, T>
    ThreadLocalAssumeInit<DynThreadId, T> for ThreadLocal<NUM_THREADS, core::mem::MaybeUninit<T>>
{
    #[inline(always)]
    unsafe fn assume_init_mut<'a>(&'a self, id: DynThreadId) -> Option<NonReentrant<'a, T>> {
        self.get_cell_slice().get(id.0).map(move |uc| NonReentrant(
            core::mem::transmute::<&UnsafeCell<core::mem::MaybeUninit<T>>, &UnsafeCell<T>>(uc)
        ))
    }
}

// ----------------

// TODO: document safety
unsafe impl<const NUM_THREADS: usize, T> Sync for ThreadLocal<NUM_THREADS, T> {}

// Needs to be unsafe, because will return a pointer that is going to
// be dereferenced.
pub unsafe trait ThreadLocalDyn<T> {
    fn get_mut<'a>(&'a self) -> Option<NonReentrant<'a, T>>;
}

pub trait ThreadLocalDynInit<T>: ThreadLocalDyn<T> {
    unsafe fn init(init: T) -> Self;
}

// Can implement directly if we have no threads:
unsafe impl<T> ThreadLocalDyn<T> for ThreadLocal<0, T> {
    fn get_mut<'a>(&'a self) -> Option<NonReentrant<'a, T>> {
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
    fn get_mut<'a>(&'a self) -> Option<NonReentrant<'a, T>> {
        <ThreadLocal<1, T> as ThreadLocalAccess<ConstThreadId<0>, T>>::get_mut(&self.0, unsafe {
            ConstThreadId::<0>::new()
        })
    }
}

impl<T: Copy> ThreadLocalDynInit<T> for SingleThread<T> {
    unsafe fn init(init: T) -> Self {
	    SingleThread(ThreadLocal::new([init]))
    }
}

impl<T> core::ops::Deref for SingleThread<T> {
    type Target = ThreadLocal<1, T>;

    fn deref(&self) -> &Self::Target {
	    &self.0
    }
}


// -----------------------------------------------------------------------------

pub struct NonReentrant<'a, T>(&'a UnsafeCell<T>);

impl<'a, T> NonReentrant<'a, T> {
    pub unsafe fn enter_nonreentrant<R, F: FnOnce(&mut T) -> R>(&self, f: F) -> R {
	    f(&mut *self.0.get())
    }
}

impl<'a, T> NonReentrant<'a, core::mem::MaybeUninit<T>> {
    pub unsafe fn enter_nonreentrant_assume_init<R, F: FnOnce(&mut T) -> R>(&self, f: F) -> R {
	    f(core::mem::MaybeUninit::<T>::assume_init_mut(&mut *self.0.get()))
    }
}

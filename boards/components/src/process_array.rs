// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! Component for the array of process references used by the kernel.

use core::mem::MaybeUninit;
use kernel::component::Component;

#[macro_export]
macro_rules! process_array_component_static {
    ($NUM_PROCS:ty $(,)?) => {{
        kernel::static_buf!(kernel::process::ProcessArray<$NUM_PROCS>)
    };};
}

pub struct ProcessArrayComponent<const NUM_PROCS: usize> {}

impl<const NUM_PROCS: usize> ProcessArrayComponent<NUM_PROCS> {
    pub fn new() -> Self {
        Self {}
    }
}

impl<const NUM_PROCS: usize> Component for ProcessArrayComponent<NUM_PROCS> {
    type StaticInput = &'static mut MaybeUninit<kernel::process::ProcessArray<NUM_PROCS>>;
    type Output = &'static kernel::process::ProcessArray<NUM_PROCS>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        static_buffer.write(kernel::process::ProcessArray::new())
    }
}

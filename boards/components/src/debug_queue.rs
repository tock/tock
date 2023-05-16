// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Component for DebugQueue, the implementation for `debug_enqueue!` and
//! `debug_flush_queue!`.
//!
//! This provides one `Component`, `DebugQueue`, which creates a kernel debug
//! queue using the provided buffer. Data is appended to the queue with
//! `debug_enqueue!`, and can be later flushed to the debug buffer with
//! `debug_flush_queue!`. Any data left on the queue is flushed upon panic.
//!
//! Usage
//! -----
//! ```rust
//! DebugQueueComponent::new().finalize(components::debug_queue_component_static!());
//! ```

// Author: Guillaume Endignoux <guillaumee@google.com>
// Last modified: 05 Mar 2020

use core::mem::MaybeUninit;
use kernel::collections::ring_buffer::RingBuffer;
use kernel::component::Component;

#[macro_export]
macro_rules! debug_queue_component_static {
    () => {{
        let ring = kernel::static_buf!(kernel::collections::ring_buffer::RingBuffer<'static, u8>);
        let queue = kernel::static_buf!(kernel::debug::DebugQueue);
        let wrapper = kernel::static_buf!(kernel::debug::DebugQueueWrapper);
        let buffer = kernel::static_buf!([u8; 1024]);

        (ring, queue, wrapper, buffer)
    };};
}

pub struct DebugQueueComponent {}

impl DebugQueueComponent {
    pub fn new() -> Self {
        Self {}
    }
}

impl Component for DebugQueueComponent {
    type StaticInput = (
        &'static mut MaybeUninit<RingBuffer<'static, u8>>,
        &'static mut MaybeUninit<kernel::debug::DebugQueue>,
        &'static mut MaybeUninit<kernel::debug::DebugQueueWrapper>,
        &'static mut MaybeUninit<[u8; 1024]>,
    );
    type Output = ();

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let buffer = s.3.write([0; 1024]);
        let ring_buffer = s.0.write(RingBuffer::new(buffer));
        let debug_queue = s.1.write(kernel::debug::DebugQueue::new(ring_buffer));
        let debug_queue_wrapper =
            s.2.write(kernel::debug::DebugQueueWrapper::new(debug_queue));
        unsafe {
            kernel::debug::set_debug_queue(debug_queue_wrapper);
        }
    }
}

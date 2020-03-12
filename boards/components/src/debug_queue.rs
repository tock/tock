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
//! let buf = static_init!([u8; 1024], [0; 1024]);
//! DebugQueueComponent::new(buf).finalize(());
//! ```

// Author: Guillaume Endignoux <guillaumee@google.com>
// Last modified: 05 Mar 2020

use kernel::component::Component;
use kernel::static_init;

pub struct DebugQueueComponent {
    buffer: &'static mut [u8],
}

impl DebugQueueComponent {
    pub fn new(buffer: &'static mut [u8]) -> Self {
        Self { buffer }
    }
}

impl Component for DebugQueueComponent {
    type StaticInput = ();
    type Output = ();

    unsafe fn finalize(self, _s: Self::StaticInput) -> Self::Output {
        let ring_buffer = static_init!(
            kernel::common::RingBuffer<'static, u8>,
            kernel::common::RingBuffer::new(self.buffer)
        );
        let debug_queue = static_init!(
            kernel::debug::DebugQueue,
            kernel::debug::DebugQueue::new(ring_buffer)
        );
        let debug_queue_wrapper = static_init!(
            kernel::debug::DebugQueueWrapper,
            kernel::debug::DebugQueueWrapper::new(debug_queue)
        );
        kernel::debug::set_debug_queue(debug_queue_wrapper);
    }
}

//! Component for DebugPanicBuffer, the implementation for `debug_panic!()`.
//!
//! This provides one `Component`, `DebugPanicBuffer`, which creates a kernel
//! debug panic buffer (for debug_panic!) using the provided buffer.
//!
//! Usage
//! -----
//! ```rust
//! let buf = static_init!([u8; 1024], [0; 1024]);
//! DebugPanicBufferComponent::new(buf).finalize(());
//! ```

// Author: Guillaume Endignoux <guillaumee@google.com>
// Last modified: 05 Mar 2020

use kernel::component::Component;
use kernel::static_init;

pub struct DebugPanicBufferComponent {
    buffer: &'static mut [u8],
}

impl DebugPanicBufferComponent {
    pub fn new(buffer: &'static mut [u8]) -> Self {
        Self { buffer }
    }
}

impl Component for DebugPanicBufferComponent {
    type StaticInput = ();
    type Output = ();

    unsafe fn finalize(self, _s: Self::StaticInput) -> Self::Output {
        let ring_buffer = static_init!(
            kernel::common::RingBuffer<'static, u8>,
            kernel::common::RingBuffer::new(self.buffer)
        );
        let debug_panic_buffer = static_init!(
            kernel::debug::DebugPanicBuffer,
            kernel::debug::DebugPanicBuffer::new(ring_buffer)
        );
        let debug_panic_buffer_wrapper = static_init!(
            kernel::debug::DebugPanicBufferWrapper,
            kernel::debug::DebugPanicBufferWrapper::new(debug_panic_buffer)
        );
        kernel::debug::set_debug_panic_buffer(debug_panic_buffer_wrapper);
    }
}

//! Component for SeggerRttMemory.
//!
//! This provides one `Component`, `SeggerRttMemoryComponent`, which creates suitable memory for
//! the Segger RTT capsule.
//!
//! Usage
//! -----
//! ```rust
//! let (rtt_memory, up_buffer, down_buffer) = SeggerRttMemoryComponent::new().finalize(());
//! ```

// Author: Guillaume Endignoux <guillaumee@google.com>
// Last modified: 07/02/2020

use capsules::segger_rtt::{SeggerRttMemory, DEFAULT_DOWN_BUFFER_LENGTH, DEFAULT_UP_BUFFER_LENGTH};
use kernel::component::Component;
use kernel::static_init;

pub struct SeggerRttMemoryComponent {}

impl SeggerRttMemoryComponent {
    pub fn new() -> SeggerRttMemoryComponent {
        SeggerRttMemoryComponent {}
    }
}

impl Component for SeggerRttMemoryComponent {
    type StaticInput = ();
    type Output = (
        &'static mut SeggerRttMemory<'static>,
        &'static mut [u8],
        &'static mut [u8],
    );

    unsafe fn finalize(&mut self, _s: Self::StaticInput) -> Self::Output {
        let name = b"Terminal\0";
        let up_buffer_name = name;
        let down_buffer_name = name;
        let up_buffer = static_init!(
            [u8; DEFAULT_UP_BUFFER_LENGTH],
            [0; DEFAULT_UP_BUFFER_LENGTH]
        );
        let down_buffer = static_init!(
            [u8; DEFAULT_DOWN_BUFFER_LENGTH],
            [0; DEFAULT_DOWN_BUFFER_LENGTH]
        );

        let rtt_memory = static_init!(
            SeggerRttMemory,
            SeggerRttMemory::new_raw(
                up_buffer_name,
                up_buffer.as_ptr(),
                up_buffer.len(),
                down_buffer_name,
                down_buffer.as_ptr(),
                down_buffer.len()
            )
        );
        (rtt_memory, up_buffer, down_buffer)
    }
}

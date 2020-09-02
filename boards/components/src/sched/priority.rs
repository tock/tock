//! Component for a priority scheduler.
//!
//! This provides one Component, PriorityComponent.
//!
//! Usage
//! -----
//! ```rust
//! let scheduler =
//!     components::priority::PriorityComponent::new(board_kernel).finalize(());
//! ```

use kernel::component::Component;
use kernel::static_init;
use kernel::PrioritySched;

pub struct PriorityComponent {
    board_kernel: &'static kernel::Kernel,
}

impl PriorityComponent {
    pub fn new(board_kernel: &'static kernel::Kernel) -> PriorityComponent {
        PriorityComponent { board_kernel }
    }
}

impl Component for PriorityComponent {
    type StaticInput = ();
    type Output = &'static mut PrioritySched;

    unsafe fn finalize(self, _static_buffer: Self::StaticInput) -> Self::Output {
        let scheduler = static_init!(PrioritySched, PrioritySched::new(self.board_kernel));
        scheduler
    }
}

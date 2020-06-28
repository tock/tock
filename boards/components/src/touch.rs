//! Components for the Touch Panel.
//!
//! Usage
//! -----
//!
//! ```rust
//! // Just Touch
//! let touch =
//!     components::touch::TouchComponent::new(board_kernel, ts, None, Some(screen))
//!         .finalize(());
//!
//! // With Gesture
//! let touch =
//!     components::touch::TouchComponent::new(board_kernel, ts, Some(ts), Some(screen))
//!         .finalize(());
//! ```
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::static_init;

pub struct TouchComponent {
    board_kernel: &'static kernel::Kernel,
    touch: &'static dyn kernel::hil::touch::Touch,
    gesture: Option<&'static dyn kernel::hil::touch::Gesture>,
    screen: Option<&'static dyn kernel::hil::screen::Screen>,
}

impl TouchComponent {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        touch: &'static dyn kernel::hil::touch::Touch,
        gesture: Option<&'static dyn kernel::hil::touch::Gesture>,
        screen: Option<&'static dyn kernel::hil::screen::Screen>,
    ) -> TouchComponent {
        TouchComponent {
            board_kernel: board_kernel,
            touch: touch,
            gesture: gesture,
            screen: screen,
        }
    }
}

impl Component for TouchComponent {
    type StaticInput = ();
    type Output = &'static capsules::touch::Touch<'static>;

    unsafe fn finalize(self, _static_input: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);
        let grant_touch = self.board_kernel.create_grant(&grant_cap);

        let touch = static_init!(
            capsules::touch::Touch,
            capsules::touch::Touch::new(self.touch, grant_touch, self.screen)
        );

        kernel::hil::touch::Touch::set_client(self.touch, touch);
        if let Some(gesture) = self.gesture {
            kernel::hil::touch::Gesture::set_client(gesture, touch);
        }

        touch
    }
}

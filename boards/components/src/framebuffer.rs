//! Components for the Framebuffer.
//!
//! Usage
//! -----
//! ```rust
//! let framebuffer =
//!     components::framebuffer::FramebufferComponent::new(board_kernel, tft).finalize();
//! ```
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::static_init;

pub struct FramebufferComponent {
    board_kernel: &'static kernel::Kernel,
    screen: &'static dyn kernel::hil::framebuffer::Screen,
}

impl FramebufferComponent {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        screen: &'static dyn kernel::hil::framebuffer::Screen,
    ) -> FramebufferComponent {
        FramebufferComponent {
            board_kernel: board_kernel,
            screen: screen,
        }
    }
}

impl Component for FramebufferComponent {
    type StaticInput = ();
    type Output = &'static capsules::framebuffer::Framebuffer<'static>;

    unsafe fn finalize(self, _static_input: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);
        let grant_framebuffer = self.board_kernel.create_grant(&grant_cap);

        let framebuffer = static_init!(
            capsules::framebuffer::Framebuffer,
            capsules::framebuffer::Framebuffer::new(self.screen, grant_framebuffer)
        );

        self.screen.set_client(Some(framebuffer));

        framebuffer
    }
}

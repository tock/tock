//! Components for the Framebuffer.
//!
//! Buffer Size
//! -----------
//!
//! Displays can receive a large amount of data and having larger transfer buffers
//! optimizes the number of bus writes.
//!
//! As memory is limited on some MCUs, the `components::framebuffer_buffer_size``
//! macro allows users to define the size of the framebuffer buffer.
//!
//! Usage
//! -----
//!
//! // Screen
//! ```rust
//! let framebuffer =
//!     components::framebuffer::FramebufferComponent::new(board_kernel, tft, None)
//!         .finalize(components::framebuffer_buffer_size!(40960));
//! ```
//!
//! // Screen with Setup
//! ```rust
//! let framebuffer =
//!     components::framebuffer::FramebufferComponent::new(board_kernel, tft, Some(tft))
//!         .finalize(components::framebuffer_buffer_size!(40960));
//! ```
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::static_init;

#[macro_export]
macro_rules! framebuffer_buffer_size {
    ($s:literal) => {{
        static mut BUFFER: [u8; $s] = [0; $s];
        (&mut BUFFER)
    }};
}

pub struct FramebufferComponent {
    board_kernel: &'static kernel::Kernel,
    screen: &'static dyn kernel::hil::framebuffer::Screen,
    screen_setup: Option<&'static dyn kernel::hil::framebuffer::ScreenSetup>,
}

impl FramebufferComponent {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        screen: &'static dyn kernel::hil::framebuffer::Screen,
        screen_setup: Option<&'static dyn kernel::hil::framebuffer::ScreenSetup>,
    ) -> FramebufferComponent {
        FramebufferComponent {
            board_kernel: board_kernel,
            screen: screen,
            screen_setup: screen_setup,
        }
    }
}

impl Component for FramebufferComponent {
    type StaticInput = &'static mut [u8];
    type Output = &'static capsules::framebuffer::Framebuffer<'static>;

    unsafe fn finalize(self, static_input: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);
        let grant_framebuffer = self.board_kernel.create_grant(&grant_cap);

        let framebuffer = static_init!(
            capsules::framebuffer::Framebuffer,
            capsules::framebuffer::Framebuffer::new(
                self.screen,
                self.screen_setup,
                static_input,
                grant_framebuffer
            )
        );

        kernel::hil::framebuffer::Screen::set_client(self.screen, Some(framebuffer));
        if let Some(screen) = self.screen_setup {
            kernel::hil::framebuffer::ScreenSetup::set_client(screen, Some(framebuffer));
        }

        framebuffer
    }
}

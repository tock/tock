//! Components for the Screen.
//!
//! Buffer Size
//! -----------
//!
//! Displays can receive a large amount of data and having larger transfer buffers
//! optimizes the number of bus writes.
//!
//! As memory is limited on some MCUs, the `components::screen_buffer_size``
//! macro allows users to define the size of the screen buffer.
//!
//! Usage
//! -----
//!
//! // Screen
//! ```rust
//! let screen =
//!     components::screen::GraphicDisplayComponent::new(board_kernel, tft, None)
//!         .finalize(components::graphic_display_component_static!(40960));
//! ```
//!
//! // Screen with Setup
//! ```rust
//! let screen =
//!     components::screen::GraphicDisplayComponent::new(board_kernel, tft, Some(tft))
//!         .finalize(components::graphic_display_component_static!(40960));
//! ```

use core::mem::MaybeUninit;
use capsules_extra::graphics_display::GraphicDisplay;
use kernel::hil::display;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;

#[macro_export]
macro_rules! graphic_display_component_static {
    ($s:literal $(,)?) => {{
        let buffer = kernel::static_buf!([u8; $s]);
        let screen = kernel::static_buf!(drivers::graphic_display::GraphicDisplay);

        (buffer, screen)
    };};
}

pub struct GraphicDisplayComponent<const SCREEN_BUF_LEN: usize> {
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
    display: &'static dyn display::GraphicDisplay<'static>,
    display_setup: Option<&'static dyn display::FrameBufferSetup<'static>>,
}

impl<const SCREEN_BUF_LEN: usize> GraphicDisplayComponent<SCREEN_BUF_LEN> {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        driver_num: usize,
        display: &'static dyn display::GraphicDisplay<'static>,
        display_setup: Option<&'static dyn display::FrameBufferSetup<'static>>,
    ) -> GraphicDisplayComponent<SCREEN_BUF_LEN> {
        GraphicDisplayComponent {
            board_kernel,
            driver_num,
            display,
            display_setup,
        }
    }
}

impl<const SCREEN_BUF_LEN: usize> Component for GraphicDisplayComponent<SCREEN_BUF_LEN> {
    type StaticInput = (
        &'static mut MaybeUninit<[u8; SCREEN_BUF_LEN]>,
        &'static mut MaybeUninit<GraphicDisplay<'static>>,
    );
    type Output = &'static GraphicDisplay<'static>;

    fn finalize(self, static_input: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);
        let grant_screen = self.board_kernel.create_grant(self.driver_num, &grant_cap);

        let buffer = static_input.0.write([0; SCREEN_BUF_LEN]);

        let display = static_input.1.write(GraphicDisplay::new(
            self.display,
            self.display_setup,
            buffer,
            grant_screen,
        ));

        display::Screen::set_client(self.display, Some(display));
        display::FrameBuffer::set_client(self.display, Some(display));
        if let Some(display_setup) = self.display_setup {
            display::FrameBuffer::set_client(display_setup, Some(display));
        }

        display
    }
}

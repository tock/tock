//! Components for the Text Screen.
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
//! ```rust
//! let text_screen =
//!     components::text_screen::TextScreenComponent::new(board_kernel, tft)
//!         .finalize(components::text_screen_component_static!(40960));
//! ```
//!

use core::mem::MaybeUninit;
use extra_capsules::text_screen::TextScreen;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;

#[macro_export]
macro_rules! text_screen_component_static {
    ($s:literal $(,)?) => {{
        let buffer = kernel::static_buf!([u8; $s]);
        let screen = kernel::static_buf!(extra_capsules::screen::TextScreen);

        (buffer, screen)
    };};
}

pub struct TextScreenComponent<const SCREEN_BUF_LEN: usize> {
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
    text_screen: &'static dyn kernel::hil::text_screen::TextScreen<'static>,
}

impl<const SCREEN_BUF_LEN: usize> TextScreenComponent<SCREEN_BUF_LEN> {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        driver_num: usize,
        text_screen: &'static dyn kernel::hil::text_screen::TextScreen<'static>,
    ) -> TextScreenComponent<SCREEN_BUF_LEN> {
        TextScreenComponent {
            board_kernel: board_kernel,
            driver_num: driver_num,
            text_screen: text_screen,
        }
    }
}

impl<const SCREEN_BUF_LEN: usize> Component for TextScreenComponent<SCREEN_BUF_LEN> {
    type StaticInput = (
        &'static mut MaybeUninit<[u8; SCREEN_BUF_LEN]>,
        &'static mut MaybeUninit<TextScreen<'static>>,
    );
    type Output = &'static TextScreen<'static>;

    fn finalize(self, static_input: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);
        let grant_text_screen = self.board_kernel.create_grant(self.driver_num, &grant_cap);

        let buffer = static_input.0.write([0; SCREEN_BUF_LEN]);

        let text_screen =
            static_input
                .1
                .write(TextScreen::new(self.text_screen, buffer, grant_text_screen));

        kernel::hil::text_screen::TextScreen::set_client(self.text_screen, Some(text_screen));

        text_screen
    }
}

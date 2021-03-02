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
//!         .finalize(components::screen_buffer_size!(40960));
//! ```
//!
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::static_init;

#[macro_export]
macro_rules! text_screen_buffer_size {
    ($s:literal) => {{
        static mut BUFFER: [u8; $s] = [0; $s];
        (&mut BUFFER)
    }};
}

pub struct TextScreenComponent {
    board_kernel: &'static kernel::Kernel,
    driver_num: u32,
    text_screen: &'static dyn kernel::hil::text_screen::TextScreen<'static>,
}

impl TextScreenComponent {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        driver_num: u32,
        text_screen: &'static dyn kernel::hil::text_screen::TextScreen<'static>,
    ) -> TextScreenComponent {
        TextScreenComponent {
            board_kernel: board_kernel,
            driver_num: driver_num,
            text_screen: text_screen,
        }
    }
}

impl Component for TextScreenComponent {
    type StaticInput = &'static mut [u8];
    type Output = &'static capsules::text_screen::TextScreen<'static>;

    unsafe fn finalize(self, static_input: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);
        let grant_text_screen = self.board_kernel.create_grant(self.driver_num, &grant_cap);

        let text_screen = static_init!(
            capsules::text_screen::TextScreen,
            capsules::text_screen::TextScreen::new(
                self.text_screen,
                static_input,
                grant_text_screen
            )
        );

        kernel::hil::text_screen::TextScreen::set_client(self.text_screen, Some(text_screen));

        text_screen
    }
}

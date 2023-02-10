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
//!     components::screen::ScreenComponent::new(board_kernel, tft, None)
//!         .finalize(components::screen_component_static!(40960));
//! ```
//!
//! // Screen with Setup
//! ```rust
//! let screen =
//!     components::screen::ScreenComponent::new(board_kernel, tft, Some(tft))
//!         .finalize(components::screen_component_static!(40960));
//! ```

use core::mem::MaybeUninit;
use extra_capsules::screen::Screen;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;

#[macro_export]
macro_rules! screen_component_static {
    ($s:literal $(,)?) => {{
        let buffer = kernel::static_buf!([u8; $s]);
        let screen = kernel::static_buf!(extra_capsules::screen::Screen);

        (buffer, screen)
    };};
}

pub struct ScreenComponent<const SCREEN_BUF_LEN: usize> {
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
    screen: &'static dyn kernel::hil::screen::Screen,
    screen_setup: Option<&'static dyn kernel::hil::screen::ScreenSetup>,
}

impl<const SCREEN_BUF_LEN: usize> ScreenComponent<SCREEN_BUF_LEN> {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        driver_num: usize,
        screen: &'static dyn kernel::hil::screen::Screen,
        screen_setup: Option<&'static dyn kernel::hil::screen::ScreenSetup>,
    ) -> ScreenComponent<SCREEN_BUF_LEN> {
        ScreenComponent {
            board_kernel: board_kernel,
            driver_num: driver_num,
            screen: screen,
            screen_setup: screen_setup,
        }
    }
}

impl<const SCREEN_BUF_LEN: usize> Component for ScreenComponent<SCREEN_BUF_LEN> {
    type StaticInput = (
        &'static mut MaybeUninit<[u8; SCREEN_BUF_LEN]>,
        &'static mut MaybeUninit<Screen<'static>>,
    );
    type Output = &'static Screen<'static>;

    fn finalize(self, static_input: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);
        let grant_screen = self.board_kernel.create_grant(self.driver_num, &grant_cap);

        let buffer = static_input.0.write([0; SCREEN_BUF_LEN]);

        let screen = static_input.1.write(Screen::new(
            self.screen,
            self.screen_setup,
            buffer,
            grant_screen,
        ));

        kernel::hil::screen::Screen::set_client(self.screen, Some(screen));
        if let Some(screen_setup) = self.screen_setup {
            kernel::hil::screen::ScreenSetup::set_client(screen_setup, Some(screen));
        }

        screen
    }
}

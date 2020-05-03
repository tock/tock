//! Components for the Framebuffer.
//!
//! Usage
//! -----
//! ```rust
//! let lcd = components::hd44780::FramebufferComponent::new(board_kernel, mux_alarm).finalize(
//!     components::hd44780_component_helper!(
//!         stm32f4xx::tim2::Tim2,
//!         // rs pin
//!         stm32f4xx::gpio::PinId::PF13.get_pin().as_ref().unwrap(),
//!         // en pin
//!         stm32f4xx::gpio::PinId::PE11.get_pin().as_ref().unwrap(),
//!         // data 4 pin
//!         stm32f4xx::gpio::PinId::PF14.get_pin().as_ref().unwrap(),
//!         // data 5 pin
//!         stm32f4xx::gpio::PinId::PE13.get_pin().as_ref().unwrap(),
//!         // data 6 pin
//!         stm32f4xx::gpio::PinId::PF15.get_pin().as_ref().unwrap(),
//!         // data 7 pin
//!         stm32f4xx::gpio::PinId::PG14.get_pin().as_ref().unwrap()
//!     )
//! );
//! ```
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::static_init;

pub struct FramebufferComponent {
    board_kernel: &'static kernel::Kernel,
}

impl FramebufferComponent {
    pub fn new(board_kernel: &'static kernel::Kernel) -> FramebufferComponent {
        FramebufferComponent {
            board_kernel: board_kernel,
        }
    }
}

impl Component for FramebufferComponent {
    type StaticInput = &'static dyn kernel::hil::framebuffer::Screen;
    type Output = &'static capsules::framebuffer::Framebuffer<'static>;

    unsafe fn finalize(self, screen: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);
        let grant_framebuffer = self.board_kernel.create_grant(&grant_cap);

        let framebuffer = static_init!(
            capsules::framebuffer::Framebuffer,
            capsules::framebuffer::Framebuffer::new(screen, grant_framebuffer)
        );

        screen.set_client (Some(framebuffer));

        framebuffer
    }
}

//! Component to cause a button press to trigger a kernel panic.
//!
//! This can be useful especially when developping or debugging console
//! capsules.
//!
//! Usage
//! -----
//!
//! ```rust
//! components::panic_button::PanicButtonComponent::new(
//!     &sam4l::gpio::PC[24],
//!     kernel::hil::gpio::ActivationMode::ActiveLow,
//!     kernel::hil::gpio::FloatingState::PullUp
//! )
//! .finalize(panic_button_component_buf!(sam4l::gpio::GPIOPin));
//! ```

use capsules::panic_button::PanicButton;
use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::hil::gpio;
use kernel::static_init_half;

#[macro_export]
macro_rules! panic_button_component_buf {
    ($Pin:ty) => {{
        use capsules::button::PanicButton;
        use core::mem::MaybeUninit;
        static mut BUF: MaybeUninit<PanicButton<'static, $Pin>> = MaybeUninit::uninit();
        &mut BUF
    };};
}

pub struct PanicButtonComponent<'a, IP: gpio::InterruptPin> {
    pin: &'a IP,
    mode: gpio::ActivationMode,
    floating_state: gpio::FloatingState,
}

impl<'a, IP: gpio::InterruptPin> PanicButtonComponent<'a, IP> {
    pub fn new(
        pin: &'a IP,
        mode: gpio::ActivationMode,
        floating_state: gpio::FloatingState,
    ) -> Self {
        Self {
            pin,
            mode,
            floating_state,
        }
    }
}

impl<IP: 'static + gpio::InterruptPin> Component for PanicButtonComponent<'static, IP> {
    type StaticInput = &'static mut MaybeUninit<PanicButton<'static, IP>>;
    type Output = ();

    unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let panic_button = static_init_half!(
            static_buffer,
            PanicButton<'static, IP>,
            PanicButton::new(self.pin, self.mode, self.floating_state)
        );
        self.pin.set_client(panic_button);
    }
}

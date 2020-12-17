//! Component for Buttons.
//!
//! Usage
//! -----
//! ```rust
//! let button = components::button::ButtonComponent::new(
//!     board_kernel,
//!     components::button_component_helper!(
//!         sam4l::gpio::GPIOPin,
//!         (
//!             &sam4l::gpio::PC[24],
//!             kernel::hil::gpio::ActivationMode::ActiveLow,
//!             kernel::hil::gpio::FloatingState::PullUp
//!         )
//!     ),
//! )
//! .finalize(button_component_buf!(sam4l::gpio::GPIOPin));
//! ```
//!
//! Typically, `ActivationMode::ActiveLow` will be associated with `FloatingState::PullUp`
//! whereas `ActivationMode::ActiveHigh` will be paired with `FloatingState::PullDown`.
//! `FloatingState::None` will be used when the board provides external pull-up/pull-down
//! resistors.

use capsules::button::Button;
use core::mem::MaybeUninit;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil::gpio;
use kernel::hil::gpio::InterruptWithValue;
use kernel::static_init_half;

#[macro_export]
macro_rules! button_component_helper {
    ($Pin:ty, $(($P:expr, $M:expr, $F:expr)),+ $(,)?) => {{
        use kernel::static_init;
        use kernel::count_expressions;
        use kernel::hil::gpio::InterruptValueWrapper;
        const NUM_BUTTONS: usize = count_expressions!($($P),+);

        static_init!(
            [(&'static InterruptValueWrapper<'static, $Pin>, kernel::hil::gpio::ActivationMode, kernel::hil::gpio::FloatingState); NUM_BUTTONS],
            [
                $(
                    (static_init!(InterruptValueWrapper<$Pin>, InterruptValueWrapper::new($P))
                    .finalize(),
                    $M,
                    $F
                    ),
                )*
            ]
        )
    };};
}

#[macro_export]
macro_rules! button_component_buf {
    ($Pin:ty $(,)?) => {{
        use capsules::button::Button;
        use core::mem::MaybeUninit;
        static mut BUF: MaybeUninit<Button<'static, $Pin>> = MaybeUninit::uninit();
        &mut BUF
    };};
}

pub struct ButtonComponent<IP: 'static + gpio::InterruptPin<'static>> {
    board_kernel: &'static kernel::Kernel,
    button_pins: &'static [(
        &'static gpio::InterruptValueWrapper<'static, IP>,
        gpio::ActivationMode,
        gpio::FloatingState,
    )],
}

impl<IP: 'static + gpio::InterruptPin<'static>> ButtonComponent<IP> {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        button_pins: &'static [(
            &'static gpio::InterruptValueWrapper<'static, IP>,
            gpio::ActivationMode,
            gpio::FloatingState,
        )],
    ) -> Self {
        Self {
            board_kernel: board_kernel,
            button_pins,
        }
    }
}

impl<IP: 'static + gpio::InterruptPin<'static>> Component for ButtonComponent<IP> {
    type StaticInput = &'static mut MaybeUninit<Button<'static, IP>>;
    type Output = &'static Button<'static, IP>;

    unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);
        let button = static_init_half!(
            static_buffer,
            capsules::button::Button<'static, IP>,
            capsules::button::Button::new(
                self.button_pins,
                self.board_kernel.create_grant(&grant_cap)
            )
        );
        for (pin, _, _) in self.button_pins.iter() {
            pin.set_client(button);
        }

        button
    }
}

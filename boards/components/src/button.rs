//! Component for Buttons.
//!
//! Usage
//! -----
//! ```rust
//! let button = components::button::ButtonComponent::new(board_kernel).finalize(
//!     components::button_component_helper!((
//!         &sam4l::gpio::PC[24],
//!         capsules::button::GpioMode::LowWhenPressed
//!     )),
//! );
//! ```

#![allow(dead_code)] // Components are intended to be conditionally included

use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::static_init;

#[macro_export]
macro_rules! button_component_helper {
    ($(($P:expr, $M:expr)),+ ) => {{
        use kernel::static_init;
        use kernel::count_expressions;
        use kernel::hil::gpio::InterruptValueWrapper;
        const NUM_BUTTONS: usize = count_expressions!($($P),+);

        static_init!(
            [(&'static dyn kernel::hil::gpio::InterruptValuePin, capsules::button::GpioMode); NUM_BUTTONS],
            [
                $(
                    (static_init!(InterruptValueWrapper, InterruptValueWrapper::new($P))
                    .finalize(),
                    $M
                    )
                )*
            ]
        )
    };};
}

pub struct ButtonComponent {
    board_kernel: &'static kernel::Kernel,
}

impl ButtonComponent {
    pub fn new(board_kernel: &'static kernel::Kernel) -> ButtonComponent {
        ButtonComponent {
            board_kernel: board_kernel,
        }
    }
}

impl Component for ButtonComponent {
    type StaticInput = &'static [(
        &'static dyn kernel::hil::gpio::InterruptValuePin,
        capsules::button::GpioMode,
    )];
    type Output = &'static capsules::button::Button<'static>;

    unsafe fn finalize(&mut self, button_pins: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);
        let button = static_init!(
            capsules::button::Button<'static>,
            capsules::button::Button::new(button_pins, self.board_kernel.create_grant(&grant_cap))
        );
        for (pin, _) in button_pins.iter() {
            pin.set_client(button);
        }

        button
    }
}

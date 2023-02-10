//! Component for Buttons.
//!
//! Usage
//! -----
//!
//! The `button_component_helper!` macro takes 'static references to GPIO pins.
//! When GPIO instances are owned values, the `button_component_helper_owned!`
//! can be used, indicating that the passed values are owned values. This macro
//! will perform static allocation of the passed in GPIO pins internally.
//!
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
//! .finalize(button_component_static!(sam4l::gpio::GPIOPin));
//! ```
//!
//! Typically, `ActivationMode::ActiveLow` will be associated with
//! `FloatingState::PullUp` whereas `ActivationMode::ActiveHigh` will be paired
//! with `FloatingState::PullDown`. `FloatingState::None` will be used when the
//! board provides external pull-up/pull-down resistors.

use core::mem::MaybeUninit;
use core_capsules::button::Button;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil::gpio;
use kernel::hil::gpio::InterruptWithValue;

#[macro_export]
macro_rules! button_component_helper_owned {
    ($Pin:ty, $(($P:expr, $M:expr, $F:expr)),+ $(,)?) => {
        $crate::button_component_helper!(
            $Pin,
            $((
                static_init!($Pin, $P),
                $M,
                $F
            ),)*
        )
    };
}

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
macro_rules! button_component_static {
    ($Pin:ty $(,)?) => {{
        kernel::static_buf!(core_capsules::button::Button<'static, $Pin>)
    };};
}

pub struct ButtonComponent<IP: 'static + gpio::InterruptPin<'static>> {
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
    button_pins: &'static [(
        &'static gpio::InterruptValueWrapper<'static, IP>,
        gpio::ActivationMode,
        gpio::FloatingState,
    )],
}

impl<IP: 'static + gpio::InterruptPin<'static>> ButtonComponent<IP> {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        driver_num: usize,
        button_pins: &'static [(
            &'static gpio::InterruptValueWrapper<'static, IP>,
            gpio::ActivationMode,
            gpio::FloatingState,
        )],
    ) -> Self {
        Self {
            board_kernel: board_kernel,
            driver_num,
            button_pins,
        }
    }
}

impl<IP: 'static + gpio::InterruptPin<'static>> Component for ButtonComponent<IP> {
    type StaticInput = &'static mut MaybeUninit<Button<'static, IP>>;
    type Output = &'static Button<'static, IP>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);
        let button = static_buffer.write(core_capsules::button::Button::new(
            self.button_pins,
            self.board_kernel.create_grant(self.driver_num, &grant_cap),
        ));
        for (pin, _, _) in self.button_pins.iter() {
            pin.set_client(button);
        }

        button
    }
}

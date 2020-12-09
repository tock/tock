//! Components for collections of LEDs.
//!
//! Usage
//! -----
//! ```rust
//! let led = components::led::LedsComponent::new(components::led_component_helper!(
//!     kernel::hil::led::LedLow<'static, sam4l::gpio::GPIOPin>,
//!     LedLow::new(&sam4l::gpio::PORT[LED_RED_PIN]),
//!     LedLow::new(&sam4l::gpio::PORT[LED_GREEN_PIN]),
//!     LedLow::new(&sam4l::gpio::PORT[LED_BLUE_PIN]),
//! ))
//! .finalize(led_component_buf!(kernel::hil::led::LedLow<'static, sam4l::gpio::GPIOPin>));
//! ```

use capsules::led::LedDriver;
use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::hil::led::Led;
use kernel::static_init_half;

#[macro_export]
macro_rules! led_component_helper {
    ($Led:ty, $($L:expr),+ $(,)?) => {{
        use kernel::count_expressions;
        use kernel::static_init;
        const NUM_LEDS: usize = count_expressions!($($L),+);

	static_init!(
	    [&'static mut $Led; NUM_LEDS],
	    [
		$(
		    static_init!(
			$Led,
			$L
		    )
		),+
	    ]
	)
    };};
}

#[macro_export]
macro_rules! led_component_buf {
    ($Led:ty $(,)?) => {{
        use capsules::led::LedDriver;
        use core::mem::MaybeUninit;
        static mut BUF: MaybeUninit<LedDriver<'static, $Led>> = MaybeUninit::uninit();
        &mut BUF
    };};
}

pub struct LedsComponent<L: 'static + Led> {
    leds: &'static mut [&'static mut L],
}

impl<L: 'static + Led> LedsComponent<L> {
    pub fn new(leds: &'static mut [&'static mut L]) -> Self {
        Self { leds }
    }
}

impl<L: 'static + Led> Component for LedsComponent<L> {
    type StaticInput = &'static mut MaybeUninit<LedDriver<'static, L>>;
    type Output = &'static LedDriver<'static, L>;

    unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        static_init_half!(
            static_buffer,
            LedDriver<'static, L>,
            LedDriver::new(self.leds)
        )
    }
}

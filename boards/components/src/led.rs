//! Components for collections of LEDs.
//!
//! Usage
//! -----
//! ```rust
//! let led = components::led::LedsComponent::new().finalize(components::led_component_static!(
//!     kernel::hil::led::LedLow<'static, sam4l::gpio::GPIOPin>,
//!     LedLow::new(&sam4l::gpio::PORT[LED_RED_PIN]),
//!     LedLow::new(&sam4l::gpio::PORT[LED_GREEN_PIN]),
//!     LedLow::new(&sam4l::gpio::PORT[LED_BLUE_PIN]),
//! ));
//! ```

use core::marker::PhantomData;
use core::mem::MaybeUninit;
use core_capsules::led::LedDriver;
use kernel::component::Component;
use kernel::hil::led::Led;

#[macro_export]
macro_rules! led_component_static {
    ($Led:ty, $($L:expr),+ $(,)?) => {{
        use kernel::count_expressions;
        use kernel::static_init;
        const NUM_LEDS: usize = count_expressions!($($L),+);
        let arr = static_init!(
            [&'static $Led; NUM_LEDS],
            [
                $(
                    static_init!(
                        $Led,
                        $L
                    )
                ),+
            ]
        );

        let led = kernel::static_buf!( core_capsules::led::LedDriver<'static, $Led, NUM_LEDS>);
        (led, arr)
    };};
}

pub struct LedsComponent<L: 'static + Led, const NUM_LEDS: usize> {
    _phantom: PhantomData<L>,
}

impl<L: 'static + Led, const NUM_LEDS: usize> LedsComponent<L, NUM_LEDS> {
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<L: 'static + Led, const NUM_LEDS: usize> Component for LedsComponent<L, NUM_LEDS> {
    type StaticInput = (
        &'static mut MaybeUninit<LedDriver<'static, L, NUM_LEDS>>,
        &'static mut [&'static L; NUM_LEDS],
    );
    type Output = &'static LedDriver<'static, L, NUM_LEDS>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        static_buffer.0.write(LedDriver::new(static_buffer.1))
    }
}

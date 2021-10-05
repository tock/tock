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
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::hil::led::Led;
use kernel::static_init_half;

#[macro_export]
macro_rules! led_component_helper {
    ($Led:ty, $($L:expr),+ $(,)?) => {{
        use capsules::led::LedDriver;
        use core::mem::MaybeUninit;
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

        static mut BUF: MaybeUninit<LedDriver<'static, $Led, NUM_LEDS>> = MaybeUninit::uninit();
        (&mut BUF, arr)
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

    unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        static_init_half!(
            static_buffer.0,
            LedDriver<'static, L, NUM_LEDS>,
            LedDriver::new(static_buffer.1)
        )
    }
}

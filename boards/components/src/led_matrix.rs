//! Component for matrices of LEDs.
//!
//! Usage
//! -----
//! ```rust
//! let led_matrix = components::led_matrix::LedMatrixComponent::new(
//!     mux_alarm,
//!     components::led_line_component_static!(
//!         nrf52833::gpio::GPIOPin,
//!         &nrf52833_peripherals.gpio_port[LED_MATRIX_COLS[0]],
//!         &nrf52833_peripherals.gpio_port[LED_MATRIX_COLS[1]],
//!         &nrf52833_peripherals.gpio_port[LED_MATRIX_COLS[2]],
//!         &nrf52833_peripherals.gpio_port[LED_MATRIX_COLS[3]],
//!         &nrf52833_peripherals.gpio_port[LED_MATRIX_COLS[4]],
//!     ),
//!     components::led_line_component_static!(
//!         nrf52833::gpio::GPIOPin,
//!         &nrf52833_peripherals.gpio_port[LED_MATRIX_ROWS[0]],
//!         &nrf52833_peripherals.gpio_port[LED_MATRIX_ROWS[1]],
//!         &nrf52833_peripherals.gpio_port[LED_MATRIX_ROWS[2]],
//!         &nrf52833_peripherals.gpio_port[LED_MATRIX_ROWS[3]],
//!         &nrf52833_peripherals.gpio_port[LED_MATRIX_ROWS[4]],
//!     ),
//!     kernel::hil::gpio::ActivationMode::ActiveLow,
//!     kernel::hil::gpio::ActivationMode::ActiveHigh,
//!     60,
//! )
//! .finalize(components::led_matrix_component_static!(
//!     nrf52833::gpio::GPIOPin,
//!     nrf52::rtc::Rtc<'static>,
//!     5,
//!     5
//! ));
//! ```
//!
//! Single LED usage
//! ----------------
//!
//! ```rust
//! let led = components::led_matrix_led!(
//!     nrf52::gpio::GPIOPin<'static>,
//!     core_capsules::virtual_alarm::VirtualMuxAlarm<'static, nrf52::rtc::Rtc<'static>>,
//!     led,
//!     1,
//!     2
//! );
//! ```
//!
//! Array LED usage
//! ----------------
//!
//! ```rust
//! let leds = components::led_matrix_leds!(
//!     nrf52::gpio::GPIOPin<'static>,
//!     core_capsules::virtual_alarm::VirtualMuxAlarm<'static, nrf52::rtc::Rtc<'static>>,
//!     led,
//!     (0, 0),
//!     (1, 0),
//!     (2, 0),
//!     (3, 0),
//!     (4, 0),
//!     (0, 1)
//!     // ...
//! );
//! ```
//!

use core::mem::MaybeUninit;
use core_capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use extra_capsules::led_matrix::LedMatrixDriver;
use kernel::component::Component;
use kernel::hil::gpio::{ActivationMode, Pin};
use kernel::hil::time::Alarm;

#[macro_export]
macro_rules! led_matrix_component_static {
    ($Pin:ty, $A: ty, $num_cols: literal, $num_rows: literal $(,)?) => {{
        let buffer = kernel::static_buf!([u8; $num_cols * $num_rows / 8 + 1]);
        let alarm = kernel::static_buf!(core_capsules::virtual_alarm::VirtualMuxAlarm<'static, $A>);
        let led = kernel::static_buf!(
            extra_capsules::led_matrix::LedMatrixDriver<
                'static,
                $Pin,
                core_capsules::virtual_alarm::VirtualMuxAlarm<'static, $A>,
            >
        );

        (alarm, led, buffer)
    };};
}

#[macro_export]
macro_rules! led_line_component_static {
    ($Pin:ty, $($L:expr),+ $(,)?) => {{
        use kernel::count_expressions;
        use kernel::static_init;
        const NUM_LEDS: usize = count_expressions!($($L),+);

        static_init!(
            [&'static $Pin; NUM_LEDS],
            [
                $(
                    static_init!(
                        &'static $Pin,
                        $L
                    )
                ),+
            ]
        )
    };};
}

#[macro_export]
macro_rules! led_matrix_led {
    ($Pin:ty, $A: ty, $led_matrix: expr, $col: expr, $row: expr) => {{
        use extra_capsules::led_matrix::LedMatrixLed;
        static_init!(
            LedMatrixLed<'static, $Pin, $A>,
            LedMatrixLed::new($led_matrix, $col, $row)
        )
    };};
}

#[macro_export]
macro_rules! led_matrix_leds {
    ($Pin:ty, $A: ty, $led_matrix: expr, $(($col: expr, $row: expr)),+) => {{
        use extra_capsules::led_matrix::LedMatrixLed;
        use kernel::count_expressions;

        const NUM_LEDS: usize = count_expressions!($(($col, $row)),+);
        let leds = static_init!(
            [&LedMatrixLed<'static, $Pin, $A>; NUM_LEDS],
            [$(
                $crate::led_matrix_led! ($Pin, $A, $led_matrix, $col, $row)
            ),+]
        );
        leds
    };};
}

pub struct LedMatrixComponent<
    L: 'static + Pin,
    A: 'static + Alarm<'static>,
    const NUM_COLS: usize,
    const NUM_ROWS: usize,
    const NUM_LED_BITS: usize,
> {
    alarm_mux: &'static MuxAlarm<'static, A>,
    col: &'static [&'static L; NUM_COLS],
    row: &'static [&'static L; NUM_ROWS],
    col_active: ActivationMode,
    row_active: ActivationMode,
    refresh_rate: usize,
}

impl<
        L: 'static + Pin,
        A: 'static + Alarm<'static>,
        const NUM_COLS: usize,
        const NUM_ROWS: usize,
        const NUM_LED_BITS: usize,
    > LedMatrixComponent<L, A, NUM_COLS, NUM_ROWS, NUM_LED_BITS>
{
    pub fn new(
        alarm_mux: &'static MuxAlarm<'static, A>,
        col: &'static [&'static L; NUM_COLS],
        row: &'static [&'static L; NUM_ROWS],
        col_active: ActivationMode,
        row_active: ActivationMode,
        refresh_rate: usize,
    ) -> Self {
        Self {
            alarm_mux,
            col,
            row,
            col_active,
            row_active,
            refresh_rate,
        }
    }
}

impl<
        L: 'static + Pin,
        A: 'static + Alarm<'static>,
        const NUM_COLS: usize,
        const NUM_ROWS: usize,
        const NUM_LED_BITS: usize,
    > Component for LedMatrixComponent<L, A, NUM_COLS, NUM_ROWS, NUM_LED_BITS>
{
    type StaticInput = (
        &'static mut MaybeUninit<VirtualMuxAlarm<'static, A>>,
        &'static mut MaybeUninit<LedMatrixDriver<'static, L, VirtualMuxAlarm<'static, A>>>,
        &'static mut MaybeUninit<[u8; NUM_LED_BITS]>,
    );
    type Output = &'static LedMatrixDriver<'static, L, VirtualMuxAlarm<'static, A>>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let led_alarm = static_buffer.0.write(VirtualMuxAlarm::new(self.alarm_mux));
        led_alarm.setup();

        let buffer = static_buffer.2.write([0; NUM_LED_BITS]);

        let led_matrix = static_buffer.1.write(LedMatrixDriver::new(
            self.col,
            self.row,
            buffer,
            led_alarm,
            self.col_active,
            self.row_active,
            self.refresh_rate,
        ));

        led_alarm.set_alarm_client(led_matrix);

        led_matrix.init();

        led_matrix
    }
}

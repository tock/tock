//! Components for martices of LEDs.
//!
//! Usage
//! -----
//! ```rust
//! let led = components::led_matrix_component_helper!(
//!     nrf52833::gpio::GPIOPin,
//!     nrf52::rtc::Rtc<'static>,
//!     mux_alarm,
//!     @fps => 60,
//!     @cols => kernel::hil::gpio::ActivationMode::ActiveLow,
//!         &base_peripherals.gpio_port[LED_MATRIX_COLS[0]],
//!         &base_peripherals.gpio_port[LED_MATRIX_COLS[1]],
//!         &base_peripherals.gpio_port[LED_MATRIX_COLS[2]],
//!         &base_peripherals.gpio_port[LED_MATRIX_COLS[3]],
//!        &base_peripherals.gpio_port[LED_MATRIX_COLS[4]],
//!     @rows => kernel::hil::gpio::ActivationMode::ActiveHigh,
//!         &base_peripherals.gpio_port[LED_MATRIX_ROWS[0]],
//!         &base_peripherals.gpio_port[LED_MATRIX_ROWS[1]],
//!         &base_peripherals.gpio_port[LED_MATRIX_ROWS[2]],
//!         &base_peripherals.gpio_port[LED_MATRIX_ROWS[3]],
//!         &base_peripherals.gpio_port[LED_MATRIX_ROWS[4]]
//!
//! )
//! .finalize(components::led_matrix_component_buf!(
//!     nrf52833::gpio::GPIOPin,
//!     nrf52::rtc::Rtc<'static>
//! ));
//! ```

use capsules::led_matrix::LedMatrixDriver;
use capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::hil::gpio::{ActivationMode, Pin};
use kernel::hil::time::Alarm;
use kernel::static_init_half;

#[macro_export]
macro_rules! led_matrix_component_helper {
    ($Pin:ty, $A: ty, $alarm_mux: expr, @fps => $refresh_rate: expr, @cols => $col_active:expr, $($C:expr),+, @rows => $row_active:expr, $($R:expr),+ $(,)?) => {{
        use kernel::count_expressions;

        const NUM_COLS: usize = count_expressions!($($C),+);
        const NUM_ROWS: usize = count_expressions!($($R),+);
        static mut BUFFER: [u8; NUM_COLS*NUM_ROWS / 8 + 1] = [0; NUM_COLS*NUM_ROWS / 8 + 1];
        components::led_matrix::LedMatrixComponent::new ($alarm_mux, $crate::led_line_component_helper!($Pin, $($C,)+), $crate::led_line_component_helper!($Pin, $($R,)+), $col_active, $row_active, $refresh_rate, &mut BUFFER)
    };};
}

#[macro_export]
macro_rules! led_line_component_helper {
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
macro_rules! led_matrix_component_buf {
    ($Pin:ty, $A: ty $(,)?) => {{
        use capsules::led_matrix::LedMatrixDriver;
        use capsules::virtual_alarm::VirtualMuxAlarm;
        use core::mem::MaybeUninit;

        static mut alarm: MaybeUninit<VirtualMuxAlarm<'static, $A>> = MaybeUninit::uninit();
        static mut led: MaybeUninit<LedMatrixDriver<'static, $Pin, VirtualMuxAlarm<'static, $A>>> =
            MaybeUninit::uninit();
        (&mut alarm, &mut led)
    };};
}

pub struct LedMatrixComponent<L: 'static + Pin, A: 'static + Alarm<'static>> {
    col: &'static [&'static L],
    row: &'static [&'static L],
    col_active: ActivationMode,
    row_active: ActivationMode,
    refresh_rate: usize,
    buffer: &'static mut [u8],
    alarm_mux: &'static MuxAlarm<'static, A>,
}

impl<L: 'static + Pin, A: 'static + Alarm<'static>> LedMatrixComponent<L, A> {
    pub fn new(
        alarm_mux: &'static MuxAlarm<'static, A>,
        col: &'static [&'static L],
        row: &'static [&'static L],
        col_active: ActivationMode,
        row_active: ActivationMode,
        refresh_rate: usize,
        buffer: &'static mut [u8],
    ) -> Self {
        Self {
            alarm_mux,
            col,
            row,
            col_active,
            row_active,
            refresh_rate,
            buffer,
        }
    }
}

impl<L: 'static + Pin, A: 'static + Alarm<'static>> Component for LedMatrixComponent<L, A> {
    type StaticInput = (
        &'static mut MaybeUninit<VirtualMuxAlarm<'static, A>>,
        &'static mut MaybeUninit<LedMatrixDriver<'static, L, VirtualMuxAlarm<'static, A>>>,
    );
    type Output = &'static LedMatrixDriver<'static, L, VirtualMuxAlarm<'static, A>>;

    unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let led_alarm = static_init_half!(
            static_buffer.0,
            VirtualMuxAlarm<'static, A>,
            VirtualMuxAlarm::new(self.alarm_mux)
        );

        let led_matrix = static_init_half!(
            static_buffer.1,
            LedMatrixDriver<'static, L, VirtualMuxAlarm<'static, A>>,
            LedMatrixDriver::new(
                self.col,
                self.row,
                self.buffer,
                led_alarm,
                self.col_active,
                self.row_active,
                self.refresh_rate
            )
        );

        led_alarm.set_alarm_client(led_matrix);

        led_matrix.init();

        led_matrix
    }
}

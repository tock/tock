//! A benchmark capsule for testing interrupt latency. Toggles an external pin when another bit is
//! triggered (e.g. an LED and button).
//!
//! To run, make sure `setup_bench` is called from the board setup in `main.rs`. Hook up to a logic
//! analyzer or oscilloscope to probe both the button and LED pins.  Trigger on the button
//! toggling, then start. When you're ready, press the button slowly a few times (the LED will
//! toggle on both edges to hold the button down to get nice spacing). The gap between the button
//! edge and the LED edge is the latency between the external event and `Client#fired` being
//! called.

use kernel::hil;
use sam4l::gpio;

static GPIO_CLIENT: DummyGPIO = DummyGPIO;

struct DummyGPIO;

impl hil::gpio::Client for DummyGPIO {
    fn fired(&self, _pin: usize) {
        let led: &hil::gpio::Pin = unsafe { &gpio::PC[10] };
        led.toggle();
    }
}

pub fn setup_bench() {
    let led: &hil::gpio::Pin = unsafe { &gpio::PC[10] };
    led.make_output();

    let int_pin: &'static mut hil::gpio::Pin = unsafe {
        gpio::PC[24].set_client(&GPIO_CLIENT);
        &mut gpio::PC[24]
    };

    int_pin.make_input();
    int_pin.enable_interrupt(0, hil::gpio::InterruptMode::EitherEdge);
}

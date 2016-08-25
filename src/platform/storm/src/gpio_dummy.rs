//! A dummy GPIO client to test the GPIO interrupt implementation

use hil;
use sam4l::gpio;

static GPIO_CLIENT: DummyGPIO = DummyGPIO;

struct DummyGPIO;

impl hil::gpio::Client for DummyGPIO {
    fn fired(&self, _pin: usize) {
        let led: &hil::gpio::GPIOPin = unsafe { &gpio::PC[10] };
        led.toggle();
    }
}

pub fn gpio_dummy_test() {
    let led: &hil::gpio::GPIOPin = unsafe { &gpio::PC[10] };
    led.enable_output();

    let int_pin: &'static mut hil::gpio::GPIOPin = unsafe {
        gpio::PA[13].set_client(&GPIO_CLIENT);
        &mut gpio::PA[13]
    };

    int_pin.enable_input(hil::gpio::InputMode::PullDown);
    int_pin.enable_interrupt(0, hil::gpio::InterruptMode::Change);
}

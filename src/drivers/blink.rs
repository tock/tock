use hil::timer::{Timer, TimerReceiver};
use hil::gpio::GPIOPin;

pub struct Blink {
    timer: &'static mut Timer,
    led: &'static mut GPIOPin
}

impl Blink {
    pub fn new(timer: &'static mut Timer,
               led: &'static mut GPIOPin) -> Blink {
        Blink {
            timer: timer,
            led: led
        }
    }

    pub fn initialize(&mut self) {
        self.led.enable_output();

        let now = self.timer.now();
        self.timer.set_alarm(now + 32768);
    }
}

impl TimerReceiver for Blink {
    fn alarm_fired(&mut self) {
        use hil::gpio::GPIOPin;
        use hil::timer::Timer;

        let now = self.timer.now();
        self.led.toggle();
        self.timer.set_alarm(now + 32768);
    }
}


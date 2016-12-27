use core::cell::Cell;
use kernel::hil::gpio::Pin;
use kernel::hil::spi;
use virtual_spi::SPIMasterDevice;
use kernel::returncode::ReturnCode;

pub struct RF233 <'a, S: spi::SPIMasterDevice + 'a> {
    spi: &'a S,
    radio_on: Cell<bool>,
    transmitting: Cell<bool>,
    reset_pin: &'a Pin,
    sleep_pin: &'a Pin,
}

impl<'a, S: spi::SPIMasterDevice + 'a> RF233 <'a, S> {
    pub fn new(spi: &'a S,
               reset: &'a Pin,
               sleep: &'a Pin) -> RF233<'a, S> {
        RF233 {
            spi: spi,
            reset_pin: reset,
            sleep_pin: sleep,
            radio_on: Cell::new(false),
            transmitting: Cell::new(false),

        }
    }

    pub fn initialize(&self) -> ReturnCode {
        self.spi.configure(spi::ClockPolarity::IdleLow,
                           spi::ClockPhase::SampleLeading,
                           40000);
        self.reset()
    }

    pub fn reset(&self) -> ReturnCode {
        self.reset_pin.make_output();
        self.sleep_pin.make_output();
        self.reset_pin.clear();
        // delay 1 ms
        self.reset_pin.set();
        self.sleep_pin.clear();
        self.transmitting.set(false);
        self.radio_on.set(true);
        ReturnCode::SUCCESS
    }

}

use hil;

pub struct LedHigh {
  pin: &'static mut hil::gpio::GPIOPin
}

pub struct LedLow {
  pin: &'static mut hil::gpio::GPIOPin
}

impl LedHigh {
  pub fn new(p: &'static mut hil::gpio::GPIOPin) -> LedHigh {
    LedHigh {
      pin: p
    }
  }
}

impl LedLow {
  pub fn new(p: &'static mut hil::gpio::GPIOPin) -> LedLow {
    LedLow {
      pin: p
    }
  }
}

impl hil::led::Led for LedHigh {
  fn on(&mut self) {
    self.pin.set();
  }
  fn off(&mut self) {
    self.pin.clear();
  }
  fn toggle(&mut self) {
    self.pin.toggle();
  }
  fn read(&self) -> bool {
    self.pin.read()
  }
}

impl hil::led::Led for LedLow {
  fn on(&mut self) {
    self.pin.clear();
  }
  fn off(&mut self) {
    self.pin.set();
  }
  fn toggle(&mut self) {
    self.pin.toggle();
  }
  fn read(&self) -> bool {
    !self.pin.read()
  }
}


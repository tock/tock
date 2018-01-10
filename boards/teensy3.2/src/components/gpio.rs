use capsules;
use mk20;
use components::{Component, ComponentWithDependency};

type PinHandle = &'static mk20::gpio::Gpio<'static>;

pub struct GpioComponent {
    pins: Option<&'static [PinHandle]>
}

impl GpioComponent {
    pub fn new() -> Self {
        GpioComponent {
            pins: None
        }
    }
}

impl Component for GpioComponent {
    type Output = &'static capsules::gpio::GPIO<'static, mk20::gpio::Gpio<'static>>;

    unsafe fn finalize(&mut self) -> Option<Self::Output> {
        if self.pins.is_none() {
            return None;
        }

        let gpio = static_init!(
                capsules::gpio::GPIO<'static, mk20::gpio::Gpio<'static>>,
                capsules::gpio::GPIO::new(self.pins.unwrap())
            );

        for pin in self.pins.unwrap().iter() {
            pin.set_client(gpio);
        }

        Some(gpio)
    }
}

impl ComponentWithDependency<&'static [PinHandle]> for GpioComponent {
    fn dependency(&mut self, pins: &'static [PinHandle]) -> &mut Self {
        self.pins = Some(pins);

        self
    }
}


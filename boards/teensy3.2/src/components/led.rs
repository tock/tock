use mk20;
use capsules;
use components::{Component, ComponentWithDependency};

type PinHandle = &'static mk20::gpio::Gpio<'static>;

pub struct LedComponent {
    leds: Option<&'static [(&'static mk20::gpio::Gpio<'static>, capsules::led::ActivationMode)]>
}

impl LedComponent {
    pub fn new() -> Self {
        LedComponent {
            leds: None
        }
    }
}

impl Component for LedComponent {
    type Output = &'static capsules::led::LED<'static, mk20::gpio::Gpio<'static>>;

    unsafe fn finalize(&mut self) -> Option<Self::Output> {
        if self.leds.is_none() {
            return None;
        }

        let leds = static_init!(
                capsules::led::LED<'static, mk20::gpio::Gpio<'static>>,
                capsules::led::LED::new(self.leds.unwrap())
            );

        Some(leds)
    }
}

impl ComponentWithDependency<&'static [(PinHandle, capsules::led::ActivationMode)]> for LedComponent {
    fn dependency(&mut self, leds: &'static [(PinHandle, capsules::led::ActivationMode)]) -> &mut Self {
        self.leds = Some(leds);

        self
    }
}


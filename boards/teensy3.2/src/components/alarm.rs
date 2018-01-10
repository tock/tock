use mk20;
use kernel;
use components::Component;
use capsules::alarm::AlarmDriver;

pub struct AlarmComponent;

impl AlarmComponent {
    pub fn new() -> Self {
        AlarmComponent {}
    }
}

impl Component for AlarmComponent {
    type Output = &'static AlarmDriver<'static, mk20::pit::Pit<'static>>;

    unsafe fn finalize(&mut self) -> Option<Self::Output> {
        mk20::pit::PIT.init();

        let alarm = static_init!(
                AlarmDriver<'static, mk20::pit::Pit>,
                AlarmDriver::new(&mk20::pit::PIT,
                                 kernel::Grant::create())
            );
        mk20::pit::PIT.set_client(alarm);
        Some(alarm)
    }
}

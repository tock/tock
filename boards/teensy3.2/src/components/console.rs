use capsules;
use mk20;
use kernel;
use kernel::hil::uart::UART;
use components::Component;

pub struct UartConsoleComponent;

impl UartConsoleComponent {
    pub fn new() -> Self {
        UartConsoleComponent {}
    }
}

impl Component for UartConsoleComponent {
    type Output = &'static capsules::console::Console<'static, mk20::uart::Uart>;

    unsafe fn finalize(&mut self) -> Option<Self::Output> {
        let console = static_init!(
                capsules::console::Console<mk20::uart::Uart>,
                capsules::console::Console::new(&mk20::uart::UART0,
                                                115200,
                                                &mut capsules::console::WRITE_BUF,
                                                kernel::Grant::create())
            );
        mk20::uart::UART0.set_client(console);
        console.initialize();

        let kc = static_init!(
                capsules::console::App,
                capsules::console::App::default()
            );
        kernel::debug::assign_console_driver(Some(console), kc);

        Some(console)
    }
}

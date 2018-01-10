use mk20;
use kernel;
use xconsole;
use kernel::hil::uart::UART;
use components::Component;

pub struct XConsoleComponent;

impl XConsoleComponent {
    pub fn new() -> Self {
        XConsoleComponent {}
    }
}

impl Component for XConsoleComponent {
    type Output = &'static xconsole::XConsole<'static, mk20::uart::Uart>;

    unsafe fn finalize(&mut self) -> Option<Self::Output> {
        let xconsole = static_init!(
                xconsole::XConsole<mk20::uart::Uart>,
                xconsole::XConsole::new(&mk20::uart::UART0,
                                        115200,
                                        &mut xconsole::WRITE_BUF,
                                        &mut xconsole::READ_BUF,
                                        kernel::Grant::create())
            );
        mk20::uart::UART0.set_client(xconsole);
        xconsole.initialize();

        let kc = static_init!(
                xconsole::App,
                xconsole::App::default()
            );
        kernel::debug::assign_console_driver(Some(xconsole), kc);

        mk20::uart::UART0.enable_rx();
        mk20::uart::UART0.enable_rx_interrupts();

        Some(xconsole)
    }
}

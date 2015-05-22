#![feature(core,no_std)]
#![no_main]
#![no_std]

extern crate core;
extern crate support;
extern crate hil;
extern crate platform;

use core::prelude::*;

mod shared;

mod vtimer {
    use platform;
    use hil;

    pub struct VTimer {
        ast: &'static mut platform::ast::Ast,
        led: platform::gpio::GPIOPin
    }

    impl VTimer {
        pub fn new(ast: &'static mut platform::ast::Ast,
                   led: platform::gpio::GPIOPin) -> VTimer {
            VTimer {
                ast: ast,
                led: led
            }
        }

        pub fn initialize(&mut self) {
            use hil::gpio::GPIOPin;
            use hil::timer::Timer;

            self.led.enable_output();
            self.ast.setup();

            let now = self.ast.now();
            self.ast.set_alarm(now + 32768);
        }
    }

    impl hil::timer::TimerReceiver for VTimer {
        fn alarm_fired(&mut self) {
            use hil::gpio::GPIOPin;
            use hil::timer::Timer;

            self.led.toggle();
            let now = self.ast.now();
            self.ast.set_alarm(now + 32768);
        }
    }
}

mod conf {
    use core::prelude::*;
    use hil::Controller;
    use platform;
    use shared::Shared;
    
    use super::vtimer;

    pub static mut AST : Option<Shared<platform::ast::Ast>> = None;
    pub static mut VTIMER : Option<Shared<vtimer::VTimer>> = None;

    pub fn init() {
        use platform::gpio;
        use hil::gpio::GPIOPin;

        let mut led : gpio::GPIOPin = Controller::new(gpio::Location::GPIOPin74);
        led.configure(None);

        unsafe {
            AST = Some(Shared::new(Controller::new(())));
            VTIMER = Some(Shared::new(
                    vtimer::VTimer::new(
                        AST.as_ref().unwrap().borrow_mut(), led)
                    ));
            let ast = AST.as_ref().unwrap().borrow_mut();
            ast.configure(VTIMER.as_ref().unwrap().borrow_mut());

            VTIMER.as_ref().unwrap().borrow_mut().initialize();
        }
    }
}

#[no_mangle]
pub extern fn main() {
    conf::init();
    loop {
        support::wfi();
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern fn AST_ALARM_Handler() {
    use core::intrinsics;
    intrinsics::volatile_load(&conf::AST).as_ref().unwrap().borrow_mut().
        handle_interrupt();
}


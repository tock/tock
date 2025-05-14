use core::{cell::Cell, future::Future};

use embedded_hal_async::delay::DelayNs;
use kernel::{
    debug,
    hil::time::Alarm,
    process::Error,
    syscall::{CommandReturn, SyscallDriver},
    ErrorCode, ProcessId,
};

use crate::{
    delay::Delay,
    executor::{AsyncDriver, Runner},
};

pub struct HelloPrintDriver<A: Alarm<'static> + 'static> {
    runner: Cell<Option<&'static dyn Runner>>,
    delay: &'static Delay<'static, A>,
}

impl<A: Alarm<'static> + 'static> HelloPrintDriver<A> {
    pub fn new(delay: &'static Delay<'static, A>) -> HelloPrintDriver<A> {
        HelloPrintDriver {
            runner: Cell::new(None),
            delay,
        }
    }

    pub fn set_runner(&self, runner: Option<&'static dyn Runner>) {
        self.runner.replace(runner);
    }

    pub fn execute(&self) -> Result<(), ErrorCode> {
        self.runner.get().unwrap().execute()
    }
}

impl<A: Alarm<'static> + 'static> SyscallDriver for HelloPrintDriver<A> {
    fn command(
        &self,
        command_num: usize,
        _r2: usize,
        _r3: usize,
        _process_id: ProcessId,
    ) -> kernel::syscall::CommandReturn {
        match command_num {
            0 => CommandReturn::success(),
            1 => {
                let _ = self.runner.get().unwrap().execute();
                CommandReturn::success()
            }
            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, _process_id: ProcessId) -> Result<(), Error> {
        Ok(())
    }
}

impl<A: Alarm<'static> + 'static> AsyncDriver for HelloPrintDriver<A> {
    type F = impl Future<Output = ()> + 'static;

    fn run(&'static self) -> Self::F {
        async {
            // you should not be able to run two futures at the same time
            // so this should never panic
            let mut delay_instance = self.delay.get_instance().unwrap();
            // loop {
            debug!("Hello");
            delay_instance.delay_ns(1_000_000_000).await;
            debug!("awaited");
            // }
        }
    }

    fn done(&self, _value: ()) {
        debug!("done");
        self.runner.get().unwrap().execute().unwrap();
    }
}

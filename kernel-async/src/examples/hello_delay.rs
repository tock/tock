use alloc::boxed::Box;
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
    executor::{Executor, ExecutorClient, Runner},
};

pub struct HelloPrintDriver {
    runner: &'static dyn Runner<()>,
}

impl HelloPrintDriver {
    pub fn new(runner: &'static dyn Runner<()>) -> HelloPrintDriver {
        HelloPrintDriver { runner }
    }

    pub fn execute(&self) {
        self.runner.execute(()).unwrap();
    }
}

impl ExecutorClient for HelloPrintDriver {
    fn ready(&self, _t: ()) {
        todo!()
    }
}

impl SyscallDriver for HelloPrintDriver {
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
                let _ = self.runner.execute(());
                CommandReturn::success()
            }
            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, _process_id: ProcessId) -> Result<(), Error> {
        Ok(())
    }
}

pub unsafe fn create_hello_print_driver<'a, A: Alarm<'a>>(
    delay: &'static Delay<'a, A>,
) -> HelloPrintDriver {
    let runner = Box::leak(Box::new(Executor::new(|_| async {
        // you should not be able to run two futures at the same time
        // so this should never panic
        let mut delay_instance = delay.get_instance().unwrap();
        loop {
            debug!("Hello");
            delay_instance.delay_ns(1_000_000).await;
            debug!("awaited");
        }
    })));
    HelloPrintDriver::new(runner)
}

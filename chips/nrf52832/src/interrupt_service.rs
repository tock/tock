/// This macro defines a struct that, when initialized,
/// instantiates all peripheral drivers for the nrf52832 chip
/// in Tock. If a board
/// wishes to use only a subset of these peripherals, this
/// macro cannot be used, and this struct should be
/// constructed manually in main.rs. The input to the macro is the name of the struct
/// that will hold the peripherals, which can be chosen by the board.
#[macro_export]
macro_rules! create_default_nrf52832_peripherals {
    ($N:ident) => {
        use nrf52832::deferred_call_tasks::DeferredCallTask;
        //create all base nrf52 peripherals
        use nrf52832::*;
        nrf52832::create_default_nrf52_peripherals!(Nrf52BasePeripherals);
        struct $N<'a> {
            nrf52_base: Nrf52BasePeripherals<'a>,
            // put additional 52832 specific peripherals here
        }
        impl<'a> $N<'a> {
            fn new(ppi: &'a ppi::Ppi) -> Self {
                Self {
                    nrf52_base: unsafe { Nrf52BasePeripherals::new(&nrf52832::gpio::PORT, ppi) },
                }
            }
            // Necessary for setting up circular dependencies
            fn init(&'a self) {
                self.nrf52_base.init();
            }
        }
        impl<'a> kernel::InterruptService<DeferredCallTask> for $N<'a> {
            unsafe fn service_interrupt(&self, interrupt: u32) -> bool {
                self.nrf52_base.service_interrupt(interrupt)
            }
            unsafe fn service_deferred_call(&self, task: DeferredCallTask) -> bool {
                self.nrf52_base.service_deferred_call(task)
            }
        }
    };
}

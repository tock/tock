use crate::chip::E310xDefaultPeripherals;
use crate::interrupts;
use e310x::deferred_call_tasks::DeferredCallTask;

#[repr(transparent)]
pub struct E310G002DefaultPeripherals<'a> {
    pub e310x: E310xDefaultPeripherals<'a>,
}

impl<'a> E310G002DefaultPeripherals<'a> {
    pub unsafe fn new() -> Self {
        Self {
            e310x: E310xDefaultPeripherals::new(),
        }
    }
}
impl<'a> kernel::platform::chip::InterruptService<DeferredCallTask>
    for E310G002DefaultPeripherals<'a>
{
    unsafe fn service_interrupt(&self, interrupt: u32) -> bool {
        match interrupt {
            interrupts::UART0 => self.e310x.uart0.handle_interrupt(),
            interrupts::UART1 => self.e310x.uart1.handle_interrupt(),
            int_pin @ interrupts::GPIO0..=interrupts::GPIO31 => {
                let pin = &self.e310x.gpio_port[(int_pin - interrupts::GPIO0) as usize];
                pin.handle_interrupt();
            }

            // put E310x specific interrupts here
            _ => return self.e310x.service_interrupt(interrupt),
        }
        true
    }

    unsafe fn service_deferred_call(&self, task: DeferredCallTask) -> bool {
        self.e310x.service_deferred_call(task)
    }
}

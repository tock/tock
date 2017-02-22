use chip;
use core::cell::Cell;
use core::mem;
use kernel::hil::temp::{TempDriver, Client};
use nvic;
use peripheral_interrupts::NvicIdx;
use peripheral_registers::{TEMP_REGS, TEMP_BASE};

#[deny(no_mangle_const_items)]
#[no_mangle]
pub struct Temp {
    regs: *mut TEMP_REGS,
    client: Cell<Option<&'static Client>>,
}

pub static mut TEMP: Temp = Temp::new();

impl Temp {
    const fn new() -> Temp {
        Temp {
            regs: TEMP_BASE as *mut TEMP_REGS,
            client: Cell::new(None),
        }
    }

    pub fn init_temp(&self) {
        ()
    }

    #[inline(never)]
    #[no_mangle]
    fn measure(&self) {

        let regs: &mut TEMP_REGS = unsafe { mem::transmute(self.regs) };

        self.enable_nvic();
        self.enable_interrupts();

        regs.DATARDY.set(0);
        regs.START.set(1);
    }

    #[inline(never)]
    #[no_mangle]
    // MEASUREMENT DONE
    pub fn handle_interrupt(&self) {
        // ONLY DATARDY CAN TRIGGER THIS INTERRUPT
        let regs: &mut TEMP_REGS = unsafe { mem::transmute(self.regs) };

        // get temperature
        let temp = regs.TEMP.get() / 4;

        // stop measurement
        regs.STOP.set(1);

        // disable interrupts
        self.disable_nvic();
        self.disable_interrupts();

        // trigger callback with temperature
        self.client.get().map(|client| client.measurement_done(temp as usize));
        nvic::clear_pending(NvicIdx::TEMP);
    }


    #[inline(never)]
    #[no_mangle]
    fn enable_interrupts(&self) {
        let regs: &mut TEMP_REGS = unsafe { mem::transmute(self.regs) };
        // enable interrupts on DATARDY events
        regs.INTEN.set(1);
        regs.INTENSET.set(1);
    }

    fn disable_interrupts(&self) {
        let regs: &mut TEMP_REGS = unsafe { mem::transmute(self.regs) };
        // disable interrupts on DATARDY events
        regs.INTENCLR.set(1);
    }

    fn enable_nvic(&self) {
        nvic::enable(NvicIdx::TEMP);
    }

    fn disable_nvic(&self) {
        nvic::disable(NvicIdx::TEMP);
    }

    pub fn set_client<C: Client>(&self, client: &'static C) {
        self.client.set(Some(client));
    }
}
// Methods of RadioDummy Trait/Interface and are shared between Capsules and Chips
impl TempDriver for Temp {
    // This Function is called once Tock is booted
    fn init(&self) {
        self.init_temp()
    }

    #[inline(never)]
    #[no_mangle]
    fn take_measurement(&self) {
        self.measure()
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "C" fn TEMP_Handler() {
    use kernel::common::Queue;
    nvic::disable(NvicIdx::TEMP);
    chip::INTERRUPT_QUEUE.as_mut().unwrap().enqueue(NvicIdx::TEMP);
}

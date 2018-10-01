use peripheral_manager::{Peripheral, PeripheralManager};
use radio;

pub static mut M: PeripheralManager = PeripheralManager::new();

static mut RF_PERIPHERAL: Peripheral<'static> = unsafe { Peripheral::new(&radio::RADIO) };

pub unsafe fn init() {
    let peripherals = [&RF_PERIPHERAL];

    for peripheral in peripherals.iter() {
        M.register_peripheral(peripheral);
    }
}

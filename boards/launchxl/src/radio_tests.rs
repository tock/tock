use cc26x2::radio;
use core::cell::Cell;
use kernel::ReturnCode;

struct RadioConfig {
    tx_power: Cell<u16>,
    start_params: [u32; 18],
    active: bool,
}

pub fn power_on_test() -> bool {
    let radio = unsafe { &mut radio::RADIO };
    let is_on = radio.power_up();
    match is_on {
        ReturnCode::SUCCESS => true,
        ReturnCode::FAIL => false,
        _ => false,
    }
}

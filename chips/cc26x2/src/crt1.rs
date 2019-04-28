use cortexm4::nvic;
use tock_rt0;

extern "C" {
    // Symbols defined in the linker file
    static mut _erelocate: u32;
    static mut _etext: u32;
    static mut _ezero: u32;
    static mut _srelocate: u32;
    static mut _szero: u32;
    pub fn reset_handler();

    // _estack is not really a function, but it makes the types work
    // You should never actually invoke it!!
    pub fn _estack();
}

#[no_mangle]
pub unsafe extern "C" fn init() {
    tock_rt0::init_data(&mut _etext, &mut _srelocate, &mut _erelocate);
    tock_rt0::zero_bss(&mut _szero, &mut _ezero);
    nvic::enable_all();
}

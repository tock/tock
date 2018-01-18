use cortexm4::{nvic, systick_handler, SVC_Handler};

extern "C" {
    // Symbols defined in the linker file
    static mut _erelocate: u32;
    static mut _etext: u32;
    static mut _ezero: u32;
    static mut _srelocate: u32;
    static mut _szero: u32;
    fn reset_handler();

    // _estack is not really a function, but it makes the types work
    // You should never actually invoke it!!
    fn _estack();
}

unsafe extern "C" fn unhandled_interrupt() {
    'loop0: loop {}
}

unsafe extern "C" fn hard_fault_handler() {
    'loop0: loop {}
}

#[link_section=".vectors"]
#[cfg_attr(rustfmt, rustfmt_skip)]
// no_mangle Ensures that the symbol is kept until the final binary
#[no_mangle]
pub static BASE_VECTORS: [unsafe extern fn(); 50] = [
    _estack,
    reset_handler,
    unhandled_interrupt, // NMI
    hard_fault_handler, // Hard Fault
    unhandled_interrupt, // MPU fault
    unhandled_interrupt, // Bus fault
    unhandled_interrupt, // Usage fault
    unhandled_interrupt, // Reserved
    unhandled_interrupt, // Reserved
    unhandled_interrupt, // Reserved
    unhandled_interrupt, // Reserved
    SVC_Handler, // SVC
    unhandled_interrupt, // Debug monitor,
    unhandled_interrupt, // Reserved
    unhandled_interrupt, // PendSV
    systick_handler, // Systick
    unhandled_interrupt, // GPIO Int handler
    unhandled_interrupt, // I2C
    unhandled_interrupt, // RF Core Command & Packet Engine 1
    unhandled_interrupt, // AON SpiSplave Rx, Tx and CS
    unhandled_interrupt, // AON RTC
    unhandled_interrupt, // UART0 Rx and Tx
    unhandled_interrupt, // AUX software event 0
    unhandled_interrupt, // SSI0 Rx and Tx
    unhandled_interrupt, // SSI1 Rx and Tx
    unhandled_interrupt, // RF Core Command & Packet Engine 0
    unhandled_interrupt, // RF Core Hardware
    unhandled_interrupt, // RF Core Command Acknowledge
    unhandled_interrupt, // I2S
    unhandled_interrupt, // AUX software event 1
    unhandled_interrupt, // Watchdog timer
    unhandled_interrupt, // Timer 0 subtimer A
    unhandled_interrupt, // Timer 0 subtimer B
    unhandled_interrupt, // Timer 1 subtimer A
    unhandled_interrupt, // Timer 1 subtimer B
    unhandled_interrupt, // Timer 2 subtimer A
    unhandled_interrupt, // Timer 2 subtimer B
    unhandled_interrupt, // Timer 3 subtimer A
    unhandled_interrupt, // Timer 3 subtimer B
    unhandled_interrupt, // Crypto Core Result available
    unhandled_interrupt, // uDMA Software
    unhandled_interrupt, // uDMA Error
    unhandled_interrupt, // Flash controller
    unhandled_interrupt, // Software Event 0
    unhandled_interrupt, // AUX combined event
    unhandled_interrupt, // AON programmable 0
    unhandled_interrupt, // Dynamic Programmable interrupt
    // source (Default: PRCM)
    unhandled_interrupt, // AUX Comparator A
    unhandled_interrupt, // AUX ADC new sample or ADC DMA
    // done, ADC underflow, ADC overflow
    unhandled_interrupt  // TRNG event
];

#[no_mangle]
pub unsafe extern "C" fn init() {
    let mut current_block;
    let mut p_src: *mut u32;
    let mut p_dest: *mut u32;

    // Move the relocate segment. This assumes it is located after the text
    // segment, which is where the storm linker file puts it
    p_src = &mut _etext as (*mut u32);
    p_dest = &mut _srelocate as (*mut u32);
    if p_src != p_dest {
        current_block = 1;
    } else {
        current_block = 2;
    }
    'loop1: loop {
        if current_block == 1 {
            if !(p_dest < &mut _erelocate as (*mut u32)) {
                current_block = 2;
                continue;
            }
            *{
                let _old = p_dest;
                p_dest = p_dest.offset(1isize);
                _old
            } = *{
                let _old = p_src;
                p_src = p_src.offset(1isize);
                _old
            };
            current_block = 1;
        } else {
            p_dest = &mut _szero as (*mut u32);
            break;
        }
    }
    'loop3: loop {
        if !(p_dest < &mut _ezero as (*mut u32)) {
            break;
        }
        *{
            let _old = p_dest;
            p_dest = p_dest.offset(1isize);
            _old
        } = 0u32;
    }
    nvic::enable_all();
}
